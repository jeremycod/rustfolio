// NOTE: This file requires database migrations to be run first:
// - 20260218100000_add_cache_status_columns.sql
// - 20260218100001_create_correlations_cache.sql
// Run: sqlx migrate run

//! Portfolio Risk Pre-calculation Background Job
//!
//! This job runs hourly at :15 (e.g., 1:15 AM, 2:15 AM, 3:15 AM) to pre-calculate
//! portfolio risk metrics for all portfolios in the system. By pre-calculating risk
//! metrics, we ensure fast response times for API requests and reduce load on external
//! price APIs.
//!
//! # Job Schedule
//!
//! - **Production**: Every hour at :15 (0 15 * * * *)
//! - **Test Mode**: Every 5 minutes (0 */5 * * * *)
//!
//! # Processing Strategy
//!
//! 1. Query all portfolios with active holdings
//! 2. For each portfolio, check cache status:
//!    - Skip if status is 'fresh' and not expired
//!    - Skip if status is 'calculating' (another job is processing)
//!    - Process if status is 'stale', 'error', or expired
//! 3. Mark cache as 'calculating' before starting
//! 4. Calculate risk metrics using standard parameters:
//!    - 90-day rolling window
//!    - SPY benchmark
//! 5. Store results with status 'fresh' on success
//! 6. Store error details with status 'error' on failure
//! 7. Add 1-second delay between portfolios to avoid rate limiting
//!
//! # Error Handling
//!
//! - Individual portfolio failures don't stop the entire job
//! - Errors are logged and stored in the cache table
//! - Retry count is incremented for failed calculations
//! - Detailed logging helps with debugging and monitoring
//!
//! # Performance Considerations
//!
//! - Uses batch price fetching where possible
//! - Implements 60-second timeout per portfolio
//! - Adds delays between portfolios to respect rate limits
//! - Skips portfolios with no holdings or negligible value

use crate::db::holding_snapshot_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::risk::{PortfolioRiskWithViolations, ThresholdViolation, ViolationSeverity};
use crate::models::{PositionRiskContribution, RiskLevel};
use crate::services::failure_cache::FailureCache;
use crate::services::rate_limiter::RateLimiter;
use crate::services::{job_scheduler_service::{JobContext, JobResult}, risk_service};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Standard parameters for portfolio risk calculations
const DEFAULT_DAYS: i64 = 90;
const DEFAULT_BENCHMARK: &str = "SPY";
const CACHE_EXPIRATION_HOURS: i64 = 4;
const PORTFOLIO_TIMEOUT_SECONDS: u64 = 60;
const INTER_PORTFOLIO_DELAY_MS: u64 = 1000;

/// Main entry point for the portfolio risk calculation job.
///
/// This function is called by the job scheduler on the defined schedule.
/// It processes all portfolios in the system, calculating risk metrics
/// and storing them in the cache for fast API responses.
///
/// # Arguments
///
/// * `ctx` - Job context containing database pool and external services
///
/// # Returns
///
/// * `Ok(JobResult)` - Success with counts of processed and failed portfolios
/// * `Err(AppError)` - Critical failure that should stop the job
pub async fn calculate_all_portfolio_risks(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔍 Starting portfolio risk pre-calculation job");

    // Query all portfolios with holdings
    let portfolios = query_portfolios_with_holdings(&ctx.pool).await?;

    if portfolios.is_empty() {
        info!("No portfolios with holdings found, nothing to process");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} portfolios to process", portfolios.len());

    let mut processed = 0;
    let mut failed = 0;

    // Process each portfolio
    for portfolio_id in portfolios {
        // Check if cache needs refresh
        match check_cache_needs_refresh(&ctx.pool, portfolio_id, DEFAULT_DAYS, DEFAULT_BENCHMARK).await {
            Ok(needs_refresh) => {
                if !needs_refresh {
                    info!("Portfolio {} cache is fresh, skipping", portfolio_id);
                    processed += 1;
                    continue;
                }
            }
            Err(e) => {
                warn!("Failed to check cache status for portfolio {}: {}", portfolio_id, e);
                // Continue processing - assume needs refresh
            }
        }

        info!("Processing portfolio {}...", portfolio_id);

        // Mark cache as 'calculating'
        if let Err(e) = mark_cache_calculating(&ctx.pool, portfolio_id, DEFAULT_DAYS, DEFAULT_BENCHMARK).await {
            error!("Failed to mark cache as calculating for portfolio {}: {}", portfolio_id, e);
            failed += 1;
            continue;
        }

        // Calculate risk metrics with timeout
        let calculation_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(PORTFOLIO_TIMEOUT_SECONDS),
            calculate_portfolio_risk_internal(
                &ctx.pool,
                portfolio_id,
                DEFAULT_DAYS,
                DEFAULT_BENCHMARK,
                ctx.price_provider.as_ref(),
                ctx.failure_cache.as_ref(),
                ctx.rate_limiter.as_ref(),
            )
        ).await;

        match calculation_result {
            Ok(Ok(risk_data)) => {
                // Successfully calculated risk metrics
                if let Err(e) = store_portfolio_risk_cache(
                    &ctx.pool,
                    portfolio_id,
                    DEFAULT_DAYS,
                    DEFAULT_BENCHMARK,
                    &risk_data,
                ).await {
                    error!("Failed to store risk cache for portfolio {}: {}", portfolio_id, e);
                    mark_cache_error(&ctx.pool, portfolio_id, DEFAULT_DAYS, DEFAULT_BENCHMARK, &e.to_string()).await.ok();
                    failed += 1;
                } else {
                    info!("✅ Successfully calculated and cached risk for portfolio {}", portfolio_id);
                    processed += 1;
                }
            }
            Ok(Err(e)) => {
                // Calculation failed
                error!("Failed to calculate risk for portfolio {}: {}", portfolio_id, e);
                mark_cache_error(&ctx.pool, portfolio_id, DEFAULT_DAYS, DEFAULT_BENCHMARK, &e.to_string()).await.ok();
                failed += 1;
            }
            Err(_) => {
                // Timeout
                let error_msg = format!("Calculation timed out after {} seconds", PORTFOLIO_TIMEOUT_SECONDS);
                error!("{} for portfolio {}", error_msg, portfolio_id);
                mark_cache_error(&ctx.pool, portfolio_id, DEFAULT_DAYS, DEFAULT_BENCHMARK, &error_msg).await.ok();
                failed += 1;
            }
        }

        // Add delay between portfolios to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_PORTFOLIO_DELAY_MS)).await;
    }

    info!(
        "✅ Portfolio risk job completed: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Query all portfolios that have holdings.
///
/// This function queries the database for portfolios with at least one holding,
/// ordered by portfolio ID for consistent processing order.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// * `Ok(Vec<Uuid>)` - List of portfolio IDs
/// * `Err(AppError)` - Database query error
async fn query_portfolios_with_holdings(pool: &PgPool) -> Result<Vec<Uuid>, AppError> {
    let portfolio_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT DISTINCT p.id
        FROM portfolios p
        INNER JOIN accounts a ON a.portfolio_id = p.id
        INNER JOIN holding_snapshots hs ON hs.account_id = a.id
        ORDER BY p.id
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(portfolio_ids)
}

/// Check if the cache needs to be refreshed for a portfolio.
///
/// Returns true if:
/// - No cache entry exists
/// - Cache status is 'stale'
/// - Cache status is 'error' (to retry)
/// - Cache has expired (expires_at < now)
///
/// Returns false if:
/// - Cache status is 'fresh' and not expired
/// - Cache status is 'calculating' (another process is working on it)
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio UUID
/// * `days` - Rolling window in days
/// * `benchmark` - Benchmark ticker
///
/// # Returns
///
/// * `Ok(bool)` - True if cache needs refresh
/// * `Err(AppError)` - Database query error
async fn check_cache_needs_refresh(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
) -> Result<bool, AppError> {
    #[derive(sqlx::FromRow)]
    struct CacheRow {
        calculation_status: Option<String>,
        expires_at: chrono::DateTime<Utc>,
    }

    let result = sqlx::query_as::<_, CacheRow>(
        r#"
        SELECT calculation_status, expires_at
        FROM portfolio_risk_cache
        WHERE portfolio_id = $1 AND days = $2 AND benchmark = $3
        "#
    )
    .bind(portfolio_id)
    .bind(days as i32)
    .bind(benchmark)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)?;

    match result {
        None => {
            // No cache entry exists, needs refresh
            Ok(true)
        }
        Some(row) => {
            let status = row.calculation_status.as_deref().unwrap_or("stale");
            let expires_at = row.expires_at;

            // Check if expired
            if expires_at < Utc::now() {
                info!("Cache expired for portfolio {} (status: {})", portfolio_id, status);
                return Ok(true);
            }

            // Check status
            match status {
                "fresh" => {
                    // Fresh and not expired, no refresh needed
                    Ok(false)
                }
                "calculating" => {
                    // Another process is calculating, skip
                    info!("Portfolio {} is already being calculated by another job", portfolio_id);
                    Ok(false)
                }
                "stale" | "error" => {
                    // Needs refresh
                    Ok(true)
                }
                _ => {
                    warn!("Unknown cache status '{}' for portfolio {}, treating as stale", status, portfolio_id);
                    Ok(true)
                }
            }
        }
    }
}

/// Mark the cache entry as 'calculating' to prevent duplicate work.
///
/// This function updates or inserts a cache entry with status 'calculating'
/// to indicate that this portfolio is currently being processed.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio UUID
/// * `days` - Rolling window in days
/// * `benchmark` - Benchmark ticker
///
/// # Returns
///
/// * `Ok(())` - Successfully marked as calculating
/// * `Err(AppError)` - Database update error
async fn mark_cache_calculating(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO portfolio_risk_cache (portfolio_id, days, benchmark, risk_data, calculated_at, expires_at, calculation_status)
        VALUES ($1, $2, $3, '{}'::jsonb, NOW(), NOW() + INTERVAL '4 hours', 'calculating')
        ON CONFLICT (portfolio_id, days, benchmark)
        DO UPDATE SET
            calculation_status = 'calculating',
            updated_at = NOW()
        "#
    )
    .bind(portfolio_id)
    .bind(days as i32)
    .bind(benchmark)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(())
}

/// Store the calculated portfolio risk in the cache with status 'fresh'.
///
/// This function serializes the risk data to JSON and stores it in the cache
/// table with a 4-hour expiration time. The status is set to 'fresh' and any
/// previous error information is cleared.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio UUID
/// * `days` - Rolling window in days
/// * `benchmark` - Benchmark ticker
/// * `risk_data` - Calculated risk data
///
/// # Returns
///
/// * `Ok(())` - Successfully stored in cache
/// * `Err(AppError)` - Serialization or database error
async fn store_portfolio_risk_cache(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
    risk_data: &PortfolioRiskWithViolations,
) -> Result<(), AppError> {
    let risk_json = serde_json::to_value(risk_data)
        .map_err(|e| AppError::External(format!("Failed to serialize risk data: {}", e)))?;

    let calculated_at = Utc::now();
    let expires_at = calculated_at + Duration::hours(CACHE_EXPIRATION_HOURS);

    sqlx::query(
        r#"
        INSERT INTO portfolio_risk_cache (
            portfolio_id, days, benchmark, risk_data, calculated_at, expires_at,
            calculation_status, last_error, retry_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'fresh', NULL, 0)
        ON CONFLICT (portfolio_id, days, benchmark)
        DO UPDATE SET
            risk_data = $4,
            calculated_at = $5,
            expires_at = $6,
            calculation_status = 'fresh',
            last_error = NULL,
            retry_count = 0,
            updated_at = NOW()
        "#
    )
    .bind(portfolio_id)
    .bind(days as i32)
    .bind(benchmark)
    .bind(risk_json)
    .bind(calculated_at)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(())
}

/// Mark the cache entry as 'error' with error details.
///
/// This function updates the cache entry to record a calculation failure,
/// including the error message and incrementing the retry count.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio UUID
/// * `days` - Rolling window in days
/// * `benchmark` - Benchmark ticker
/// * `error_message` - Error description
///
/// # Returns
///
/// * `Ok(())` - Successfully marked as error
/// * `Err(AppError)` - Database update error
async fn mark_cache_error(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
    error_message: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO portfolio_risk_cache (
            portfolio_id, days, benchmark, risk_data, calculated_at, expires_at,
            calculation_status, last_error, retry_count
        )
        VALUES ($1, $2, $3, '{}'::jsonb, NOW(), NOW() + INTERVAL '1 hour', 'error', $4, 1)
        ON CONFLICT (portfolio_id, days, benchmark)
        DO UPDATE SET
            calculation_status = 'error',
            last_error = $4,
            retry_count = COALESCE(portfolio_risk_cache.retry_count, 0) + 1,
            updated_at = NOW()
        "#
    )
    .bind(portfolio_id)
    .bind(days as i32)
    .bind(benchmark)
    .bind(error_message)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(())
}

/// Calculate portfolio risk metrics for a single portfolio.
///
/// This is the core calculation function that:
/// 1. Fetches portfolio holdings
/// 2. Aggregates holdings by ticker
/// 3. Calculates risk metrics for each position
/// 4. Computes portfolio-level aggregated metrics
/// 5. Detects threshold violations
///
/// This function is adapted from the logic in `routes/risk.rs::get_portfolio_risk()`
/// but optimized for batch processing.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio UUID
/// * `days` - Rolling window in days
/// * `benchmark` - Benchmark ticker
/// * `price_provider` - External price data provider
/// * `failure_cache` - Cache to avoid repeated failed API calls
/// * `rate_limiter` - Rate limiter for API requests
///
/// # Returns
///
/// * `Ok(PortfolioRiskWithViolations)` - Calculated risk data
/// * `Err(AppError)` - Calculation error
async fn calculate_portfolio_risk_internal(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    rate_limiter: &RateLimiter,
) -> Result<PortfolioRiskWithViolations, AppError> {
    // 1. Fetch all latest holdings for the portfolio
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(
        pool,
        portfolio_id
    ).await.map_err(|e| {
        error!("Failed to fetch portfolio holdings: {}", e);
        AppError::Db(e)
    })?;

    if holdings.is_empty() {
        return Err(AppError::External(
            "Portfolio has no holdings".to_string()
        ));
    }

    // 2. Aggregate holdings by ticker (same ticker across multiple accounts)
    let mut ticker_aggregates: HashMap<String, (f64, f64)> = HashMap::new(); // (quantity, market_value)

    for holding in &holdings {
        let market_value = holding.market_value.to_string().parse::<f64>().unwrap_or(0.0);
        let quantity = holding.quantity.to_string().parse::<f64>().unwrap_or(0.0);

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|(q, mv)| {
                *q += quantity;
                *mv += market_value;
            })
            .or_insert((quantity, market_value));
    }

    // Calculate total portfolio value
    let total_value: f64 = ticker_aggregates.values().map(|(_, mv)| mv).sum();

    if total_value == 0.0 {
        return Err(AppError::External(
            "Portfolio has no holdings with market value".to_string()
        ));
    }

    // 3. Compute risk metrics for each ticker and collect contributions
    let mut position_risks = Vec::new();
    let mut weighted_volatility = 0.0;
    let mut weighted_max_drawdown = 0.0;
    let mut weighted_beta = 0.0;
    let mut weighted_sharpe = 0.0;
    let mut beta_count = 0;
    let mut sharpe_count = 0;

    // Get risk free rate from environment or use default
    let risk_free_rate = std::env::var("RISK_FREE_RATE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.045); // Default 4.5%

    for (ticker, (_quantity, market_value)) in ticker_aggregates {
        // Skip positions with negligible value (< 0.1% of portfolio)
        let weight = market_value / total_value;
        if weight < 0.001 {
            continue;
        }

        // Compute risk metrics for this ticker
        match risk_service::compute_risk_metrics(
            pool,
            &ticker,
            days,
            benchmark,
            price_provider,
            failure_cache,
            rate_limiter,
            risk_free_rate,
        ).await {
            Ok(assessment) => {
                // Weight metrics by position size
                weighted_volatility += assessment.metrics.volatility * weight;
                weighted_max_drawdown += assessment.metrics.max_drawdown * weight;

                if let Some(beta) = assessment.metrics.beta {
                    weighted_beta += beta * weight;
                    beta_count += 1;
                }

                if let Some(sharpe) = assessment.metrics.sharpe {
                    weighted_sharpe += sharpe * weight;
                    sharpe_count += 1;
                }

                position_risks.push(PositionRiskContribution {
                    ticker: ticker.clone(),
                    market_value,
                    weight,
                    risk_assessment: assessment,
                });
            },
            Err(e) => {
                // Log but don't fail - some positions might not have risk data
                warn!("Could not compute risk for {} in portfolio {}: {}", ticker, portfolio_id, e);
            }
        }
    }

    if position_risks.is_empty() {
        return Err(AppError::External(
            "No positions in portfolio have available risk data".to_string()
        ));
    }

    // 4. Calculate portfolio-level risk score
    let portfolio_risk_score = risk_service::score_risk(&crate::models::PositionRisk {
        volatility: weighted_volatility,
        max_drawdown: weighted_max_drawdown,
        beta: if beta_count > 0 { Some(weighted_beta) } else { None },
        beta_spy: if beta_count > 0 { Some(weighted_beta) } else { None },
        beta_qqq: None,
        beta_iwm: None,
        risk_decomposition: None,
        sharpe: if sharpe_count > 0 { Some(weighted_sharpe) } else { None },
        sortino: None,
        annualized_return: None,
        value_at_risk: None,
        var_95: None,
        var_99: None,
        expected_shortfall_95: None,
        expected_shortfall_99: None,
    });

    let risk_level = RiskLevel::from_score(portfolio_risk_score);

    // 5. Sort positions by risk contribution (highest to lowest)
    position_risks.sort_by(|a, b| {
        b.risk_assessment.risk_score.partial_cmp(&a.risk_assessment.risk_score).unwrap()
    });

    let portfolio_risk = crate::models::PortfolioRisk {
        portfolio_id: portfolio_id.to_string(),
        total_value,
        portfolio_volatility: weighted_volatility,
        portfolio_max_drawdown: weighted_max_drawdown,
        portfolio_beta: if beta_count > 0 { Some(weighted_beta) } else { None },
        portfolio_sharpe: if sharpe_count > 0 { Some(weighted_sharpe) } else { None },
        portfolio_risk_score,
        risk_level,
        position_risks: position_risks.clone(),
    };

    // 6. Fetch risk thresholds
    let thresholds = crate::db::risk_threshold_queries::get_thresholds(pool, portfolio_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch risk thresholds: {}", e);
            AppError::Db(e)
        })?;

    // 7. Detect threshold violations
    let violations = detect_violations(&portfolio_risk, &thresholds);

    Ok(PortfolioRiskWithViolations {
        portfolio_risk,
        thresholds,
        violations,
    })
}

/// Detect threshold violations in portfolio risk data.
///
/// This function checks each position's risk metrics against the configured
/// thresholds and generates violations for metrics that exceed warning or
/// critical thresholds.
///
/// # Arguments
///
/// * `portfolio_risk` - Portfolio risk data with all positions
/// * `thresholds` - Risk threshold settings
///
/// # Returns
///
/// Vector of threshold violations (empty if no violations)
fn detect_violations(
    portfolio_risk: &crate::models::PortfolioRisk,
    thresholds: &crate::models::risk::RiskThresholdSettings,
) -> Vec<ThresholdViolation> {
    let mut violations = Vec::new();

    // Check each position for violations
    for position in &portfolio_risk.position_risks {
        let metrics = &position.risk_assessment.metrics;

        // Check volatility
        if metrics.volatility >= thresholds.volatility_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Volatility".to_string(),
                metric_value: metrics.volatility,
                threshold_value: thresholds.volatility_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if metrics.volatility >= thresholds.volatility_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Volatility".to_string(),
                metric_value: metrics.volatility,
                threshold_value: thresholds.volatility_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check max drawdown (more negative is worse)
        if metrics.max_drawdown <= thresholds.drawdown_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Max Drawdown".to_string(),
                metric_value: metrics.max_drawdown,
                threshold_value: thresholds.drawdown_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if metrics.max_drawdown <= thresholds.drawdown_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Max Drawdown".to_string(),
                metric_value: metrics.max_drawdown,
                threshold_value: thresholds.drawdown_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check beta
        if let Some(beta) = metrics.beta {
            if beta >= thresholds.beta_critical_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Beta".to_string(),
                    metric_value: beta,
                    threshold_value: thresholds.beta_critical_threshold,
                    threshold_type: ViolationSeverity::Critical,
                });
            } else if beta >= thresholds.beta_warning_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Beta".to_string(),
                    metric_value: beta,
                    threshold_value: thresholds.beta_warning_threshold,
                    threshold_type: ViolationSeverity::Warning,
                });
            }
        }

        // Check risk score
        let risk_score = position.risk_assessment.risk_score;
        if risk_score >= thresholds.risk_score_critical_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Risk Score".to_string(),
                metric_value: risk_score,
                threshold_value: thresholds.risk_score_critical_threshold,
                threshold_type: ViolationSeverity::Critical,
            });
        } else if risk_score >= thresholds.risk_score_warning_threshold {
            violations.push(ThresholdViolation {
                ticker: position.ticker.clone(),
                holding_name: None,
                metric_name: "Risk Score".to_string(),
                metric_value: risk_score,
                threshold_value: thresholds.risk_score_warning_threshold,
                threshold_type: ViolationSeverity::Warning,
            });
        }

        // Check VaR (more negative is worse)
        if let Some(var) = metrics.value_at_risk {
            if var <= thresholds.var_critical_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Value at Risk".to_string(),
                    metric_value: var,
                    threshold_value: thresholds.var_critical_threshold,
                    threshold_type: ViolationSeverity::Critical,
                });
            } else if var <= thresholds.var_warning_threshold {
                violations.push(ThresholdViolation {
                    ticker: position.ticker.clone(),
                    holding_name: None,
                    metric_name: "Value at Risk".to_string(),
                    metric_value: var,
                    threshold_value: thresholds.var_warning_threshold,
                    threshold_type: ViolationSeverity::Warning,
                });
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_DAYS, 90);
        assert_eq!(DEFAULT_BENCHMARK, "SPY");
        assert_eq!(CACHE_EXPIRATION_HOURS, 4);
        assert_eq!(PORTFOLIO_TIMEOUT_SECONDS, 60);
        assert_eq!(INTER_PORTFOLIO_DELAY_MS, 1000);
    }
}

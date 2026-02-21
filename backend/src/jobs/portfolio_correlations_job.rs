// NOTE: This file requires database migrations to be run first:
// - 20260218100000_add_cache_status_columns.sql
// - 20260218100001_create_correlations_cache.sql
// Run: sqlx migrate run

/// Portfolio Correlations Background Job
///
/// This background job pre-calculates correlation matrices for all portfolios with holdings.
/// Correlation calculations are computationally expensive as they require:
/// - Fetching holdings data for entire portfolio
/// - Batch querying historical price data across multiple tickers
/// - Computing pairwise correlations between all positions
/// - Building 2D correlation matrices for visualization
///
/// **Schedule**: Every 2 hours at :45 (e.g., 00:45, 02:45, 04:45, etc.)
///
/// **Why Less Frequent Than Risk?**
/// Correlations change more slowly than individual risk metrics:
/// - Correlation relationships are relatively stable intraday
/// - Price movements affect risk metrics (volatility, VaR) more immediately
/// - Correlation calculations are more expensive (O(n²) for n positions)
/// - 2-hour refresh provides good balance of freshness vs. resource usage
///
/// **Cache Strategy**:
/// - Results stored in `portfolio_correlations_cache` with 24-hour expiry
/// - Cache checked before calculation to avoid redundant work
/// - Graceful error handling - failures don't stop other portfolios
/// - 2-second delay between portfolios to prevent database overload
///
/// **Performance Optimization**:
/// - Limited to top 10 positions by value to prevent timeouts
/// - Batch price fetching for all tickers at once
/// - Filters out mutual funds and proprietary tickers (no price data)
/// - Only positions >= 1% of portfolio value are included

use crate::db::{holding_snapshot_queries, price_queries};
use crate::errors::AppError;
use crate::models::risk::{CorrelationMatrix, CorrelationMatrixWithStats, CorrelationPair};
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::risk_service;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Main entry point for the portfolio correlations background job.
///
/// This function is called by the job scheduler according to the cron schedule.
/// It processes all portfolios sequentially, checking cache freshness and
/// calculating correlations only when needed.
///
/// # Arguments
/// * `ctx` - Job context containing database pool and other shared resources
///
/// # Returns
/// * `Ok(JobResult)` - Success with counts of processed/failed portfolios
/// * `Err(AppError)` - Critical error that stops the entire job
pub async fn calculate_all_portfolio_correlations(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔗 Starting portfolio correlations calculation job...");

    // Get all portfolios that have at least one account (which would have holdings)
    let portfolios = sqlx::query!(
        r#"
        SELECT DISTINCT p.id, p.name
        FROM portfolios p
        INNER JOIN accounts a ON p.id = a.portfolio_id
        ORDER BY p.name
        "#
    )
    .fetch_all(ctx.pool.as_ref())
    .await?;

    if portfolios.is_empty() {
        info!("No portfolios with accounts found");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} portfolios to process", portfolios.len());

    let mut processed = 0;
    let mut failed = 0;

    // Standard correlation lookback period (90 days)
    let days: i64 = 90;

    for portfolio in portfolios {
        let portfolio_id = portfolio.id;
        let portfolio_name = &portfolio.name;

        info!(
            "Processing portfolio: {} ({})",
            portfolio_name, portfolio_id
        );

        // Check if cache needs refresh
        match check_correlations_cache_needs_refresh(ctx.pool.as_ref(), portfolio_id, days).await
        {
            Ok(needs_refresh) => {
                if !needs_refresh {
                    info!(
                        "✓ Skipping {} - cache is fresh",
                        portfolio_name
                    );
                    processed += 1;
                    continue;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to check cache for {}: {}",
                    portfolio_name, e
                );
                // Continue anyway - better to recalculate than skip
            }
        }

        // Calculate correlations for this portfolio
        match calculate_portfolio_correlations_internal(ctx.pool.as_ref(), portfolio_id, days)
            .await
        {
            Ok(result) => {
                // Store in cache
                match store_correlations_cache(ctx.pool.as_ref(), portfolio_id, days, &result)
                    .await
                {
                    Ok(_) => {
                        processed += 1;
                        info!(
                            "✅ Successfully calculated correlations for {} ({} tickers)",
                            portfolio_name, result.matrix.tickers.len()
                        );
                    }
                    Err(e) => {
                        failed += 1;
                        error!(
                            "Failed to store correlations cache for {}: {}",
                            portfolio_name, e
                        );
                    }
                }
            }
            Err(e) => {
                failed += 1;
                // Store error in cache to prevent repeated failures
                if let Err(cache_err) =
                    store_correlations_error(ctx.pool.as_ref(), portfolio_id, days, &e.to_string())
                        .await
                {
                    error!(
                        "Failed to store error in cache for {}: {}",
                        portfolio_name, cache_err
                    );
                }
                warn!(
                    "Failed to calculate correlations for {}: {}",
                    portfolio_name, e
                );
            }
        }

        // Add delay between portfolios to avoid overloading database
        // Correlations are more expensive than risk calculations
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!(
        "🏁 Portfolio correlations job completed: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Check if correlation cache needs refresh.
///
/// Returns true if:
/// - No cache entry exists
/// - Cache has expired (expires_at < NOW)
/// - Cache status is 'stale' or 'error'
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio to check
/// * `days` - Lookback period for correlation calculation
async fn check_correlations_cache_needs_refresh(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT expires_at, calculation_status
        FROM portfolio_correlations_cache
        WHERE portfolio_id = $1 AND days = $2
        "#,
        portfolio_id,
        days as i32
    )
    .fetch_optional(pool)
    .await?;

    match result {
        None => Ok(true), // No cache entry - needs calculation
        Some(record) => {
            let now = chrono::Utc::now();
            let expired = record.expires_at < now;
            let is_error = record
                .calculation_status
                .as_ref()
                .map(|s: &String| s.as_str() == "error" || s.as_str() == "stale")
                .unwrap_or(false);

            Ok(expired || is_error)
        }
    }
}

/// Store successful correlation calculation in cache.
///
/// Cache entries expire after 24 hours since correlations are relatively stable.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio ID
/// * `days` - Lookback period used
/// * `result` - Calculated correlation matrix with statistics
async fn store_correlations_cache(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    result: &CorrelationMatrixWithStats,
) -> Result<(), AppError> {
    let correlations_json = serde_json::to_value(result)
        .map_err(|e| AppError::External(format!("Failed to serialize correlations: {}", e)))?;
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    sqlx::query!(
        r#"
        INSERT INTO portfolio_correlations_cache
            (portfolio_id, days, correlations_data, calculated_at, expires_at, calculation_status, last_error)
        VALUES ($1, $2, $3, NOW(), $4, 'fresh', NULL)
        ON CONFLICT (portfolio_id, days)
        DO UPDATE SET
            correlations_data = EXCLUDED.correlations_data,
            calculated_at = NOW(),
            expires_at = EXCLUDED.expires_at,
            calculation_status = 'fresh',
            last_error = NULL,
            updated_at = NOW()
        "#,
        portfolio_id,
        days as i32,
        correlations_json,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Store error information in cache to prevent repeated failed calculations.
///
/// When a calculation fails, we cache the error for 1 hour to avoid
/// hammering the database with repeated failing calculations.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio ID
/// * `days` - Lookback period attempted
/// * `error_message` - Error message to store
async fn store_correlations_error(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
    error_message: &str,
) -> Result<(), AppError> {
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    sqlx::query!(
        r#"
        INSERT INTO portfolio_correlations_cache
            (portfolio_id, days, correlations_data, calculated_at, expires_at, calculation_status, last_error)
        VALUES ($1, $2, '{}'::jsonb, NOW(), $3, 'error', $4)
        ON CONFLICT (portfolio_id, days)
        DO UPDATE SET
            calculated_at = NOW(),
            expires_at = EXCLUDED.expires_at,
            calculation_status = 'error',
            last_error = EXCLUDED.last_error,
            updated_at = NOW()
        "#,
        portfolio_id,
        days as i32,
        expires_at,
        error_message
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Calculate correlation matrix for a single portfolio.
///
/// This function is adapted from the logic in `routes/risk.rs::get_portfolio_correlations()`.
/// It performs the following steps:
/// 1. Fetch portfolio holdings
/// 2. Aggregate by ticker and filter mutual funds
/// 3. Apply position size threshold (1% of portfolio)
/// 4. Limit to top 10 positions by value (performance optimization)
/// 5. Batch fetch price data for all tickers
/// 6. Calculate pairwise correlations
/// 7. Build 2D correlation matrix
/// 8. Calculate correlation statistics
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `portfolio_id` - Portfolio to analyze
/// * `days` - Number of days of historical data for correlation
///
/// # Returns
/// * `Ok(CorrelationMatrixWithStats)` - Correlation matrix with statistics
/// * `Err(AppError)` - Calculation failed (insufficient data, DB error, etc.)
async fn calculate_portfolio_correlations_internal(
    pool: &PgPool,
    portfolio_id: Uuid,
    days: i64,
) -> Result<CorrelationMatrixWithStats, AppError> {
    // 1. Fetch all latest holdings for the portfolio
    let holdings =
        holding_snapshot_queries::fetch_portfolio_latest_holdings(pool, portfolio_id).await?;

    if holdings.is_empty() {
        return Err(AppError::External(format!(
            "No holdings found for portfolio {}",
            portfolio_id
        )));
    }

    // 2. Aggregate holdings by ticker and filter out mutual funds and proprietary tickers
    let mut ticker_aggregates: HashMap<String, f64> = HashMap::new();
    let mut total_value = 0.0;
    let mut filtered_count = 0;

    for holding in &holdings {
        let market_value = holding
            .market_value
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);
        total_value += market_value;

        // Skip mutual funds and proprietary tickers (no price data available)
        let is_mutual_fund = holding
            .industry
            .as_ref()
            .map(|i| i.to_lowercase().contains("mutual fund"))
            .unwrap_or(false);

        let is_proprietary_ticker = holding.ticker.starts_with("FID")
            || holding.ticker.starts_with("RBF")
            || holding.ticker.starts_with("LYZ")
            || holding.ticker.starts_with("BIP")
            || holding.ticker.starts_with("DYN")
            || holding.ticker.starts_with("EDG")
            || holding.ticker.len() > 5;

        if is_mutual_fund || is_proprietary_ticker {
            filtered_count += 1;
            continue;
        }

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|mv| *mv += market_value)
            .or_insert(market_value);
    }

    if total_value == 0.0 {
        return Err(AppError::External(
            "Portfolio has no holdings with market value".to_string(),
        ));
    }

    // Filter to only include positions >= 1% of portfolio value
    let min_value = total_value * 0.01;
    let mut tickers: Vec<String> = ticker_aggregates
        .iter()
        .filter(|(_, &market_value)| market_value >= min_value)
        .map(|(ticker, _)| ticker.clone())
        .collect();

    tickers.sort();

    // Limit to top 10 positions by value to prevent timeout
    // (10 tickers = 45 correlation pairs, vs 20 tickers = 190 pairs)
    if tickers.len() > 10 {
        let mut ticker_values: Vec<(String, f64)> = ticker_aggregates
            .iter()
            .map(|(t, &v)| (t.clone(), v))
            .collect();
        ticker_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        tickers = ticker_values
            .iter()
            .take(10)
            .map(|(t, _)| t.clone())
            .collect();
        tickers.sort();
    }

    if tickers.len() < 2 {
        let msg = if filtered_count > 0 {
            format!(
                "Insufficient data for correlation analysis. Portfolio contains mostly mutual funds ({} filtered out). \
                 Correlation analysis requires at least 2 publicly traded stocks or ETFs with price history.",
                filtered_count
            )
        } else {
            "Need at least 2 equity/ETF positions with price data for correlation analysis.".to_string()
        };
        return Err(AppError::External(msg));
    }

    // 3. Fetch price data for all tickers in one batch query (much faster!)
    let price_data = price_queries::fetch_window_batch(pool, &tickers, days).await?;

    // Filter tickers to only those with sufficient price data (at least 2 points)
    tickers.retain(|t| {
        if let Some(prices) = price_data.get(t) {
            prices.len() >= 2
        } else {
            false
        }
    });

    if tickers.len() < 2 {
        return Err(AppError::External(format!(
            "Insufficient price data for correlation analysis. Only {} position(s) have price history. \
             Correlation requires at least 2 stocks/ETFs with historical price data.",
            tickers.len()
        )));
    }

    // 4. Calculate correlation for each pair (upper triangle only)
    let mut correlations = Vec::new();

    for i in 0..tickers.len() {
        for j in (i + 1)..tickers.len() {
            let ticker1 = &tickers[i];
            let ticker2 = &tickers[j];

            // Get price data - these should exist since we filtered above
            let series1 = match price_data.get(ticker1) {
                Some(s) => s,
                None => continue,
            };
            let series2 = match price_data.get(ticker2) {
                Some(s) => s,
                None => continue,
            };

            if let Some(corr) = risk_service::compute_correlation(series1, series2) {
                correlations.push(CorrelationPair {
                    ticker1: ticker1.clone(),
                    ticker2: ticker2.clone(),
                    correlation: corr,
                });
            }
        }
    }

    if correlations.is_empty() {
        return Err(AppError::External(
            "Failed to compute any correlations".to_string(),
        ));
    }

    // 5. Build 2D matrix for heatmap visualization
    let n = tickers.len();
    let mut matrix_2d = vec![vec![0.0; n]; n];

    // Set diagonal to 1.0 (perfect self-correlation)
    for i in 0..n {
        matrix_2d[i][i] = 1.0;
    }

    // Fill in correlations from pairs
    for pair in &correlations {
        if let (Some(i), Some(j)) = (
            tickers.iter().position(|t| t == &pair.ticker1),
            tickers.iter().position(|t| t == &pair.ticker2),
        ) {
            matrix_2d[i][j] = pair.correlation;
            matrix_2d[j][i] = pair.correlation; // Symmetric
        }
    }

    let matrix = CorrelationMatrix {
        portfolio_id: portfolio_id.to_string(),
        tickers: tickers.clone(),
        correlations,
        matrix_2d,
    };

    // 6. Calculate correlation statistics
    let position_count = tickers.len();
    let statistics = risk_service::calculate_correlation_statistics(&matrix, position_count);

    Ok(CorrelationMatrixWithStats { matrix, statistics })
}

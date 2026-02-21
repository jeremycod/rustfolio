use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::errors::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/reset-all-data", post(reset_all_data))
        .route("/admin/cache-health", get(get_cache_health))
        .route("/admin/jobs", get(list_scheduled_jobs))
        .route("/admin/jobs/recent", get(get_recent_job_runs))
        .route("/admin/jobs/:job_name/history", get(get_job_history))
        .route("/admin/jobs/:job_name/stats", get(get_job_stats))
        .route("/admin/jobs/:job_name/trigger", post(trigger_job_manually))
}

#[derive(Debug, Serialize)]
pub struct ResetResponse {
    pub message: String,
    pub tables_cleared: Vec<String>,
}

pub async fn reset_all_data(
    State(state): State<AppState>,
) -> Result<Json<ResetResponse>, AppError> {
    info!("POST /admin/reset-all-data - Resetting all data");

    let tables = vec![
        "detected_transactions",
        "cash_flows",
        "holdings_snapshots",
        "accounts",
        "price_points",
        "portfolios",
    ];

    for table in &tables {
        let query = format!("DELETE FROM {}", table);
        match sqlx::query(&query).execute(&state.pool).await {
            Ok(result) => {
                info!("Deleted {} rows from {}", result.rows_affected(), table);
            }
            Err(e) => {
                error!("Failed to delete from {}: {}", table, e);
                return Err(AppError::Db(e));
            }
        }
    }

    info!("Successfully reset all data");

    Ok(Json(ResetResponse {
        message: "All data has been successfully deleted".to_string(),
        tables_cleared: tables.iter().map(|s| s.to_string()).collect(),
    }))
}

// ============================================================================
// Cache Health Monitoring Models
// ============================================================================

/// Overall cache health status across all cache tables
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHealthStatus {
    /// Timestamp when health check was performed
    pub checked_at: DateTime<Utc>,
    /// Overall health status
    pub status: CacheHealthLevel,
    /// Individual cache table statistics
    pub tables: Vec<CacheTableHealth>,
    /// Summary statistics
    pub summary: CacheHealthSummary,
}

/// Overall health level
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheHealthLevel {
    Healthy,
    Degraded,
    Critical,
}

/// Health statistics for a specific cache table
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheTableHealth {
    /// Table name
    pub table_name: String,
    /// Total number of cache entries
    pub total_entries: i64,
    /// Number of fresh (valid) entries
    pub fresh_entries: i64,
    /// Number of stale (expired) entries
    pub stale_entries: i64,
    /// Number of entries currently being calculated
    pub calculating_entries: i64,
    /// Number of entries with errors
    pub error_entries: i64,
    /// Hit rate percentage (if available)
    pub hit_rate_pct: Option<f64>,
    /// Average age of cache entries in hours
    pub avg_age_hours: Option<f64>,
}

/// Summary of cache health across all tables
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHealthSummary {
    /// Total cache entries across all tables
    pub total_entries: i64,
    /// Total fresh entries
    pub total_fresh: i64,
    /// Total stale entries
    pub total_stale: i64,
    /// Total calculating entries
    pub total_calculating: i64,
    /// Total error entries
    pub total_errors: i64,
    /// Overall freshness percentage
    pub freshness_pct: f64,
    /// Overall error rate percentage
    pub error_rate_pct: f64,
}

/// Cache status for a specific portfolio
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioCacheStatus {
    /// Portfolio ID
    pub portfolio_id: String,
    /// Timestamp when status was checked
    pub checked_at: DateTime<Utc>,
    /// Risk cache entries for this portfolio
    pub risk_cache: Vec<CacheEntry>,
    /// Correlation cache entries for this portfolio
    pub correlation_cache: Vec<CacheEntry>,
    /// Narrative cache entries for this portfolio
    pub narrative_cache: Vec<CacheEntry>,
}

/// Individual cache entry details
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache entry ID
    pub id: String,
    /// Parameters used (e.g., "90d SPY")
    pub parameters: String,
    /// Calculation status
    pub status: String,
    /// When the cache was calculated
    pub calculated_at: Option<DateTime<Utc>>,
    /// When the cache expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Age in hours
    pub age_hours: Option<f64>,
    /// Time until expiration in hours (negative if expired)
    pub ttl_hours: Option<f64>,
    /// Last error message if any
    pub last_error: Option<String>,
    /// Number of retry attempts
    pub retry_count: Option<i32>,
}

/// Response for cache invalidation
#[derive(Debug, Serialize)]
pub struct InvalidateCacheResponse {
    pub success: bool,
    pub message: String,
    pub portfolio_id: String,
    pub invalidated_at: DateTime<Utc>,
    pub caches_invalidated: Vec<String>,
}

// ============================================================================
// Admin Endpoints
// ============================================================================

/// GET /api/admin/cache-health
///
/// Returns comprehensive health statistics for all cache tables.
/// Provides insights into cache freshness, error rates, and performance.
///
/// This endpoint is useful for:
/// - Monitoring cache health in production
/// - Debugging cache-related issues
/// - Understanding cache utilization patterns
/// - Identifying caches that need attention
///
/// Example response:
/// ```json
/// {
///   "checked_at": "2026-02-20T10:30:00Z",
///   "status": "healthy",
///   "tables": [...],
///   "summary": {
///     "total_entries": 150,
///     "total_fresh": 120,
///     "freshness_pct": 80.0,
///     "error_rate_pct": 2.0
///   }
/// }
/// ```
pub async fn get_cache_health(
    State(state): State<AppState>,
) -> Result<Json<CacheHealthStatus>, AppError> {
    info!("GET /api/admin/cache-health - Checking cache health across all tables");

    let mut tables = Vec::new();

    // Query portfolio_risk_cache
    let risk_cache_health = query_cache_table_health(
        &state.pool,
        "portfolio_risk_cache",
    ).await?;
    tables.push(risk_cache_health);

    // Query portfolio_correlations_cache
    let corr_cache_health = query_cache_table_health(
        &state.pool,
        "portfolio_correlations_cache",
    ).await?;
    tables.push(corr_cache_health);

    // Query portfolio_narrative_cache
    let narrative_cache_health = query_cache_table_health(
        &state.pool,
        "portfolio_narrative_cache",
    ).await?;
    tables.push(narrative_cache_health);

    // Calculate summary statistics
    let total_entries: i64 = tables.iter().map(|t| t.total_entries).sum();
    let total_fresh: i64 = tables.iter().map(|t| t.fresh_entries).sum();
    let total_stale: i64 = tables.iter().map(|t| t.stale_entries).sum();
    let total_calculating: i64 = tables.iter().map(|t| t.calculating_entries).sum();
    let total_errors: i64 = tables.iter().map(|t| t.error_entries).sum();

    let freshness_pct = if total_entries > 0 {
        (total_fresh as f64 / total_entries as f64) * 100.0
    } else {
        100.0
    };

    let error_rate_pct = if total_entries > 0 {
        (total_errors as f64 / total_entries as f64) * 100.0
    } else {
        0.0
    };

    let summary = CacheHealthSummary {
        total_entries,
        total_fresh,
        total_stale,
        total_calculating,
        total_errors,
        freshness_pct,
        error_rate_pct,
    };

    // Determine overall health status
    let status = if error_rate_pct > 10.0 || freshness_pct < 50.0 {
        CacheHealthLevel::Critical
    } else if error_rate_pct > 5.0 || freshness_pct < 80.0 {
        CacheHealthLevel::Degraded
    } else {
        CacheHealthLevel::Healthy
    };

    info!(
        "Cache health check complete: {} - {:.1}% fresh, {:.1}% errors",
        match status {
            CacheHealthLevel::Healthy => "HEALTHY",
            CacheHealthLevel::Degraded => "DEGRADED",
            CacheHealthLevel::Critical => "CRITICAL",
        },
        freshness_pct,
        error_rate_pct
    );

    Ok(Json(CacheHealthStatus {
        checked_at: Utc::now(),
        status,
        tables,
        summary,
    }))
}

/// Query health statistics for a specific cache table
async fn query_cache_table_health(
    pool: &sqlx::PgPool,
    table_name: &str,
) -> Result<CacheTableHealth, AppError> {
    // Determine if table has calculation_status column
    let has_status = table_name == "portfolio_risk_cache"
        || table_name == "portfolio_correlations_cache";

    let stats = if has_status {
        // Query with status breakdown
        let result = sqlx::query_as::<_, (i64, i64, i64, i64, i64, Option<f64>)>(
            &format!(
                r#"
                SELECT
                    COUNT(*) as total,
                    COUNT(*) FILTER (WHERE calculation_status = 'fresh' AND expires_at > NOW()) as fresh,
                    COUNT(*) FILTER (WHERE calculation_status = 'stale' OR (calculation_status = 'fresh' AND expires_at <= NOW())) as stale,
                    COUNT(*) FILTER (WHERE calculation_status = 'calculating') as calculating,
                    COUNT(*) FILTER (WHERE calculation_status = 'error') as error,
                    AVG(EXTRACT(EPOCH FROM (NOW() - calculated_at)) / 3600.0) as avg_age_hours
                FROM {}
                "#,
                table_name
            ),
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)?;

        CacheTableHealth {
            table_name: table_name.to_string(),
            total_entries: result.0,
            fresh_entries: result.1,
            stale_entries: result.2,
            calculating_entries: result.3,
            error_entries: result.4,
            hit_rate_pct: None, // Would require tracking cache hits/misses
            avg_age_hours: result.5,
        }
    } else {
        // Query without status (older tables)
        let result = sqlx::query_as::<_, (i64, i64, i64, Option<f64>)>(
            &format!(
                r#"
                SELECT
                    COUNT(*) as total,
                    COUNT(*) FILTER (WHERE expires_at > NOW()) as fresh,
                    COUNT(*) FILTER (WHERE expires_at <= NOW()) as stale,
                    AVG(EXTRACT(EPOCH FROM (NOW() - calculated_at)) / 3600.0) as avg_age_hours
                FROM {}
                "#,
                table_name
            ),
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)?;

        CacheTableHealth {
            table_name: table_name.to_string(),
            total_entries: result.0,
            fresh_entries: result.1,
            stale_entries: result.2,
            calculating_entries: 0,
            error_entries: 0,
            hit_rate_pct: None,
            avg_age_hours: result.3,
        }
    };

    Ok(stats)
}

/// GET /api/risk/portfolios/:portfolio_id/cache-status
///
/// Returns detailed cache status for a specific portfolio.
/// Shows all cache entries across different cache tables for this portfolio.
///
/// This endpoint helps:
/// - Debug cache issues for a specific portfolio
/// - Understand cache freshness for a portfolio
/// - Identify which caches need refresh
/// - Monitor cache errors for a portfolio
///
/// Example: GET /api/risk/portfolios/{uuid}/cache-status
pub async fn get_portfolio_cache_status(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioCacheStatus>, AppError> {
    info!(
        "GET /api/risk/portfolios/{}/cache-status - Fetching cache status",
        portfolio_id
    );

    // Query risk cache entries
    let risk_cache = query_portfolio_risk_cache(&state.pool, portfolio_id).await?;

    // Query correlation cache entries
    let correlation_cache = query_portfolio_correlation_cache(&state.pool, portfolio_id).await?;

    // Query narrative cache entries
    let narrative_cache = query_portfolio_narrative_cache(&state.pool, portfolio_id).await?;

    info!(
        "Portfolio {} cache status: {} risk, {} correlation, {} narrative entries",
        portfolio_id,
        risk_cache.len(),
        correlation_cache.len(),
        narrative_cache.len()
    );

    Ok(Json(PortfolioCacheStatus {
        portfolio_id: portfolio_id.to_string(),
        checked_at: Utc::now(),
        risk_cache,
        correlation_cache,
        narrative_cache,
    }))
}

/// Query risk cache entries for a portfolio
async fn query_portfolio_risk_cache(
    pool: &sqlx::PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<CacheEntry>, AppError> {
    let entries = sqlx::query_as::<_, (String, i32, String, String, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<String>, Option<i32>)>(
        r#"
        SELECT
            id::TEXT,
            days,
            benchmark,
            COALESCE(calculation_status, 'unknown'),
            calculated_at,
            expires_at,
            last_error,
            retry_count
        FROM portfolio_risk_cache
        WHERE portfolio_id = $1
        ORDER BY days, benchmark
        "#
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    let now = Utc::now();
    Ok(entries
        .into_iter()
        .map(|(id, days, benchmark, status, calculated_at, expires_at, last_error, retry_count)| {
            let age_hours = calculated_at.map(|calc| {
                (now - calc).num_seconds() as f64 / 3600.0
            });
            let ttl_hours = expires_at.map(|exp| {
                (exp - now).num_seconds() as f64 / 3600.0
            });

            CacheEntry {
                id,
                parameters: format!("{}d {}", days, benchmark),
                status,
                calculated_at,
                expires_at,
                age_hours,
                ttl_hours,
                last_error,
                retry_count,
            }
        })
        .collect())
}

/// Query correlation cache entries for a portfolio
async fn query_portfolio_correlation_cache(
    pool: &sqlx::PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<CacheEntry>, AppError> {
    let entries = sqlx::query_as::<_, (String, i32, String, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<String>)>(
        r#"
        SELECT
            id::TEXT,
            days,
            COALESCE(calculation_status, 'unknown'),
            calculated_at,
            expires_at,
            last_error
        FROM portfolio_correlations_cache
        WHERE portfolio_id = $1
        ORDER BY days
        "#
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    let now = Utc::now();
    Ok(entries
        .into_iter()
        .map(|(id, days, status, calculated_at, expires_at, last_error)| {
            let age_hours = calculated_at.map(|calc| {
                (now - calc).num_seconds() as f64 / 3600.0
            });
            let ttl_hours = expires_at.map(|exp| {
                (exp - now).num_seconds() as f64 / 3600.0
            });

            CacheEntry {
                id,
                parameters: format!("{}d", days),
                status,
                calculated_at,
                expires_at,
                age_hours,
                ttl_hours,
                last_error,
                retry_count: None,
            }
        })
        .collect())
}

/// Query narrative cache entries for a portfolio
async fn query_portfolio_narrative_cache(
    pool: &sqlx::PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<CacheEntry>, AppError> {
    let entries = sqlx::query_as::<_, (String, String, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
        r#"
        SELECT
            id::TEXT,
            time_period,
            generated_at,
            expires_at
        FROM portfolio_narrative_cache
        WHERE portfolio_id = $1
        ORDER BY time_period
        "#
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    let now = Utc::now();
    Ok(entries
        .into_iter()
        .map(|(id, time_period, generated_at, expires_at)| {
            let age_hours = generated_at.map(|gen| {
                (now - gen).num_seconds() as f64 / 3600.0
            });
            let ttl_hours = expires_at.map(|exp| {
                (exp - now).num_seconds() as f64 / 3600.0
            });
            let status = if let Some(exp) = expires_at {
                if exp > now {
                    "fresh".to_string()
                } else {
                    "stale".to_string()
                }
            } else {
                "unknown".to_string()
            };

            CacheEntry {
                id,
                parameters: time_period,
                status,
                calculated_at: generated_at,
                expires_at,
                age_hours,
                ttl_hours,
                last_error: None,
                retry_count: None,
            }
        })
        .collect())
}

/// POST /api/risk/portfolios/:portfolio_id/invalidate-cache
///
/// Manually invalidates all cache entries for a specific portfolio.
/// This forces fresh recalculation on the next request.
///
/// Use cases:
/// - After manual data corrections
/// - When cache appears stale or incorrect
/// - For testing cache refresh behavior
/// - After significant portfolio changes
///
/// Example: POST /api/risk/portfolios/{uuid}/invalidate-cache
pub async fn invalidate_cache(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<InvalidateCacheResponse>, AppError> {
    info!(
        "POST /api/risk/portfolios/{}/invalidate-cache - Invalidating all caches",
        portfolio_id
    );

    let mut caches_invalidated = Vec::new();

    // Mark all risk cache entries as stale
    let risk_count = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE portfolio_risk_cache
        SET calculation_status = 'stale',
            expires_at = NOW(),
            updated_at = NOW()
        WHERE portfolio_id = $1
        RETURNING 1
        "#
    )
    .bind(portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Db)?
    .len();

    if risk_count > 0 {
        caches_invalidated.push(format!("portfolio_risk_cache ({} entries)", risk_count));
    }

    // Mark all correlation cache entries as stale
    let corr_count = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE portfolio_correlations_cache
        SET calculation_status = 'stale',
            expires_at = NOW(),
            updated_at = NOW()
        WHERE portfolio_id = $1
        RETURNING 1
        "#
    )
    .bind(portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Db)?
    .len();

    if corr_count > 0 {
        caches_invalidated.push(format!("portfolio_correlations_cache ({} entries)", corr_count));
    }

    // Mark all narrative cache entries as expired
    let narrative_count = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE portfolio_narrative_cache
        SET expires_at = NOW(),
            updated_at = NOW()
        WHERE portfolio_id = $1
        RETURNING 1
        "#
    )
    .bind(portfolio_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Db)?
    .len();

    if narrative_count > 0 {
        caches_invalidated.push(format!("portfolio_narrative_cache ({} entries)", narrative_count));
    }

    let total_invalidated = risk_count + corr_count + narrative_count;

    if total_invalidated == 0 {
        warn!(
            "No cache entries found for portfolio {} to invalidate",
            portfolio_id
        );
    } else {
        info!(
            "Successfully invalidated {} cache entries for portfolio {}",
            total_invalidated, portfolio_id
        );
    }

    Ok(Json(InvalidateCacheResponse {
        success: true,
        message: format!("Successfully invalidated {} cache entries", total_invalidated),
        portfolio_id: portfolio_id.to_string(),
        invalidated_at: Utc::now(),
        caches_invalidated,
    }))
}

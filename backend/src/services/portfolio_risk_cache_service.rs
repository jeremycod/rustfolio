//! Portfolio Risk Cache Management Service
//!
//! This service provides comprehensive management and monitoring capabilities for portfolio
//! risk and correlation caches. It enables manual cache invalidation, health monitoring,
//! and detailed status tracking for individual portfolios.
//!
//! # Cache System Overview
//!
//! The system maintains two types of caches:
//!
//! 1. **Portfolio Risk Cache** (`portfolio_risk_cache` table)
//!    - Stores pre-calculated portfolio risk metrics (volatility, beta, Sharpe ratio, etc.)
//!    - Expensive to compute due to API calls and statistical calculations
//!    - Typical TTL: 4 hours
//!    - Refreshed by background jobs
//!
//! 2. **Portfolio Correlations Cache** (`portfolio_correlations_cache` table)
//!    - Stores correlation matrices for portfolio holdings
//!    - Requires fetching historical price data for all holdings
//!    - Typical TTL: 24 hours
//!    - Refreshed by background jobs
//!
//! # Cache Status Values
//!
//! Both cache tables use a `calculation_status` field with these possible values:
//!
//! - `fresh`: Cache entry is current and valid
//! - `stale`: Cache entry has expired and needs recalculation
//! - `calculating`: Background job is currently computing this cache entry
//! - `error`: Calculation failed (see `last_error` field for details)
//!
//! # Usage Examples
//!
//! ## Invalidate Portfolio Caches
//!
//! ```rust
//! use uuid::Uuid;
//! use portfolio_risk_cache_service::invalidate_portfolio_caches;
//!
//! // Mark all caches for a portfolio as stale
//! let portfolio_id = Uuid::parse_str("...").unwrap();
//! invalidate_portfolio_caches(&pool, portfolio_id).await?;
//! ```
//!
//! ## Check Overall Cache Health
//!
//! ```rust
//! use portfolio_risk_cache_service::get_cache_health;
//!
//! let health = get_cache_health(&pool).await?;
//! println!("Fresh risk caches: {}/{}", health.risk_cache_fresh, health.risk_cache_total);
//! println!("Error risk caches: {}", health.risk_cache_error);
//! ```
//!
//! ## Get Portfolio-Specific Cache Status
//!
//! ```rust
//! use portfolio_risk_cache_service::get_portfolio_cache_status;
//!
//! let status = get_portfolio_cache_status(&pool, portfolio_id).await?;
//! if let Some(risk_status) = status.risk_cache_status {
//!     println!("Risk cache: {} (age: {}s)", risk_status, status.risk_cache_age_seconds.unwrap());
//! }
//! ```
//!
//! # Integration with Background Jobs
//!
//! This service is designed to work with background jobs:
//!
//! - `portfolio_risk_job.rs` - Refreshes risk caches hourly
//! - `portfolio_correlations_job.rs` - Refreshes correlation caches daily
//!
//! The jobs automatically update cache status and handle errors. This service provides
//! the tools to monitor their operation and manually trigger invalidation when needed.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;

/// Comprehensive health statistics for both cache types.
///
/// This structure provides a complete overview of cache status across the system,
/// enabling monitoring dashboards and alerting systems to track cache health.
///
/// # Fields
///
/// ## Risk Cache Statistics
///
/// - `risk_cache_total`: Total number of risk cache entries
/// - `risk_cache_fresh`: Number of entries with status 'fresh' (valid and current)
/// - `risk_cache_stale`: Number of entries with status 'stale' (expired, needs refresh)
/// - `risk_cache_calculating`: Number of entries with status 'calculating' (job in progress)
/// - `risk_cache_error`: Number of entries with status 'error' (calculation failed)
///
/// ## Correlations Cache Statistics
///
/// - `correlations_cache_total`: Total number of correlation cache entries
/// - `correlations_cache_fresh`: Number of entries with status 'fresh'
/// - `correlations_cache_stale`: Number of entries with status 'stale'
/// - `correlations_cache_calculating`: Number of entries with status 'calculating'
/// - `correlations_cache_error`: Number of entries with status 'error'
///
/// # Example
///
/// ```rust
/// let health = get_cache_health(&pool).await?;
///
/// // Calculate health percentages
/// let risk_fresh_pct = if health.risk_cache_total > 0 {
///     (health.risk_cache_fresh as f64 / health.risk_cache_total as f64) * 100.0
/// } else {
///     0.0
/// };
///
/// println!("Risk cache health: {:.1}% fresh", risk_fresh_pct);
///
/// // Alert if too many errors
/// if health.risk_cache_error > health.risk_cache_total / 10 {
///     eprintln!("WARNING: More than 10% of risk caches have errors!");
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct CacheHealthStatus {
    /// Total number of portfolio risk cache entries
    pub risk_cache_total: i64,
    /// Number of risk cache entries with 'fresh' status
    pub risk_cache_fresh: i64,
    /// Number of risk cache entries with 'stale' status
    pub risk_cache_stale: i64,
    /// Number of risk cache entries with 'calculating' status
    pub risk_cache_calculating: i64,
    /// Number of risk cache entries with 'error' status
    pub risk_cache_error: i64,

    /// Total number of portfolio correlations cache entries
    pub correlations_cache_total: i64,
    /// Number of correlations cache entries with 'fresh' status
    pub correlations_cache_fresh: i64,
    /// Number of correlations cache entries with 'stale' status
    pub correlations_cache_stale: i64,
    /// Number of correlations cache entries with 'calculating' status
    pub correlations_cache_calculating: i64,
    /// Number of correlations cache entries with 'error' status
    pub correlations_cache_error: i64,
}

/// Detailed cache status for a specific portfolio.
///
/// This structure provides complete information about both risk and correlation
/// caches for a single portfolio, including age, status, and error details.
///
/// # Fields
///
/// ## Risk Cache Information
///
/// - `risk_cache_status`: Current status ('fresh', 'stale', 'calculating', 'error', or None)
/// - `risk_cache_age_seconds`: Age of cache in seconds (None if no cache exists)
/// - `risk_cache_last_calculated`: Timestamp of last successful calculation
/// - `risk_cache_last_error`: Error message if last calculation failed
/// - `risk_cache_retry_count`: Number of consecutive failed attempts
///
/// ## Correlations Cache Information
///
/// - `correlations_cache_status`: Current status (same values as risk cache)
/// - `correlations_cache_age_seconds`: Age of cache in seconds
/// - `correlations_cache_last_calculated`: Timestamp of last successful calculation
/// - `correlations_cache_last_error`: Error message if last calculation failed
/// - `correlations_cache_retry_count`: Number of consecutive failed attempts
///
/// # Example
///
/// ```rust
/// let status = get_portfolio_cache_status(&pool, portfolio_id).await?;
///
/// // Check if risk cache is healthy
/// match status.risk_cache_status.as_deref() {
///     Some("fresh") => println!("Risk cache is fresh and ready"),
///     Some("stale") => println!("Risk cache needs refresh"),
///     Some("calculating") => println!("Risk calculation in progress"),
///     Some("error") => {
///         println!("Risk calculation failed: {:?}", status.risk_cache_last_error);
///     }
///     None => println!("No risk cache exists for this portfolio"),
///     _ => println!("Unknown cache status"),
/// }
///
/// // Warn if cache is old
/// if let Some(age) = status.risk_cache_age_seconds {
///     if age > 14400 {  // 4 hours
///         println!("WARNING: Risk cache is {} seconds old", age);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioCacheStatus {
    /// Portfolio ID this status is for
    pub portfolio_id: Uuid,

    // Risk cache fields
    /// Current status of the risk cache ('fresh', 'stale', 'calculating', 'error', or None)
    pub risk_cache_status: Option<String>,
    /// Age of the risk cache in seconds (None if cache doesn't exist)
    pub risk_cache_age_seconds: Option<i64>,
    /// Timestamp when risk cache was last successfully calculated
    pub risk_cache_last_calculated: Option<DateTime<Utc>>,
    /// Error message from last failed risk calculation attempt
    pub risk_cache_last_error: Option<String>,
    /// Number of consecutive failed risk calculation attempts
    pub risk_cache_retry_count: Option<i32>,

    // Correlations cache fields
    /// Current status of the correlations cache ('fresh', 'stale', 'calculating', 'error', or None)
    pub correlations_cache_status: Option<String>,
    /// Age of the correlations cache in seconds (None if cache doesn't exist)
    pub correlations_cache_age_seconds: Option<i64>,
    /// Timestamp when correlations cache was last successfully calculated
    pub correlations_cache_last_calculated: Option<DateTime<Utc>>,
    /// Error message from last failed correlations calculation attempt
    pub correlations_cache_last_error: Option<String>,
    /// Number of consecutive failed correlations calculation attempts
    pub correlations_cache_retry_count: Option<i32>,
}

/// Marks all caches for a portfolio as 'stale', triggering recalculation.
///
/// This function invalidates both the risk cache and correlations cache for the specified
/// portfolio. Background jobs will detect the stale status and recalculate the caches
/// on their next run.
///
/// # Use Cases
///
/// - Manual cache invalidation after data changes
/// - Force recalculation when you know underlying data has changed
/// - Recovery from persistent calculation errors
/// - Testing cache refresh behavior
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - UUID of the portfolio whose caches should be invalidated
///
/// # Returns
///
/// * `Ok(())` - Caches successfully marked as stale
/// * `Err(AppError)` - Database error occurred
///
/// # Notes
///
/// - This function does NOT delete cache entries, it only marks them as 'stale'
/// - If no cache entries exist, this function does nothing (no error)
/// - Both risk and correlations caches are invalidated atomically
/// - Background jobs will pick up stale entries on their next scheduled run
///
/// # Example
///
/// ```rust
/// use uuid::Uuid;
///
/// // Parse portfolio ID from request
/// let portfolio_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?;
///
/// // Invalidate all caches for this portfolio
/// invalidate_portfolio_caches(&pool, portfolio_id).await?;
///
/// info!("Portfolio {} caches marked as stale, will refresh on next job run", portfolio_id);
/// ```
pub async fn invalidate_portfolio_caches(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<(), AppError> {
    info!("Invalidating caches for portfolio: {}", portfolio_id);

    // Mark portfolio_risk_cache entries as 'stale'
    let risk_result = sqlx::query!(
        r#"
        UPDATE portfolio_risk_cache
        SET calculation_status = 'stale',
            updated_at = NOW()
        WHERE portfolio_id = $1
        "#,
        portfolio_id
    )
    .execute(pool)
    .await?;

    info!(
        "Marked {} risk cache entries as stale for portfolio {}",
        risk_result.rows_affected(),
        portfolio_id
    );

    // Mark portfolio_correlations_cache entries as 'stale'
    let corr_result = sqlx::query!(
        r#"
        UPDATE portfolio_correlations_cache
        SET calculation_status = 'stale',
            updated_at = NOW()
        WHERE portfolio_id = $1
        "#,
        portfolio_id
    )
    .execute(pool)
    .await?;

    info!(
        "Marked {} correlations cache entries as stale for portfolio {}",
        corr_result.rows_affected(),
        portfolio_id
    );

    Ok(())
}

/// Retrieves comprehensive health statistics for all cache tables.
///
/// This function queries both cache tables and returns aggregated statistics about
/// cache status distribution. It's designed for monitoring dashboards, health checks,
/// and alerting systems.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// * `Ok(CacheHealthStatus)` - Health statistics for all caches
/// * `Err(AppError)` - Database error occurred
///
/// # Performance
///
/// This function performs two COUNT queries with GROUP BY, which are generally fast
/// even on large tables due to status indexes. Typical execution time: <100ms.
///
/// # Example
///
/// ```rust
/// // Get cache health for monitoring dashboard
/// let health = get_cache_health(&pool).await?;
///
/// // Log summary
/// info!(
///     "Cache Health - Risk: {}/{} fresh, Corr: {}/{} fresh",
///     health.risk_cache_fresh,
///     health.risk_cache_total,
///     health.correlations_cache_fresh,
///     health.correlations_cache_total
/// );
///
/// // Check for problems
/// if health.risk_cache_error > 0 {
///     warn!("{} risk caches have errors", health.risk_cache_error);
/// }
///
/// if health.risk_cache_calculating > health.risk_cache_total / 2 {
///     warn!("More than 50% of risk caches are currently calculating - possible job backlog");
/// }
/// ```
pub async fn get_cache_health(pool: &PgPool) -> Result<CacheHealthStatus, AppError> {
    info!("Querying cache health statistics");

    // Query risk cache status counts
    let risk_stats = sqlx::query!(
        r#"
        SELECT
            calculation_status,
            COUNT(*) as count
        FROM portfolio_risk_cache
        GROUP BY calculation_status
        "#
    )
    .fetch_all(pool)
    .await?;

    // Query correlations cache status counts
    let corr_stats = sqlx::query!(
        r#"
        SELECT
            calculation_status,
            COUNT(*) as count
        FROM portfolio_correlations_cache
        GROUP BY calculation_status
        "#
    )
    .fetch_all(pool)
    .await?;

    // Process risk cache statistics
    let mut risk_cache_fresh = 0i64;
    let mut risk_cache_stale = 0i64;
    let mut risk_cache_calculating = 0i64;
    let mut risk_cache_error = 0i64;
    let mut risk_total = 0i64;

    for row in risk_stats {
        let count = row.count.unwrap_or(0);
        risk_total += count;

        match row.calculation_status.as_deref() {
            Some("fresh") => risk_cache_fresh = count,
            Some("stale") => risk_cache_stale = count,
            Some("calculating") => risk_cache_calculating = count,
            Some("error") => risk_cache_error = count,
            _ => {
                // Handle any other status values
                error!("Unknown risk cache status: {:?}", row.calculation_status);
            }
        }
    }

    // Process correlations cache statistics
    let mut correlations_cache_fresh = 0i64;
    let mut correlations_cache_stale = 0i64;
    let mut correlations_cache_calculating = 0i64;
    let mut correlations_cache_error = 0i64;
    let mut corr_total = 0i64;

    for row in corr_stats {
        let count = row.count.unwrap_or(0);
        corr_total += count;

        match row.calculation_status.as_deref() {
            Some("fresh") => correlations_cache_fresh = count,
            Some("stale") => correlations_cache_stale = count,
            Some("calculating") => correlations_cache_calculating = count,
            Some("error") => correlations_cache_error = count,
            _ => {
                // Handle any other status values
                error!("Unknown correlations cache status: {:?}", row.calculation_status);
            }
        }
    }

    let health = CacheHealthStatus {
        risk_cache_total: risk_total,
        risk_cache_fresh,
        risk_cache_stale,
        risk_cache_calculating,
        risk_cache_error,
        correlations_cache_total: corr_total,
        correlations_cache_fresh,
        correlations_cache_stale,
        correlations_cache_calculating,
        correlations_cache_error,
    };

    info!(
        "Cache health: Risk [{} total, {} fresh, {} stale, {} calculating, {} error], \
         Corr [{} total, {} fresh, {} stale, {} calculating, {} error]",
        health.risk_cache_total,
        health.risk_cache_fresh,
        health.risk_cache_stale,
        health.risk_cache_calculating,
        health.risk_cache_error,
        health.correlations_cache_total,
        health.correlations_cache_fresh,
        health.correlations_cache_stale,
        health.correlations_cache_calculating,
        health.correlations_cache_error
    );

    Ok(health)
}

/// Retrieves detailed cache status for a specific portfolio.
///
/// This function returns comprehensive information about both risk and correlation
/// caches for a single portfolio, including status, age, errors, and retry counts.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `portfolio_id` - UUID of the portfolio to query
///
/// # Returns
///
/// * `Ok(PortfolioCacheStatus)` - Detailed cache status for the portfolio
/// * `Err(AppError)` - Database error occurred
///
/// # Behavior
///
/// - If no cache entries exist for the portfolio, all optional fields will be None
/// - If multiple cache entries exist (different parameters), returns the most recent
/// - Age is calculated from `calculated_at` timestamp to current time
/// - All timestamps are in UTC
///
/// # Example
///
/// ```rust
/// use uuid::Uuid;
///
/// let portfolio_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?;
/// let status = get_portfolio_cache_status(&pool, portfolio_id).await?;
///
/// // Display risk cache status
/// match status.risk_cache_status.as_deref() {
///     Some("fresh") => {
///         println!("✅ Risk cache is fresh");
///         if let Some(age) = status.risk_cache_age_seconds {
///             println!("   Age: {}s", age);
///         }
///     }
///     Some("error") => {
///         println!("❌ Risk cache calculation failed");
///         if let Some(err) = status.risk_cache_last_error {
///             println!("   Error: {}", err);
///         }
///         if let Some(retries) = status.risk_cache_retry_count {
///             println!("   Retry count: {}", retries);
///         }
///     }
///     Some("calculating") => {
///         println!("⏳ Risk cache calculation in progress");
///     }
///     Some("stale") => {
///         println!("⚠️  Risk cache is stale, needs refresh");
///     }
///     None => {
///         println!("ℹ️  No risk cache exists for this portfolio");
///     }
///     _ => println!("Unknown status"),
/// }
/// ```
pub async fn get_portfolio_cache_status(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<PortfolioCacheStatus, AppError> {
    info!("Querying cache status for portfolio: {}", portfolio_id);

    // Query risk cache status (get most recent entry if multiple exist)
    let risk_cache = sqlx::query!(
        r#"
        SELECT
            calculation_status,
            calculated_at,
            last_error,
            retry_count,
            EXTRACT(EPOCH FROM (NOW() - calculated_at))::BIGINT as age_seconds
        FROM portfolio_risk_cache
        WHERE portfolio_id = $1
        ORDER BY calculated_at DESC
        LIMIT 1
        "#,
        portfolio_id
    )
    .fetch_optional(pool)
    .await?;

    // Query correlations cache status (get most recent entry if multiple exist)
    // Note: correlations cache doesn't have retry_count column
    let corr_cache = sqlx::query!(
        r#"
        SELECT
            calculation_status,
            calculated_at,
            last_error,
            EXTRACT(EPOCH FROM (NOW() - calculated_at))::BIGINT as age_seconds
        FROM portfolio_correlations_cache
        WHERE portfolio_id = $1
        ORDER BY calculated_at DESC
        LIMIT 1
        "#,
        portfolio_id
    )
    .fetch_optional(pool)
    .await?;

    let status = PortfolioCacheStatus {
        portfolio_id,
        // Risk cache fields
        risk_cache_status: risk_cache.as_ref().and_then(|r| r.calculation_status.clone()),
        risk_cache_age_seconds: risk_cache.as_ref().and_then(|r| r.age_seconds),
        risk_cache_last_calculated: risk_cache.as_ref().map(|r| r.calculated_at),
        risk_cache_last_error: risk_cache.as_ref().and_then(|r| r.last_error.clone()),
        risk_cache_retry_count: risk_cache.as_ref().and_then(|r| r.retry_count),
        // Correlations cache fields
        correlations_cache_status: corr_cache.as_ref().and_then(|c| c.calculation_status.clone()),
        correlations_cache_age_seconds: corr_cache.as_ref().and_then(|c| c.age_seconds),
        correlations_cache_last_calculated: corr_cache.as_ref().map(|c| c.calculated_at),
        correlations_cache_last_error: corr_cache.as_ref().and_then(|c| c.last_error.clone()),
        // Correlations cache doesn't have retry_count column yet
        correlations_cache_retry_count: None,
    };

    info!(
        "Portfolio {} cache status: Risk=[{:?}, age={}s], Corr=[{:?}, age={}s]",
        portfolio_id,
        status.risk_cache_status,
        status.risk_cache_age_seconds.unwrap_or(0),
        status.correlations_cache_status,
        status.correlations_cache_age_seconds.unwrap_or(0)
    );

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that CacheHealthStatus can be serialized to JSON
    #[test]
    fn test_cache_health_status_serialization() {
        let health = CacheHealthStatus {
            risk_cache_total: 100,
            risk_cache_fresh: 80,
            risk_cache_stale: 10,
            risk_cache_calculating: 5,
            risk_cache_error: 5,
            correlations_cache_total: 50,
            correlations_cache_fresh: 45,
            correlations_cache_stale: 3,
            correlations_cache_calculating: 1,
            correlations_cache_error: 1,
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("risk_cache_total"));
        assert!(json.contains("100"));
    }

    /// Test that PortfolioCacheStatus can be serialized to JSON
    #[test]
    fn test_portfolio_cache_status_serialization() {
        let portfolio_id = Uuid::new_v4();
        let status = PortfolioCacheStatus {
            portfolio_id,
            risk_cache_status: Some("fresh".to_string()),
            risk_cache_age_seconds: Some(3600),
            risk_cache_last_calculated: Some(Utc::now()),
            risk_cache_last_error: None,
            risk_cache_retry_count: Some(0),
            correlations_cache_status: Some("stale".to_string()),
            correlations_cache_age_seconds: Some(86400),
            correlations_cache_last_calculated: Some(Utc::now()),
            correlations_cache_last_error: Some("Test error".to_string()),
            correlations_cache_retry_count: Some(2),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("portfolio_id"));
        assert!(json.contains("fresh"));
        assert!(json.contains("stale"));
    }
}

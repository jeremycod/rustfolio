//! Daily Risk Snapshots Background Job
//!
//! This job runs daily at 5:00 PM ET (17:00) to create historical risk snapshots
//! for all portfolios in the system. By capturing daily snapshots, we build a
//! historical record of portfolio risk metrics that enables trend analysis,
//! alerting, and performance tracking over time.
//!
//! # Job Schedule
//!
//! - **Production**: Daily at 5:00 PM ET (0 0 17 * * *)
//! - **Test Mode**: Every 5 minutes (0 */5 * * * *)
//!
//! # Purpose and Distinction
//!
//! This job is separate from the `portfolio_risk_job` which runs hourly:
//!
//! - **portfolio_risk_job**: Writes to `portfolio_risk_cache` (ephemeral cache)
//!   - Purpose: Fast API responses for current risk metrics
//!   - Frequency: Hourly at :15
//!   - Data lifetime: 4 hours
//!
//! - **daily_risk_snapshots_job**: Writes to `risk_snapshots` (persistent storage)
//!   - Purpose: Historical tracking and trend analysis
//!   - Frequency: Daily at 5:00 PM
//!   - Data lifetime: 1 year (archived weekly)
//!
//! # Data Created
//!
//! For each portfolio, this job creates:
//! 1. Portfolio-level snapshot: Aggregated risk metrics for the entire portfolio
//! 2. Position-level snapshots: Individual risk metrics for each holding
//!
//! These snapshots are stored in the `risk_snapshots` table with:
//! - Volatility, max drawdown, beta, Sharpe ratio
//! - Risk score and risk level classification
//! - Market values and weights
//! - Snapshot date for historical tracking
//!
//! # Processing Strategy
//!
//! 1. Query all portfolios with active holdings
//! 2. For each portfolio:
//!    - Call `risk_snapshot_service::create_daily_snapshots()`
//!    - This creates both portfolio-level and position-level snapshots
//!    - Snapshots are upserted into `risk_snapshots` table
//!    - Errors for individual positions don't stop portfolio processing
//! 3. Track success/failure counts for monitoring
//! 4. Add 1-second delay between portfolios to respect rate limits
//!
//! # Error Handling
//!
//! - Individual portfolio failures don't stop the entire job
//! - Portfolios with no holdings are skipped with a warning
//! - Position-level failures within a portfolio are handled gracefully
//! - Detailed logging helps with debugging and monitoring
//! - Proper error propagation to job tracking system
//!
//! # Performance Considerations
//!
//! - Runs after market close to capture end-of-day prices
//! - Uses existing price data (no additional fetching needed)
//! - Implements delays between portfolios to respect rate limits
//! - Leverages existing risk calculation infrastructure
//! - Designed for idempotent execution (can be safely re-run)

use crate::errors::AppError;
use crate::services::{job_scheduler_service::{JobContext, JobResult}, risk_snapshot_service};
use chrono::Utc;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Standard parameters for risk snapshot calculations
const INTER_PORTFOLIO_DELAY_MS: u64 = 1000;

/// Main entry point for the daily risk snapshots job.
///
/// This function is called by the job scheduler at 5:00 PM ET daily.
/// It processes all portfolios in the system, creating persistent historical
/// risk snapshots for both portfolio-level and position-level metrics.
///
/// # Arguments
///
/// * `ctx` - Job context containing database pool and external services
///
/// # Returns
///
/// * `Ok(JobResult)` - Success with counts of processed and failed portfolios
/// * `Err(AppError)` - Critical failure that should stop the job
pub async fn create_all_daily_risk_snapshots(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📸 Starting daily risk snapshots job");

    // Get today's date for snapshots
    let today = Utc::now().date_naive();
    info!("Creating snapshots for date: {}", today);

    // Query all portfolios with holdings
    let portfolios = query_portfolios_with_holdings(&ctx.pool).await?;

    if portfolios.is_empty() {
        info!("No portfolios with holdings found, nothing to process");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} portfolios to snapshot", portfolios.len());

    let mut processed = 0;
    let mut failed = 0;

    // Get risk free rate from environment or use default
    let risk_free_rate = std::env::var("RISK_FREE_RATE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.045); // Default 4.5%

    // Process each portfolio
    for portfolio_id in portfolios {
        info!("Creating snapshots for portfolio {}...", portfolio_id);

        // Create daily snapshots for this portfolio
        match risk_snapshot_service::create_daily_snapshots(
            &ctx.pool,
            portfolio_id,
            today,
            ctx.price_provider.as_ref(),
            ctx.failure_cache.as_ref(),
            ctx.rate_limiter.as_ref(),
            risk_free_rate,
        )
        .await
        {
            Ok(snapshots) => {
                let snapshot_count = snapshots.len();
                info!(
                    "✅ Created {} snapshots for portfolio {} (1 portfolio + {} positions)",
                    snapshot_count,
                    portfolio_id,
                    snapshot_count.saturating_sub(1)
                );
                processed += 1;
            }
            Err(e) => {
                // Check if error is due to no holdings (expected case, not a failure)
                let error_str = e.to_string();
                if error_str.contains("No holdings found") {
                    warn!(
                        "⚠️  Portfolio {} has no holdings, skipping snapshot",
                        portfolio_id
                    );
                    // Count as processed since this is not an error condition
                    processed += 1;
                } else {
                    error!(
                        "❌ Failed to create snapshots for portfolio {}: {}",
                        portfolio_id, e
                    );
                    failed += 1;
                }
            }
        }

        // Add delay between portfolios to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(
            INTER_PORTFOLIO_DELAY_MS,
        ))
        .await;
    }

    info!(
        "✅ Daily risk snapshots job completed: {} portfolios processed, {} failed",
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
/// ordered by portfolio ID for consistent processing order. This ensures we
/// only attempt to create snapshots for portfolios that have actual positions.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// * `Ok(Vec<Uuid>)` - List of portfolio IDs with holdings
/// * `Err(AppError)` - Database query error
async fn query_portfolios_with_holdings(pool: &PgPool) -> Result<Vec<Uuid>, AppError> {
    let portfolio_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT DISTINCT p.id
        FROM portfolios p
        INNER JOIN accounts a ON a.portfolio_id = p.id
        INNER JOIN holding_snapshots hs ON hs.account_id = a.id
        ORDER BY p.id
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(portfolio_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(INTER_PORTFOLIO_DELAY_MS, 1000);
    }
}

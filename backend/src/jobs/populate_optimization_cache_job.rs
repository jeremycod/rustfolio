//! Portfolio Optimization Cache Population Background Job
//!
//! This job runs regularly to pre-calculate and cache portfolio optimization
//! recommendations. By pre-caching optimization results, we ensure fast response
//! times for optimization queries and reduce computation overhead.
//!
//! # Job Schedule
//!
//! - **Production**: Every 6 hours (0 0 */6 * * *)
//! - **Test Mode**: Every 15 minutes (0 */15 * * * *)
//!
//! # Processing Strategy
//!
//! 1. Query all portfolios with active holdings
//! 2. For each portfolio, check if cache is expired or missing
//! 3. Calculate optimization recommendations using optimization_service
//! 4. Store results in portfolio_optimization_cache table
//! 5. Add delays between portfolios to avoid overloading the system
//!
//! # Error Handling
//!
//! - Individual portfolio failures don't stop the entire job
//! - Errors are logged with portfolio context
//! - Failed portfolios are tracked and reported in job results
//!
//! # Performance Considerations
//!
//! - Skips portfolios with fresh cache (< 6 hours old)
//! - Implements delays to avoid overwhelming price APIs
//! - Processes portfolios sequentially for stability

use crate::errors::AppError;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::optimization_service;
use chrono::Utc;
use tracing::{error, info};
use uuid::Uuid;

const CACHE_EXPIRATION_HOURS: i64 = 6;
const INTER_PORTFOLIO_DELAY_MS: u64 = 1000; // 1 second delay between portfolios

/// Main entry point for the optimization cache population job.
///
/// This function is called by the job scheduler to populate optimization
/// cache for all active portfolios.
///
/// # Arguments
///
/// * `ctx` - Job context containing database pool and external services
///
/// # Returns
///
/// * `Ok(JobResult)` - Success with counts of processed and failed portfolios
/// * `Err(AppError)` - Critical error that prevents job execution
pub async fn populate_all_optimization_caches(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🎯 Starting optimization cache population job");

    // 1. Get all portfolios with active holdings
    let portfolios = get_active_portfolios(ctx.pool.as_ref()).await?;

    if portfolios.is_empty() {
        info!("No active portfolios found - nothing to process");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} portfolios to process", portfolios.len());

    let mut processed = 0;
    let mut failed = 0;

    // 2. Process each portfolio
    for (idx, portfolio_id) in portfolios.iter().enumerate() {
        info!(
            "Processing portfolio {}/{}: {}",
            idx + 1,
            portfolios.len(),
            portfolio_id
        );

        // Check if cache is fresh
        if is_cache_fresh(ctx.pool.as_ref(), portfolio_id).await? {
            info!("Cache for portfolio {} is fresh, skipping", portfolio_id);
            processed += 1;
            continue;
        }

        // Calculate and cache optimization for this portfolio
        let start = std::time::Instant::now();
        match calculate_and_cache_optimization(
            &ctx,
            portfolio_id,
        )
        .await
        {
            Ok(_) => {
                let duration = start.elapsed();
                info!(
                    "✅ Successfully cached optimization for portfolio {} in {:?}",
                    portfolio_id, duration
                );
                processed += 1;
            }
            Err(e) => {
                error!(
                    "❌ Failed to cache optimization for portfolio {}: {}",
                    portfolio_id, e
                );
                failed += 1;
            }
        }

        // Add delay between portfolios
        if idx < portfolios.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(INTER_PORTFOLIO_DELAY_MS)).await;
        }
    }

    info!(
        "✅ Optimization cache population complete: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Get all portfolios with active holdings
async fn get_active_portfolios(pool: &sqlx::PgPool) -> Result<Vec<Uuid>, AppError> {
    let portfolios = sqlx::query!(
        r#"
        SELECT DISTINCT p.id
        FROM portfolios p
        JOIN accounts a ON a.portfolio_id = p.id
        JOIN holdings_snapshots hs ON hs.account_id = a.id
        WHERE hs.quantity > 0
        ORDER BY p.id
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(portfolios.into_iter().map(|r| r.id).collect())
}

/// Check if cache for a portfolio is still fresh
async fn is_cache_fresh(pool: &sqlx::PgPool, portfolio_id: &Uuid) -> Result<bool, AppError> {
    let cache_exists = sqlx::query!(
        r#"
        SELECT 1 as "exists!"
        FROM portfolio_optimization_cache
        WHERE portfolio_id = $1
          AND expires_at > NOW()
        "#,
        portfolio_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(cache_exists.is_some())
}

/// Public function to manually calculate optimization for a single portfolio
/// (used by API endpoint for on-demand generation)
pub async fn calculate_single_portfolio_optimization(
    ctx: &JobContext,
    portfolio_id: Uuid,
) -> Result<(), AppError> {
    info!("📊 Manually calculating optimization for portfolio {}", portfolio_id);
    calculate_and_cache_optimization(ctx, &portfolio_id).await
}

/// Calculate optimization and store in cache
async fn calculate_and_cache_optimization(
    ctx: &JobContext,
    portfolio_id: &Uuid,
) -> Result<(), AppError> {
    let start = std::time::Instant::now();

    // Calculate optimization using optimization service
    let analysis = optimization_service::analyze_portfolio(
        ctx.pool.as_ref(),
        *portfolio_id,
        ctx.price_provider.as_ref(),
        &ctx.failure_cache,
        &ctx.rate_limiter,
        0.04, // risk_free_rate = 4%
    )
    .await?;

    let duration_ms = start.elapsed().as_millis() as i32;

    // Calculate expiration time (6 hours from now)
    let expires_at = Utc::now() + chrono::Duration::hours(CACHE_EXPIRATION_HOURS);

    // Serialize recommendations to JSON
    let recommendations_json = serde_json::to_value(&analysis.recommendations)?;

    let risk_free_rate = 0.04; // 4% risk-free rate

    // Store in cache
    sqlx::query!(
        r#"
        INSERT INTO portfolio_optimization_cache (
            portfolio_id,
            calculated_at,
            expires_at,
            recommendations,
            risk_free_rate,
            positions_analyzed,
            calculation_duration_ms
        )
        VALUES ($1, NOW(), $2, $3, $4, $5, $6)
        ON CONFLICT (portfolio_id) DO UPDATE SET
            calculated_at = NOW(),
            expires_at = $2,
            recommendations = $3,
            risk_free_rate = $4,
            positions_analyzed = $5,
            calculation_duration_ms = $6
        "#,
        portfolio_id,
        expires_at.naive_utc(),
        recommendations_json,
        risk_free_rate,
        analysis.recommendations.len() as i32,
        duration_ms
    )
    .execute(ctx.pool.as_ref())
    .await?;

    info!(
        "Cached {} recommendations for portfolio {} ({}ms)",
        analysis.recommendations.len(),
        portfolio_id,
        duration_ms
    );

    Ok(())
}

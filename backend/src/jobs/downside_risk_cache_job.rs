//! Downside Risk Cache Population Background Job
//!
//! This job pre-calculates and caches downside risk metrics (CVaR, Sortino ratio,
//! downside deviation) for all portfolios. These are computationally expensive
//! calculations that should not be done on every request.
//!
//! # Job Schedule
//!
//! - **Production**: Every 6 hours (0 0 */6 * * *)
//! - Aligns with portfolio risk job schedule
//!
//! # Processing Strategy
//!
//! 1. Query all active portfolios (with positions)
//! 2. For each portfolio, check if cache is expired or missing
//! 3. Calculate downside risk metrics using risk_service
//! 4. Store results in downside_risk_cache table
//! 5. Use delays to avoid overwhelming external APIs

use crate::errors::AppError;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::risk_service;
use chrono::{Duration, Utc};
use serde_json;
use tracing::{info, warn};
use uuid::Uuid;

const CACHE_EXPIRATION_HOURS: i64 = 6; // 6-hour cache TTL
const INTER_PORTFOLIO_DELAY_MS: u64 = 2000; // 2 second delay between portfolios
const COMPUTATION_TIMEOUT_SECONDS: u64 = 300; // 5 minute timeout for downside risk computation

/// Main entry point for downside risk cache population
pub async fn populate_downside_risk_caches(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔄 [DOWNSIDE_RISK_JOB] Starting downside risk cache population job");

    // Get all portfolios with positions
    info!("🔍 [DOWNSIDE_RISK_JOB] Querying portfolios with positions...");
    let portfolios = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT portfolio_id FROM positions ORDER BY portfolio_id"
    )
    .fetch_all(ctx.pool.as_ref())
    .await?;

    if portfolios.is_empty() {
        info!("⚠️ [DOWNSIDE_RISK_JOB] No portfolios found to cache");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("✅ [DOWNSIDE_RISK_JOB] Found {} portfolios to process", portfolios.len());

    let mut processed = 0;
    let mut failed = 0;

    // Standard parameters
    let days = 90;
    let benchmark = "SPY";

    for (index, portfolio_id) in portfolios.iter().enumerate() {
        info!("📊 [DOWNSIDE_RISK_JOB] Processing portfolio {}/{}: {}", index + 1, portfolios.len(), portfolio_id);

        // Check if cache needs refresh
        info!("🔍 [DOWNSIDE_RISK_JOB] Checking cache status for portfolio {}", portfolio_id);
        let needs_refresh = check_cache_needs_refresh(
            ctx.pool.as_ref(),
            *portfolio_id,
            days,
            benchmark,
        )
        .await?;

        if !needs_refresh {
            info!("⏭️ [DOWNSIDE_RISK_JOB] Cache for portfolio {} is still fresh, skipping", portfolio_id);
            processed += 1;
            continue;
        }

        info!("🚀 [DOWNSIDE_RISK_JOB] Starting computation for portfolio {} (timeout: {}s)", portfolio_id, COMPUTATION_TIMEOUT_SECONDS);
        let start_time = std::time::Instant::now();

        // Compute and cache downside risk with timeout
        let computation_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(COMPUTATION_TIMEOUT_SECONDS),
            compute_and_cache_downside_risk(
                &ctx,
                *portfolio_id,
                days,
                benchmark,
            )
        )
        .await;

        let elapsed = start_time.elapsed();

        match computation_result {
            Ok(Ok(_)) => {
                processed += 1;
                info!("✅ [DOWNSIDE_RISK_JOB] Successfully cached downside risk for portfolio {} (took {:.2}s)", portfolio_id, elapsed.as_secs_f64());
            }
            Ok(Err(e)) => {
                failed += 1;
                warn!("❌ [DOWNSIDE_RISK_JOB] Failed to cache downside risk for portfolio {}: {} (took {:.2}s)", portfolio_id, e, elapsed.as_secs_f64());
            }
            Err(_) => {
                failed += 1;
                warn!("⏱️ [DOWNSIDE_RISK_JOB] Downside risk computation TIMED OUT after {} seconds for portfolio {}", COMPUTATION_TIMEOUT_SECONDS, portfolio_id);
            }
        }

        // Delay to avoid rate limiting external APIs
        info!("⏸️ [DOWNSIDE_RISK_JOB] Waiting {}ms before next portfolio...", INTER_PORTFOLIO_DELAY_MS);
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_PORTFOLIO_DELAY_MS)).await;
    }

    info!(
        "🏁 [DOWNSIDE_RISK_JOB] Downside risk cache population COMPLETED: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Check if cache needs refresh
async fn check_cache_needs_refresh(
    pool: &sqlx::PgPool,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query_scalar::<_, chrono::NaiveDateTime>(
        "SELECT expires_at FROM downside_risk_cache
         WHERE portfolio_id = $1 AND days = $2 AND benchmark = $3"
    )
    .bind(portfolio_id)
    .bind(days as i32)
    .bind(benchmark)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(expires_at) => {
            use chrono::TimeZone;
            let expires_at_utc = Utc.from_utc_datetime(&expires_at);
            let needs_refresh = expires_at_utc < Utc::now();
            Ok(needs_refresh)
        }
        None => Ok(true), // No cache exists
    }
}

/// Compute downside risk and store in cache
async fn compute_and_cache_downside_risk(
    ctx: &JobContext,
    portfolio_id: Uuid,
    days: i64,
    benchmark: &str,
) -> Result<(), AppError> {
    info!("🧮 [COMPUTE_DOWNSIDE] Starting computation for portfolio {} (days={}, benchmark={})", portfolio_id, days, benchmark);

    // Compute downside risk analysis
    let compute_start = std::time::Instant::now();
    let downside_risk = risk_service::compute_portfolio_downside_risk(
        ctx.pool.as_ref(),
        portfolio_id,
        days,
        benchmark,
        ctx.price_provider.as_ref(),
        ctx.failure_cache.as_ref(),
        ctx.rate_limiter.as_ref(),
        0.04, // Default risk-free rate 4%
    )
    .await?;
    let compute_elapsed = compute_start.elapsed();
    info!("✅ [COMPUTE_DOWNSIDE] Computation completed for portfolio {} in {:.2}s", portfolio_id, compute_elapsed.as_secs_f64());

    info!("📝 [COMPUTE_DOWNSIDE] Serializing risk data to JSONB for portfolio {}", portfolio_id);
    // Serialize to JSONB
    let risk_data = serde_json::to_value(&downside_risk)
        .map_err(|e| AppError::External(format!("Failed to serialize risk data: {}", e)))?;

    let expires_at = (Utc::now() + Duration::hours(CACHE_EXPIRATION_HOURS)).naive_utc();

    info!("💾 [COMPUTE_DOWNSIDE] Storing in cache for portfolio {} (expires in {} hours)", portfolio_id, CACHE_EXPIRATION_HOURS);
    // Upsert into cache
    let db_start = std::time::Instant::now();
    sqlx::query!(
        r#"
        INSERT INTO downside_risk_cache (
            portfolio_id, days, benchmark,
            risk_data, calculated_at, expires_at
        )
        VALUES ($1, $2, $3, $4, NOW(), $5)
        ON CONFLICT (portfolio_id, days, benchmark)
        DO UPDATE SET
            risk_data = EXCLUDED.risk_data,
            calculated_at = NOW(),
            expires_at = EXCLUDED.expires_at
        "#,
        portfolio_id,
        days as i32,
        benchmark,
        risk_data,
        expires_at
    )
    .execute(ctx.pool.as_ref())
    .await?;
    let db_elapsed = db_start.elapsed();
    info!("✅ [COMPUTE_DOWNSIDE] Cached successfully for portfolio {} (DB write took {:.2}s)", portfolio_id, db_elapsed.as_secs_f64());

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_job_compiles() {
        // Ensures job compiles correctly
    }
}

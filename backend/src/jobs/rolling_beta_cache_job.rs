//! Rolling Beta Cache Population Background Job
//!
//! This job runs regularly to pre-calculate and cache rolling beta analysis
//! for all active positions. By pre-caching results, we ensure fast response
//! times for rolling beta queries.
//!
//! # Job Schedule
//!
//! - **Production**: Every 6 hours (0 0 */6 * * *)
//! - **Test Mode**: Every 30 minutes (0 */30 * * * *)
//!
//! # Processing Strategy
//!
//! 1. Query all unique tickers from active positions
//! 2. For each ticker, check if cache is expired or missing
//! 3. Calculate rolling beta analysis using risk_service
//! 4. Store results in rolling_beta_cache table
//! 5. Add delays between tickers to avoid overloading the system

use crate::errors::AppError;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::risk_service;
use chrono::{Duration, Utc};
use serde_json::json;
use tracing::{info, warn};

const CACHE_EXPIRATION_HOURS: i64 = 24; // 24-hour cache TTL
const INTER_TICKER_DELAY_MS: u64 = 1000; // 1 second delay between tickers

/// Main entry point for the rolling beta cache population job.
pub async fn populate_rolling_beta_caches(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔄 Populating rolling beta caches...");

    // Get all unique tickers from positions
    let tickers = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT ticker FROM positions ORDER BY ticker"
    )
    .fetch_all(ctx.pool.as_ref())
    .await?;

    if tickers.is_empty() {
        info!("No tickers found to cache");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} tickers to process", tickers.len());

    let mut processed = 0;
    let mut failed = 0;

    // Standard parameters for rolling beta
    let days = 180;
    let benchmark = "SPY";

    for ticker in tickers {
        // Check if cache exists and is still fresh
        let needs_refresh = check_cache_needs_refresh(
            ctx.pool.as_ref(),
            &ticker,
            benchmark,
            days,
        )
        .await?;

        if !needs_refresh {
            info!("Cache for {} is still fresh, skipping", ticker);
            processed += 1;
            continue;
        }

        // Compute and cache rolling beta
        match compute_and_cache_rolling_beta(
            &ctx,
            &ticker,
            benchmark,
            days,
        )
        .await
        {
            Ok(_) => {
                processed += 1;
                info!("✅ Cached rolling beta for {}", ticker);
            }
            Err(e) => {
                failed += 1;
                warn!("❌ Failed to cache rolling beta for {}: {}", ticker, e);
            }
        }

        // Delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_TICKER_DELAY_MS)).await;
    }

    info!(
        "Rolling beta cache population completed: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Check if cache needs refresh (missing or expired)
async fn check_cache_needs_refresh(
    pool: &sqlx::PgPool,
    ticker: &str,
    benchmark: &str,
    days: i64,
) -> Result<bool, AppError> {
    let result = sqlx::query_scalar::<_, chrono::NaiveDateTime>(
        "SELECT expires_at FROM rolling_beta_cache
         WHERE ticker = $1 AND benchmark = $2 AND total_days = $3"
    )
    .bind(ticker)
    .bind(benchmark)
    .bind(days as i32)
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

/// Compute rolling beta and store in cache
async fn compute_and_cache_rolling_beta(
    ctx: &JobContext,
    ticker: &str,
    benchmark: &str,
    days: i64,
) -> Result<(), AppError> {
    // Compute rolling beta analysis
    let analysis = risk_service::compute_rolling_beta(
        ctx.pool.as_ref(),
        ticker,
        benchmark,
        days,
        ctx.price_provider.as_ref(),
        ctx.failure_cache.as_ref(),
    )
    .await?;

    // Serialize beta time series to JSONB
    let beta_30d = json!(analysis.beta_30d);
    let beta_60d = json!(analysis.beta_60d);
    let beta_90d = json!(analysis.beta_90d);

    let expires_at = (Utc::now() + Duration::hours(CACHE_EXPIRATION_HOURS)).naive_utc();

    // Upsert into cache
    sqlx::query!(
        r#"
        INSERT INTO rolling_beta_cache (
            ticker, benchmark, total_days,
            beta_30d, beta_60d, beta_90d,
            current_beta, beta_volatility,
            calculated_at, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
        ON CONFLICT (ticker, benchmark, total_days)
        DO UPDATE SET
            beta_30d = EXCLUDED.beta_30d,
            beta_60d = EXCLUDED.beta_60d,
            beta_90d = EXCLUDED.beta_90d,
            current_beta = EXCLUDED.current_beta,
            beta_volatility = EXCLUDED.beta_volatility,
            calculated_at = NOW(),
            expires_at = EXCLUDED.expires_at
        "#,
        ticker,
        benchmark,
        days as i32,
        beta_30d,
        beta_60d,
        beta_90d,
        analysis.current_beta,
        analysis.beta_volatility,
        expires_at
    )
    .execute(ctx.pool.as_ref())
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_job_compiles() {
        // Ensures job compiles correctly
    }
}

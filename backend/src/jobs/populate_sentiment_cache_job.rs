//! Sentiment Cache Population Background Job
//!
//! This job runs regularly to pre-calculate and cache sentiment signals for all
//! active tickers in user portfolios. By pre-caching sentiment data, we ensure
//! fast response times for portfolio sentiment queries and reduce load on external
//! news APIs.
//!
//! # Job Schedule
//!
//! - **Production**: Every 4 hours (0 0 */4 * * *)
//! - **Test Mode**: Every 10 minutes (0 */10 * * * *)
//!
//! # Processing Strategy
//!
//! 1. Query all unique tickers from active portfolio holdings
//! 2. For each ticker, check if cache is expired or missing
//! 3. Fetch sentiment data from news API and calculate signals
//! 4. Store results in sentiment_signal_cache table
//! 5. Add delays between tickers to respect rate limits
//!
//! # Error Handling
//!
//! - Individual ticker failures don't stop the entire job
//! - Errors are logged with ticker context
//! - Failed tickers are tracked and reported in job results
//!
//! # Performance Considerations
//!
//! - Skips tickers with fresh cache (< 4 hours old)
//! - Implements delays to respect news API rate limits
//! - Processes tickers sequentially to avoid overwhelming external services

use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::services::failure_cache::FailureCache;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::rate_limiter::RateLimiter;
use crate::services::sentiment_service;
use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info};

const CACHE_EXPIRATION_HOURS: i64 = 4;
const INTER_TICKER_DELAY_MS: u64 = 500; // 500ms delay between tickers
const SENTIMENT_LOOKBACK_DAYS: usize = 30;

/// Main entry point for the sentiment cache population job.
///
/// This function is called by the job scheduler to populate sentiment
/// cache for all active portfolio tickers.
///
/// # Arguments
///
/// * `ctx` - Job context containing database pool and external services
///
/// # Returns
///
/// * `Ok(JobResult)` - Success with counts of processed and failed tickers
/// * `Err(AppError)` - Critical error that prevents job execution
pub async fn populate_all_sentiment_caches(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📊 Starting sentiment cache population job");

    // 1. Get all unique tickers from active portfolio holdings
    let tickers = get_active_portfolio_tickers(ctx.pool.as_ref()).await?;

    if tickers.is_empty() {
        info!("No active portfolio tickers found - nothing to process");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Found {} unique tickers to process", tickers.len());

    let mut processed = 0;
    let mut failed = 0;

    // 2. Process each ticker
    for (idx, ticker) in tickers.iter().enumerate() {
        info!(
            "Processing ticker {}/{}: {}",
            idx + 1,
            tickers.len(),
            ticker
        );

        // Check if cache is fresh
        if is_cache_fresh(ctx.pool.as_ref(), ticker).await? {
            info!("Cache for {} is fresh, skipping", ticker);
            processed += 1;
            continue;
        }

        // Fetch and cache sentiment for this ticker
        match fetch_and_cache_sentiment(
            ctx.pool.clone(),
            ctx.price_provider.clone(),
            ctx.failure_cache.clone(),
            ctx.rate_limiter.clone(),
            ticker,
        )
        .await
        {
            Ok(_) => {
                info!("✅ Successfully cached sentiment for {}", ticker);
                processed += 1;
            }
            Err(e) => {
                error!("❌ Failed to cache sentiment for {}: {}", ticker, e);
                failed += 1;
            }
        }

        // Add delay between tickers to respect rate limits
        if idx < tickers.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(INTER_TICKER_DELAY_MS)).await;
        }
    }

    info!(
        "✅ Sentiment cache population complete: {} processed, {} failed",
        processed, failed
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

/// Get all unique tickers from active portfolio holdings
async fn get_active_portfolio_tickers(pool: &sqlx::PgPool) -> Result<Vec<String>, AppError> {
    let tickers = sqlx::query!(
        r#"
        SELECT DISTINCT hs.ticker
        FROM holdings_snapshots hs
        JOIN accounts a ON hs.account_id = a.id
        WHERE hs.quantity > 0
          AND a.portfolio_id IS NOT NULL
        ORDER BY hs.ticker
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(tickers.into_iter().map(|r| r.ticker).collect())
}

/// Check if cache for a ticker is still fresh
async fn is_cache_fresh(pool: &sqlx::PgPool, ticker: &str) -> Result<bool, AppError> {
    let cache_age = sqlx::query!(
        r#"
        SELECT
            EXTRACT(EPOCH FROM (NOW() - calculated_at)) / 3600 as "age_hours"
        FROM sentiment_signal_cache
        WHERE ticker = $1
          AND expires_at > NOW()
        "#,
        ticker
    )
    .fetch_optional(pool)
    .await?;

    Ok(cache_age
        .map(|r| r.age_hours.unwrap_or(999.0) < CACHE_EXPIRATION_HOURS as f64)
        .unwrap_or(false))
}

/// Fetch sentiment data and store in cache
async fn fetch_and_cache_sentiment(
    pool: Arc<sqlx::PgPool>,
    price_provider: Arc<dyn PriceProvider>,
    failure_cache: Arc<FailureCache>,
    rate_limiter: Arc<RateLimiter>,
    ticker: &str,
) -> Result<(), AppError> {
    // Fetch sentiment signal using sentiment service
    let signal = sentiment_service::generate_sentiment_signal(
        pool.as_ref(),
        ticker,
        SENTIMENT_LOOKBACK_DAYS,
        price_provider.as_ref(),
        &failure_cache,
        &rate_limiter,
    )
    .await?;

    // Calculate expiration time (4 hours from now)
    let expires_at = Utc::now() + chrono::Duration::hours(CACHE_EXPIRATION_HOURS);

    // Serialize historical data to JSON
    let historical_json = serde_json::to_value(&signal.historical_sentiment)?;
    let warnings_json = serde_json::to_value(&signal.warnings)?;

    // Store in cache
    sqlx::query!(
        r#"
        INSERT INTO sentiment_signal_cache (
            ticker,
            calculated_at,
            expires_at,
            current_sentiment,
            sentiment_trend,
            momentum_trend,
            divergence,
            sentiment_price_correlation,
            correlation_lag_days,
            historical_sentiment,
            news_articles_analyzed,
            warnings
        )
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (ticker) DO UPDATE SET
            calculated_at = NOW(),
            expires_at = $2,
            current_sentiment = $3,
            sentiment_trend = $4,
            momentum_trend = $5,
            divergence = $6,
            sentiment_price_correlation = $7,
            correlation_lag_days = $8,
            historical_sentiment = $9,
            news_articles_analyzed = $10,
            warnings = $11
        "#,
        ticker,
        expires_at,
        signal.current_sentiment,
        format!("{:?}", signal.sentiment_trend),
        format!("{:?}", signal.momentum_trend),
        format!("{:?}", signal.divergence),
        signal.sentiment_price_correlation,
        signal.correlation_lag_days,
        historical_json,
        signal.news_articles_analyzed as i32,
        warnings_json
    )
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

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
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::sentiment_service;
use crate::services::news_service::NewsService;
use crate::services::llm_service::LlmService;
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
            ctx.news_service.clone(),
            ctx.llm_service.clone(),
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
    use bigdecimal::ToPrimitive;

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
        .and_then(|r| r.age_hours)
        .and_then(|age| age.to_f64())
        .map(|age| age < CACHE_EXPIRATION_HOURS as f64)
        .unwrap_or(false))
}

/// Fetch sentiment data and store in cache
async fn fetch_and_cache_sentiment(
    pool: Arc<sqlx::PgPool>,
    news_service: Arc<NewsService>,
    _llm_service: Arc<LlmService>, // Used by news_service.cluster_into_themes internally
    ticker: &str,
) -> Result<(), AppError> {
    use uuid::Uuid;

    // 1. Fetch news articles for the ticker
    let articles = news_service.fetch_ticker_news(ticker, SENTIMENT_LOOKBACK_DAYS as i32).await?;

    if articles.is_empty() {
        return Err(AppError::Validation(
            format!("No news data available for {}", ticker)
        ));
    }

    info!("Fetched {} news articles for {}", articles.len(), ticker);

    // 2. Cluster articles into themes using LLM
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| AppError::External(format!("Failed to parse demo user UUID: {}", e)))?;

    let themes = news_service.cluster_into_themes(articles, demo_user_id).await?;

    if themes.is_empty() {
        return Err(AppError::Validation(
            format!("Failed to extract themes from news for {}", ticker)
        ));
    }

    info!("Extracted {} themes for {}", themes.len(), ticker);

    // 3. Fetch price history for correlation analysis
    let prices = crate::services::price_service::get_history(pool.as_ref(), ticker).await?;

    if prices.is_empty() {
        return Err(AppError::Validation(
            format!("No price history available for {}", ticker)
        ));
    }

    info!("Fetched {} price points for {}", prices.len(), ticker);

    // 4. Generate sentiment signal
    let signal = sentiment_service::generate_sentiment_signal(
        pool.as_ref(),
        ticker,
        themes,
        prices,
    )
    .await?;

    info!("Generated sentiment signal for {}: score={:.2}", ticker, signal.current_sentiment);

    // Note: The signal is already cached by generate_sentiment_signal() at sentiment_service.rs:341
    // No need to cache again here

    Ok(())
}

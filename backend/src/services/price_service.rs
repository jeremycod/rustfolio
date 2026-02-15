
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use tracing::{error, warn, info};
use tokio::time::{sleep as async_sleep, Duration};
use crate::db;
use crate::errors::AppError;
use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use crate::models::PricePoint;
use crate::services::failure_cache::{FailureCache, FailureType};
use chrono::{Utc, Duration as ChronoDuration};

pub async fn get_history(pool: &PgPool, ticker: &str)
                         -> Result<Vec<PricePoint>, AppError> {
    db::price_queries::fetch_all(pool, ticker).await
        .map_err(|e| {
            error!("Failed to fetch price history for ticker {}: {}", ticker, e);
            AppError::Db(e)
        })
}

pub async fn get_latest(pool: &PgPool, ticker: &str)
                        -> Result<PricePoint, AppError> {
    db::price_queries::fetch_latest(pool, ticker)
        .await
        .map_err(|e| {
            error!("Failed to fetch latest price for ticker {}: {}", ticker, e);
            AppError::Db(e)
        })?
        .ok_or_else(|| {
            error!("No price data found for ticker {}", ticker);
            AppError::NotFound(format!("No price data found for ticker {}", ticker))
        })
}

/*pub async fn refresh_from_api(pool: &PgPool, ticker: &str)
                              -> Result<(), AppError> {
    let api_prices = external::price_provider::fetch_daily(ticker).await?;
    db::price_queries::insert_many(pool, &api_prices).await?;
    Ok(())
}*/

pub async fn generate_mock(pool: &PgPool, ticker: &str) -> Result<(), AppError> {
    let today = Utc::now().date_naive();
    let mut points: Vec<ExternalPricePoint> = Vec::new();

    let mut current = 100.0_f64;

    for i in 0..180 {
        current *= 1.0 + (rand::random::<f64>() - 0.5) * 0.02;

        points.push(ExternalPricePoint {
            date: today - ChronoDuration::days(i),
            close: current.to_string().parse::<BigDecimal>().unwrap(),
        });
    }

    db::price_queries::upsert_external_points(pool, ticker, &points).await
        .map_err(|e| {
            error!("Failed to generate mock prices for ticker {}: {}", ticker, e);
            AppError::Db(e)
        })?;
    Ok(())
}

pub async fn search_for_ticker_from_api(
    provider: &dyn PriceProvider,
    keyword: &str) -> Result<Vec<ExternalTickerMatch>, AppError> {
    match provider.search_ticker_by_keyword(keyword).await {
        Ok(matches) => Ok(matches),
        Err(PriceProviderError::RateLimited) => Err(AppError::RateLimited),
        Err(e) => {
            Err(AppError::External(e.to_string()))
        },
    }
}
pub async fn refresh_from_api(
    pool: &PgPool,
    provider: &dyn PriceProvider,
    ticker: &str,
    failure_cache: &FailureCache,
) -> Result<(), AppError> {
    // Check database failure cache first - avoid repeated calls for known-bad tickers
    let should_retry = db::ticker_fetch_failure_queries::should_retry_ticker(pool, ticker)
        .await
        .map_err(|e| {
            error!("Failed to check failure cache for ticker {}: {}", ticker, e);
            AppError::Db(e)
        })?;

    if !should_retry {
        if let Ok(Some(failure)) = db::ticker_fetch_failure_queries::get_active_failure(pool, ticker).await {
            info!("⚠️ Skipping API call for {} - ticker is in failure cache ({}). Will retry after {}",
                  ticker,
                  failure.failure_type,
                  failure.retry_after);
            return Err(AppError::External(
                format!("Ticker {} is in failure cache ({}). Will retry after {}",
                    ticker, failure.failure_type, failure.retry_after)
            ));
        }
    }

    // Check if we already have recent data in database
    if let Some(latest) = db::price_queries::fetch_latest(pool, ticker).await? {
        let today = Utc::now().date_naive();

        // If we already have recent data, skip fetching to avoid rate limits
        if latest.date >= today - ChronoDuration::hours(6) {
            info!("✓ Skipping API call for {} - data is recent ({})", ticker, latest.date);
            return Ok(());
        }
    }

    // Retry logic with exponential backoff
    let mut retry_count = 0;
    let max_retries = 3;

    loop {
        match provider.fetch_daily_history(ticker, 60).await {
            Ok(external_points) => {
                db::price_queries::upsert_external_points(pool, ticker, &external_points).await
                    .map_err(|e| {
                        error!("Failed to refresh prices from API for ticker {}: {}", ticker, e);
                        AppError::Db(e)
                    })?;

                // Clear from failure cache on success
                failure_cache.clear(ticker);
                if let Err(e) = db::ticker_fetch_failure_queries::clear_fetch_failure(pool, ticker).await {
                    warn!("Failed to clear failure cache for ticker {}: {}", ticker, e);
                }

                info!("✓ Successfully fetched price data for {}", ticker);
                return Ok(());
            },
            Err(PriceProviderError::RateLimited) if retry_count < max_retries => {
                retry_count += 1;
                let delay = Duration::from_secs(5 * retry_count as u64); // 5, 10, 15 seconds
                warn!("Rate limited for ticker {}, retrying in {}s (attempt {}/{})",
                      ticker, delay.as_secs(), retry_count, max_retries);
                async_sleep(delay).await;
            },
            Err(e) => {
                // Record failure in both memory and database cache to avoid retrying
                let failure_type_str = match &e {
                    PriceProviderError::RateLimited => "rate_limited",
                    PriceProviderError::NotFound => "not_found",
                    _ => "api_error",
                };

                let failure_type_mem = match &e {
                    PriceProviderError::RateLimited => FailureType::RateLimited,
                    PriceProviderError::NotFound => FailureType::NotFound,
                    _ => FailureType::ApiError,
                };

                failure_cache.record_failure(ticker, failure_type_mem);

                if let Err(db_err) = db::ticker_fetch_failure_queries::record_fetch_failure(
                    pool,
                    ticker,
                    failure_type_str,
                    Some(&e.to_string())
                ).await {
                    error!("Failed to record failure in database for ticker {}: {}", ticker, db_err);
                }

                error!("✗ Failed to fetch price data for {}: {}", ticker, e);
                return Err(match e {
                    PriceProviderError::RateLimited => AppError::RateLimited,
                    _ => AppError::External(e.to_string()),
                });
            }
        }
    }
}

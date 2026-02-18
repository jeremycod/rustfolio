
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use tracing::{error, warn, info};
use tokio::time::{sleep as async_sleep, Duration};
use crate::db;
use crate::errors::AppError;
use crate::external::price_provider::{ExternalPricePoint, ExternalTickerMatch, PriceProvider, PriceProviderError};
use crate::models::PricePoint;
use crate::services::failure_cache::{FailureCache, FailureType};
use chrono::{Utc, Duration as ChronoDuration, Datelike, Timelike};

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

/// Determines if we should refresh price data based on market hours and data age
///
/// Strategy:
/// - Weekends: Don't refresh (markets closed)
/// - Before market open (< 9:30 AM ET): Use data from previous day
/// - During market hours (9:30 AM - 4:00 PM ET): Refresh if older than 15 minutes
/// - After market close (> 4:00 PM ET): Don't refresh until next day
fn should_refresh_price_data(latest_price_date: chrono::NaiveDate) -> bool {
    use chrono::Weekday;

    let now = Utc::now();
    let today = now.date_naive();

    // Convert to Eastern Time (approximate - 5 hours behind UTC, or 4 during DST)
    // For simplicity, we'll use a fixed offset. In production, consider using chrono_tz.
    let et_hour = if now.hour() >= 5 { now.hour() - 5 } else { now.hour() + 19 };

    // Weekend: No need to refresh
    if matches!(now.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }

    // If data is from today, check based on market hours
    if latest_price_date == today {
        // During market hours (9:30 AM - 4:00 PM ET ≈ 14:30 - 21:00 UTC)
        if et_hour >= 9 && et_hour < 16 {
            // Refresh every 15 minutes during market hours
            // Since we can't track exact time, we'll refresh on each request during market hours
            // but the rate limiter will prevent too many concurrent requests
            return true;
        }
        // Before or after market hours with today's data: don't refresh
        return false;
    }

    // If data is from yesterday and it's before market open, that's recent enough
    if latest_price_date == today - ChronoDuration::days(1) && et_hour < 9 {
        return false;
    }

    // Data is old (> 1 day or > 1 business day): refresh
    true
}

/// Validates whether a ticker symbol is valid for API calls
/// Returns false for empty strings, non-alphabetic symbols, and known mutual fund codes
fn is_valid_ticker(ticker: &str) -> bool {
    let ticker = ticker.trim();

    // Must not be empty
    if ticker.is_empty() {
        return false;
    }

    // Must contain at least one letter
    if !ticker.chars().any(|c| c.is_alphabetic()) {
        return false;
    }

    // Skip known mutual fund codes that won't be found in stock APIs
    // These are internal fund identifiers used by Canadian fund providers
    let mutual_fund_prefixes = [
        "FID",  // Fidelity funds
        "DYN",  // Dynamic Funds
        "EDG",  // Edge funds
        "BIP",  // Brookfield funds
        "LYZ",  // Lysander funds
        "RBF",  // RBC funds
        "AGF",  // AGF funds
        "MFC",  // Manulife funds
        "RPD",  // Fund code
        "MMF",  // Fund code
        "NWT",  // Fund code
    ];

    if mutual_fund_prefixes.iter().any(|prefix| ticker.starts_with(prefix)) {
        return false;
    }

    true
}

pub async fn refresh_from_api(
    pool: &PgPool,
    provider: &dyn PriceProvider,
    ticker: &str,
    failure_cache: &FailureCache,
    rate_limiter: &crate::services::rate_limiter::RateLimiter,
) -> Result<(), AppError> {
    // Validate ticker before attempting any API calls
    if !is_valid_ticker(ticker) {
        info!("⊘ Skipping invalid ticker: '{}' (empty, non-alphabetic, or mutual fund code)", ticker);
        return Err(AppError::External(format!(
            "Invalid ticker symbol: '{}'. This appears to be empty, malformed, or a mutual fund code that requires manual pricing.",
            ticker
        )));
    }

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

    // Check if we already have recent data in database (smart caching)
    if let Some(latest) = db::price_queries::fetch_latest(pool, ticker).await? {
        if !should_refresh_price_data(latest.date) {
            info!("✓ Skipping API call for {} - data is recent enough ({})", ticker, latest.date);
            return Ok(());
        }
    }

    // Retry logic with exponential backoff
    let mut retry_count = 0;
    let max_retries = 3;

    loop {
        // Acquire rate limiter permit before making API call
        let _guard = rate_limiter.acquire().await;

        // Fetch 365 days of history to support rolling beta analysis (needs 180 days + 90-day window)
        match provider.fetch_daily_history(ticker, 365).await {
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

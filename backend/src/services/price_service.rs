use std::thread::sleep;

use sqlx::PgPool;
use tracing::{error, warn};
use tokio::time::{sleep as async_sleep, Duration};
use crate::{db, external};
use crate::errors::AppError;
use crate::external::price_provider::{ExternalPricePoint, PriceProvider, PriceProviderError};
use crate::models::PricePoint;
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
            AppError::NotFound
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
            close: current,
        });
    }

    db::price_queries::upsert_external_points(pool, ticker, &points).await
        .map_err(|e| {
            error!("Failed to generate mock prices for ticker {}: {}", ticker, e);
            AppError::Db(e)
        })?;
    Ok(())
}
pub async fn refresh_from_api(
    pool: &PgPool,
    provider: &dyn PriceProvider,
    ticker: &str,
) -> Result<(), AppError> {
    if let Some(latest) = db::price_queries::fetch_latest(pool, ticker).await? {
        let today = Utc::now().date_naive();

        // If we already have recent data, skip fetching to avoid rate limits
        if latest.date >= today - ChronoDuration::hours(6) {
            warn!("Skipping API call for {} - data is recent ({})", ticker, latest.date);
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
                return Err(match e {
                    PriceProviderError::RateLimited => AppError::RateLimited,
                    _ => AppError::External(e.to_string()),
                });
            }
        }
    }
}

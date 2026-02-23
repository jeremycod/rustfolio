//! Regime Forecast Generation Job
//!
//! This job runs daily to generate HMM-based regime forecasts for multiple horizons.
//! It uses the latest trained HMM model to predict market regimes 5, 10, and 30 days ahead.
//!
//! # Job Schedule
//!
//! - **Production**: Daily at 5:30 PM ET (0 30 17 * * *)
//! - After market close and regime detection
//!
//! # Process
//!
//! 1. Load the latest HMM model
//! 2. Fetch recent market observations (returns + volatility)
//! 3. Generate forecasts for 5, 10, and 30 day horizons
//! 4. Save forecasts to regime_forecasts table
//! 5. Clean up old forecasts (keep last 90 days)

use chrono::Utc;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::hmm_queries;
use crate::errors::AppError;
use crate::external::alphavantage::AlphaVantageProvider;
use crate::external::price_provider::PriceProvider;
use crate::services::hmm_inference_service;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::models::{ObservationSymbol, PricePoint};

/// Generate regime forecasts for multiple horizons
///
/// This is the main entry point called by the job scheduler
pub async fn generate_all_regime_forecasts(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔮 Starting regime forecast generation job");
    let start_time = std::time::Instant::now();

    match run_forecast_generation(&ctx.pool).await {
        Ok(forecast_count) => {
            let elapsed = start_time.elapsed();
            info!(
                "✅ Generated {} regime forecasts in {:.2}s",
                forecast_count,
                elapsed.as_secs_f64()
            );
            Ok(JobResult {
                items_processed: forecast_count as i32,
                items_failed: 0,
            })
        }
        Err(e) => {
            error!("❌ Regime forecast generation failed: {}", e);
            Err(e)
        }
    }
}

/// Core forecast generation logic
async fn run_forecast_generation(pool: &PgPool) -> Result<usize, AppError> {
    // 1. Check if HMM model exists
    match hmm_queries::load_latest_hmm_model(pool, "SPY").await {
        Ok(model) => {
            info!("Using HMM model: {} (trained on {})", model.model_name, model.trained_on_date);
        }
        Err(_) => {
            warn!("No HMM model found - forecasts cannot be generated yet");
            return Err(AppError::External(
                "HMM model not trained yet. Run HMM training job first.".to_string()
            ));
        }
    }

    // 2. Fetch recent market data for observations
    let price_provider = AlphaVantageProvider::from_env()
        .map_err(|e| AppError::External(format!("Failed to init price provider: {}", e)))?;

    let recent_prices = match fetch_recent_market_data(&price_provider).await {
        Ok(prices) => prices,
        Err(e) => {
            error!("Failed to fetch recent market data: {}", e);
            return Err(e);
        }
    };

    if recent_prices.len() < 20 {
        return Err(AppError::External(
            "Insufficient recent price data for forecast generation".to_string()
        ));
    }

    // 3. Convert recent prices to observation symbols
    let observations = convert_to_observations(&recent_prices)?;

    if observations.is_empty() {
        return Err(AppError::External(
            "Failed to convert price data to HMM observations".to_string()
        ));
    }

    info!("Generated {} recent observations for forecasting", observations.len());

    // 4. Generate forecasts for multiple horizons
    let forecast_date = Utc::now().date_naive();
    let forecasts = hmm_inference_service::generate_regime_forecasts(
        pool,
        forecast_date,
        &observations,
    )
    .await?;

    info!("Generated {} forecasts for date {}", forecasts.len(), forecast_date);

    // 5. Cleanup old forecasts (keep last 90 days)
    match hmm_queries::cleanup_old_regime_forecasts(pool, 90).await {
        Ok(deleted) => {
            if deleted > 0 {
                info!("Cleaned up {} old regime forecasts", deleted);
            }
        }
        Err(e) => {
            warn!("Failed to cleanup old forecasts: {}", e);
            // Don't fail the job on cleanup errors
        }
    }

    Ok(forecasts.len())
}

/// Fetch recent market data for observation generation
async fn fetch_recent_market_data(
    price_provider: &AlphaVantageProvider,
) -> Result<Vec<PricePoint>, AppError> {
    // Fetch last 30 days of SPY data
    let days = 30;
    let external_prices = price_provider
        .fetch_daily_history("SPY", days)
        .await
        .map_err(|e| AppError::External(format!("Failed to fetch SPY prices: {}", e)))?;

    if external_prices.is_empty() {
        return Err(AppError::External(
            "No price data available for SPY".to_string()
        ));
    }

    // Convert ExternalPricePoint to PricePoint
    let prices: Vec<PricePoint> = external_prices
        .into_iter()
        .map(|ext_point| PricePoint {
            id: Uuid::new_v4(),
            ticker: "SPY".to_string(),
            date: ext_point.date,
            close_price: ext_point.close,
            created_at: Utc::now(),
        })
        .collect();

    Ok(prices)
}

/// Convert price data to HMM observation symbols
fn convert_to_observations(prices: &[PricePoint]) -> Result<Vec<ObservationSymbol>, AppError> {
    use bigdecimal::ToPrimitive;

    if prices.len() < 2 {
        return Err(AppError::External(
            "Need at least 2 price points to calculate observations".to_string()
        ));
    }

    let mut observations = Vec::new();

    // Calculate returns and volatility for recent period
    for window in prices.windows(20) {
        if window.len() < 20 {
            continue;
        }

        // Calculate daily return for the last day in window
        let prev_price = window[window.len() - 2]
            .close_price
            .to_f64()
            .unwrap_or(0.0);
        let curr_price = window[window.len() - 1]
            .close_price
            .to_f64()
            .unwrap_or(0.0);

        if prev_price <= 0.0 {
            continue;
        }

        let daily_return = ((curr_price - prev_price) / prev_price) * 100.0;

        // Calculate 20-day volatility
        let mut returns = Vec::new();
        for i in 1..window.len() {
            let p1 = window[i - 1].close_price.to_f64().unwrap_or(0.0);
            let p2 = window[i].close_price.to_f64().unwrap_or(0.0);
            if p1 > 0.0 {
                returns.push((p2 - p1) / p1);
            }
        }

        if returns.is_empty() {
            continue;
        }

        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let volatility = (variance.sqrt() * (252.0_f64).sqrt()) * 100.0; // Annualized %

        let observation = ObservationSymbol::from_metrics(daily_return, volatility);
        observations.push(observation);
    }

    // Return the most recent observations (last 5 for current state estimation)
    let recent_count = 5.min(observations.len());
    Ok(observations[observations.len() - recent_count..].to_vec())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_convert_observations_sufficient_data() {
        // This test validates the observation conversion logic
        // Real test would need mock price data
    }
}

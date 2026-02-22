//! Market Regime Update Background Job
//!
//! This job runs daily at 5:00 PM ET (17:00) to detect and record the current
//! market regime based on recent benchmark data (typically SPY). The regime
//! classification is used to dynamically adjust risk alert thresholds.
//!
//! # Job Schedule
//!
//! - **Production**: Daily at 5:00 PM ET (0 0 17 * * *)
//! - Same time as risk snapshots to maintain consistency
//!
//! # Purpose
//!
//! Market regimes affect how we interpret risk metrics:
//! - **Bull Market**: Tighten thresholds (0.8x) to catch early warning signs
//! - **Bear Market**: Loosen thresholds (1.3x) to reduce noise during volatility
//! - **High Volatility**: Significantly loosen (1.5x) to avoid alert fatigue
//! - **Normal Market**: Standard thresholds (1.0x)
//!
//! By detecting regime changes automatically, the system adapts risk alerting
//! to market conditions without manual intervention.
//!
//! # Detection Algorithm
//!
//! 1. Fetch recent price data for benchmark (SPY)
//! 2. Calculate volatility (annualized standard deviation)
//! 3. Calculate returns over lookback period
//! 4. Classify regime based on volatility and return thresholds:
//!    - High Volatility: vol > 35%
//!    - Bull: returns > 0% and vol < 20%
//!    - Bear: returns < 0% and vol > 25%
//!    - Normal: everything else
//! 5. Store regime with confidence score in database
//!
//! # Error Handling
//!
//! - Failures are logged but don't crash the job
//! - If benchmark data is unavailable, job fails gracefully
//! - Previous regime remains in effect until successfully updated
//!
//! # Performance Considerations
//!
//! - Runs after market close (5:00 PM ET) to capture full day's data
//! - Single calculation per day (lightweight operation)
//! - Uses existing price data infrastructure
//! - Respects rate limits for external API calls

use crate::errors::AppError;
use crate::models::RegimeDetectionParams;
use crate::services::{job_scheduler_service::{JobContext, JobResult}, market_regime_service};
use chrono::Utc;
use tracing::{error, info};

/// Main entry point for the market regime update job.
///
/// This function is called by the job scheduler at 5:00 PM ET daily.
/// It detects the current market regime and stores it in the database.
///
/// # Arguments
///
/// * `ctx` - Job context containing database pool and external services
///
/// # Returns
///
/// * `Ok(JobResult)` - Success with processed count
/// * `Err(AppError)` - Critical failure that should be logged
pub async fn update_market_regime(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📊 Starting market regime update job");

    let today = Utc::now().date_naive();
    info!("Detecting market regime for date: {}", today);

    // Use default detection parameters
    // Can be customized via environment variables in the future
    let params = RegimeDetectionParams::default();

    info!(
        "Using detection params: benchmark={}, lookback_days={}",
        params.benchmark_ticker, params.lookback_days
    );

    // Detect and update regime
    match market_regime_service::update_regime_for_date(
        &ctx.pool,
        today,
        &params,
        ctx.price_provider.as_ref(),
    )
    .await
    {
        Ok(regime) => {
            info!(
                "✅ Market regime updated: {} (volatility: {}%, confidence: {}%)",
                regime.regime_type,
                regime.volatility_level,
                regime.confidence
            );

            Ok(JobResult {
                items_processed: 1,
                items_failed: 0,
            })
        }
        Err(e) => {
            error!("❌ Failed to update market regime: {}", e);

            // Return error but don't panic - previous regime will remain in effect
            Ok(JobResult {
                items_processed: 0,
                items_failed: 1,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_compiles() {
        // This test ensures the job function compiles correctly
    }
}

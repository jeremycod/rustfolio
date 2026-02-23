use sqlx::PgPool;
use tracing::{error, info};

use crate::db::hmm_queries;
use crate::errors::AppError;
use crate::external::alphavantage::AlphaVantageProvider;
use crate::services::hmm_training_service::{train_hmm_model, HmmTrainingConfig};
use crate::services::job_scheduler_service::{JobContext, JobResult};

/// Job scheduler-compatible HMM training job wrapper
///
/// This wrapper function adapts the HMM training logic to work with the
/// job scheduler system, returning JobResult for proper tracking and reporting.
pub async fn train_hmm_model_job(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🧠 Starting HMM model training job");
    let start_time = std::time::Instant::now();

    match run_hmm_training_job(&ctx.pool).await {
        Ok(()) => {
            let elapsed = start_time.elapsed();
            info!(
                "✅ HMM training job completed successfully in {:.2}s",
                elapsed.as_secs_f64()
            );
            Ok(JobResult {
                items_processed: 1, // 1 model trained
                items_failed: 0,
            })
        }
        Err(e) => {
            error!("❌ HMM training job failed: {}", e);
            Err(e)
        }
    }
}

/// Monthly HMM retraining job
///
/// Runs on the 1st of each month at midnight to retrain the HMM model
/// with the latest market data.
///
/// Schedule: `0 0 1 * *` (cron: At 00:00 on day-of-month 1)
#[allow(dead_code)]
pub async fn run_hmm_training_job(pool: &PgPool) -> Result<(), AppError> {
    info!("Starting HMM model retraining job");

    let start_time = std::time::Instant::now();

    // Initialize price provider
    let price_provider = match AlphaVantageProvider::from_env() {
        Ok(provider) => provider,
        Err(e) => {
            error!("Failed to initialize Alpha Vantage provider: {}", e);
            return Err(AppError::External(format!("Price provider init failed: {}", e)));
        }
    };

    // Configure training
    let config = HmmTrainingConfig {
        ticker: "SPY".to_string(),
        lookback_years: 10,
        max_iterations: Some(100),
        volatility_window_days: 20,
    };

    // Train HMM model
    match train_hmm_model(pool, &config, &price_provider).await {
        Ok(trained_model) => {
            info!(
                "HMM training completed successfully: {} states, accuracy: {:.2}%",
                trained_model.num_states,
                trained_model.model_accuracy * 100.0
            );

            // Save model to database
            match hmm_queries::save_hmm_model(pool, &trained_model).await {
                Ok(model_id) => {
                    info!("HMM model saved to database with ID: {}", model_id);
                }
                Err(e) => {
                    error!("Failed to save HMM model to database: {}", e);
                    return Err(e);
                }
            }

            // Cleanup old models (keep only last 6 months worth = 6 models)
            match hmm_queries::cleanup_old_hmm_models(pool, "SPY", 6).await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!("Cleaned up {} old HMM models", deleted_count);
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup old HMM models: {}", e);
                    // Don't fail job on cleanup error
                }
            }

            let elapsed = start_time.elapsed();
            info!(
                "HMM retraining job completed successfully in {:.2}s",
                elapsed.as_secs_f64()
            );

            Ok(())
        }
        Err(e) => {
            error!("HMM training failed: {}", e);
            Err(e)
        }
    }
}

/// Test HMM training on a small dataset
///
/// This can be used for manual testing or validation
#[allow(dead_code)]
pub async fn run_hmm_training_test(pool: &PgPool) -> Result<(), AppError> {
    info!("Running HMM training test with limited dataset");

    let price_provider = match AlphaVantageProvider::from_env() {
        Ok(provider) => provider,
        Err(e) => {
            error!("Failed to initialize Alpha Vantage provider: {}", e);
            return Err(AppError::External(format!("Price provider init failed: {}", e)));
        }
    };

    // Use shorter lookback for testing
    let config = HmmTrainingConfig {
        ticker: "SPY".to_string(),
        lookback_years: 3, // Just 3 years for quick test
        max_iterations: Some(50), // Fewer iterations
        volatility_window_days: 20,
    };

    match train_hmm_model(pool, &config, &price_provider).await {
        Ok(trained_model) => {
            info!(
                "Test HMM training successful: {} states, accuracy: {:.2}%",
                trained_model.num_states,
                trained_model.model_accuracy * 100.0
            );
            Ok(())
        }
        Err(e) => {
            error!("Test HMM training failed: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_job_compiles() {
        // Ensures job compiles correctly
        // Integration test would require database connection
    }
}

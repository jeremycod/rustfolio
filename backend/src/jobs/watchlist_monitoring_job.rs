use crate::db::watchlist_queries;
use crate::errors::AppError;
use crate::services::job_scheduler_service::{JobContext, JobResult};
use crate::services::watchlist_monitoring_service;
use tracing::{error, info, warn};

const INTER_TICKER_DELAY_MS: u64 = 500;

/// Main entry point for the watchlist monitoring background job.
///
/// This job:
/// 1. Gets all distinct tickers across all watchlists
/// 2. For each ticker, runs monitoring checks (thresholds, patterns, sentiment)
/// 3. Stores generated alerts in the database
///
/// Designed to run every 30 minutes during market hours.
pub async fn run_watchlist_monitoring(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("Starting watchlist monitoring job");

    let pool = ctx.pool.as_ref();

    // Get all unique tickers from watchlists
    let tickers = watchlist_queries::get_all_watchlist_tickers(pool)
        .await
        .map_err(AppError::Db)?;

    if tickers.is_empty() {
        info!("No watchlist tickers to monitor");
        return Ok(JobResult {
            items_processed: 0,
            items_failed: 0,
        });
    }

    info!("Monitoring {} watchlist tickers", tickers.len());

    let mut processed = 0;
    let mut failed = 0;
    let mut total_alerts = 0;

    for ticker in &tickers {
        match watchlist_monitoring_service::monitor_ticker(pool, ticker).await {
            Ok(results) => {
                // Store alerts
                for result in &results {
                    match watchlist_queries::create_watchlist_alert(
                        pool,
                        result.watchlist_item_id,
                        result.user_id,
                        &result.ticker,
                        &result.alert_type,
                        &result.severity,
                        &result.message,
                        Some(result.actual_value),
                        result.threshold_value,
                        result.metadata.clone(),
                    )
                    .await
                    {
                        Ok(_) => {
                            total_alerts += 1;
                            info!(
                                "Alert generated for {}: {} ({})",
                                result.ticker, result.alert_type, result.severity
                            );
                        }
                        Err(e) => {
                            warn!("Failed to store alert for {}: {}", result.ticker, e);
                        }
                    }
                }

                processed += 1;
            }
            Err(e) => {
                error!("Failed to monitor ticker {}: {}", ticker, e);
                failed += 1;
            }
        }

        // Delay between tickers to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(INTER_TICKER_DELAY_MS)).await;
    }

    info!(
        "Watchlist monitoring completed: {} tickers processed, {} failed, {} alerts generated",
        processed, failed, total_alerts
    );

    Ok(JobResult {
        items_processed: processed,
        items_failed: failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(INTER_TICKER_DELAY_MS, 500);
    }
}

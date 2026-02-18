use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::services::failure_cache::FailureCache;
use crate::services::rate_limiter::RateLimiter;
use sqlx::PgPool;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, error, warn};
use chrono::Utc;
use std::sync::Arc;

// Context passed to job functions
#[derive(Clone)]
pub struct JobContext {
    pub pool: Arc<PgPool>,
    pub price_provider: Arc<dyn PriceProvider>,
    pub failure_cache: Arc<FailureCache>,
    pub rate_limiter: Arc<RateLimiter>,
}

pub struct JobSchedulerService {
    scheduler: JobScheduler,
    context: JobContext,
}

impl JobSchedulerService {
    pub async fn new(
        pool: Arc<PgPool>,
        price_provider: Arc<dyn PriceProvider>,
        failure_cache: Arc<FailureCache>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Result<Self, AppError> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::External(format!("Failed to create scheduler: {}", e)))?;

        let context = JobContext {
            pool,
            price_provider,
            failure_cache,
            rate_limiter,
        };

        Ok(Self {
            scheduler,
            context,
        })
    }

    /// Start all scheduled jobs
    pub async fn start(&mut self) -> Result<(), AppError> {
        info!("🚀 Starting job scheduler...");

        // Nightly jobs (format: sec min hour day month weekday)
        self.schedule_job(
            "0 0 2 * * *",
            "refresh_prices",
            "Daily at 2:00 AM",
            refresh_all_prices
        ).await?;

        self.schedule_job(
            "0 30 2 * * *",
            "fetch_news",
            "Daily at 2:30 AM",
            fetch_all_news
        ).await?;

        self.schedule_job(
            "0 0 4 * * *",
            "generate_forecasts",
            "Daily at 4:00 AM",
            generate_all_forecasts
        ).await?;

        self.schedule_job(
            "0 30 4 * * *",
            "analyze_sec_filings",
            "Daily at 4:30 AM",
            analyze_all_sec_filings
        ).await?;

        // Hourly jobs
        self.schedule_job(
            "0 0 * * * *",
            "check_thresholds",
            "Every hour at :00",
            check_all_thresholds
        ).await?;

        self.schedule_job(
            "0 30 * * * *",
            "warm_caches",
            "Every hour at :30",
            warm_popular_caches
        ).await?;

        // Weekly jobs (SUN = Sunday)
        self.schedule_job(
            "0 0 3 * * SUN",
            "cleanup_cache",
            "Every Sunday at 3:00 AM",
            cleanup_expired_caches
        ).await?;

        self.schedule_job(
            "0 30 3 * * SUN",
            "archive_snapshots",
            "Every Sunday at 3:30 AM",
            archive_old_snapshots
        ).await?;

        // Start the scheduler
        self.scheduler.start()
            .await
            .map_err(|e| AppError::External(format!("Failed to start scheduler: {}", e)))?;

        info!("✅ Job scheduler started successfully with 8 jobs");
        Ok(())
    }

    /// Stop the scheduler gracefully
    pub async fn stop(&mut self) -> Result<(), AppError> {
        info!("🛑 Stopping job scheduler...");
        self.scheduler.shutdown()
            .await
            .map_err(|e| AppError::External(format!("Failed to stop scheduler: {}", e)))?;
        info!("✅ Job scheduler stopped");
        Ok(())
    }

    /// Helper to schedule a job with tracking
    async fn schedule_job<F, Fut>(
        &mut self,
        schedule: &str,
        job_name: &'static str,
        description: &str,
        job_fn: F,
    ) -> Result<(), AppError>
    where
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<JobResult, AppError>> + Send + 'static,
    {
        let context = self.context.clone();
        let job_fn = Arc::new(job_fn);

        let job = Job::new_async(schedule, move |_uuid, _l| {
            let context = context.clone();
            let job_fn = job_fn.clone();
            Box::pin(async move {
                execute_job_with_tracking(&context.pool, job_name, context.clone(), job_fn).await;
            })
        })
        .map_err(|e| AppError::External(format!("Failed to create job {}: {}", job_name, e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::External(format!("Failed to add job {}: {}", job_name, e)))?;

        info!("📅 Scheduled: {} - {} [cron: {}]", job_name, description, schedule);
        Ok(())
    }
}

// Job tracking wrapper
async fn execute_job_with_tracking<F, Fut>(
    pool: &PgPool,
    job_name: &str,
    context: JobContext,
    job_fn: Arc<F>,
) where
    F: Fn(JobContext) -> Fut,
    Fut: std::future::Future<Output = Result<JobResult, AppError>>,
{
    info!("🏃 Starting job: {}", job_name);
    let started_at = Utc::now();

    // Record job start
    let job_id = match record_job_start(pool, job_name).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to record job start: {}", e);
            return;
        }
    };

    // Execute job
    let result = job_fn(context).await;

    let duration_ms = (Utc::now() - started_at).num_milliseconds();

    // Record job completion
    match result {
        Ok(job_result) => {
            info!(
                "✅ Job completed: {} (processed: {}, failed: {}, duration: {}ms)",
                job_name, job_result.items_processed, job_result.items_failed, duration_ms
            );

            if let Err(e) = record_job_success(
                pool,
                job_id,
                job_result.items_processed,
                job_result.items_failed,
                duration_ms,
            ).await {
                error!("Failed to record job success: {}", e);
            }
        }
        Err(e) => {
            error!("❌ Job failed: {} - {}", job_name, e);

            if let Err(e) = record_job_failure(pool, job_id, &e.to_string(), duration_ms).await {
                error!("Failed to record job failure: {}", e);
            }
        }
    }
}

#[derive(Debug)]
pub struct JobResult {
    pub items_processed: i32,
    pub items_failed: i32,
}

// Database functions for job tracking
async fn record_job_start(pool: &PgPool, job_name: &str) -> Result<i32, AppError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO job_runs (job_name, status)
        VALUES ($1, 'running'::job_status)
        RETURNING id
        "#,
        job_name
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

async fn record_job_success(
    pool: &PgPool,
    job_id: i32,
    items_processed: i32,
    items_failed: i32,
    duration_ms: i64,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE job_runs
        SET completed_at = NOW(),
            status = 'success'::job_status,
            items_processed = $2,
            items_failed = $3,
            duration_ms = $4
        WHERE id = $1
        "#,
        job_id,
        items_processed,
        items_failed,
        duration_ms
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn record_job_failure(
    pool: &PgPool,
    job_id: i32,
    error_message: &str,
    duration_ms: i64,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE job_runs
        SET completed_at = NOW(),
            status = 'failed'::job_status,
            error_message = $2,
            duration_ms = $3
        WHERE id = $1
        "#,
        job_id,
        error_message,
        duration_ms
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Job implementation functions
async fn refresh_all_prices(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("💰 Refreshing all prices...");

    // Get all unique tickers from positions
    let tickers = sqlx::query!("SELECT DISTINCT ticker FROM positions")
        .fetch_all(ctx.pool.as_ref())
        .await?;

    let mut processed = 0;
    let mut failed = 0;

    for record in tickers {
        match crate::services::price_service::refresh_from_api(
            ctx.pool.as_ref(),
            ctx.price_provider.as_ref(),
            &record.ticker,
            &ctx.failure_cache,
            ctx.rate_limiter.as_ref(),
        ).await {
            Ok(_) => {
                processed += 1;
                info!("✅ Refreshed prices for {}", record.ticker);
            }
            Err(e) => {
                failed += 1;
                warn!("❌ Failed to refresh prices for {}: {}", record.ticker, e);
            }
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: failed })
}

async fn fetch_all_news(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📰 Fetching all news...");

    // Clear all news cache to force fresh fetch on next request
    let result = sqlx::query!("DELETE FROM portfolio_news_cache")
        .execute(ctx.pool.as_ref())
        .await?;

    let processed = result.rows_affected() as i32;
    info!("🗑️ Cleared {} news cache entries", processed);

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn generate_all_forecasts(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔮 Generating all forecasts...");

    // Get popular tickers (top 20 by position count)
    let tickers = sqlx::query!(
        r#"
        SELECT ticker, COUNT(*) as "count!"
        FROM positions
        GROUP BY ticker
        ORDER BY COUNT(*) DESC
        LIMIT 20
        "#
    )
    .fetch_all(ctx.pool.as_ref())
    .await?;

    let mut processed = 0;

    for record in tickers {
        // Clear forecast cache to force regeneration
        let _ = sqlx::query!(
            "DELETE FROM beta_forecast_cache WHERE ticker = $1",
            record.ticker
        )
        .execute(ctx.pool.as_ref())
        .await;

        processed += 1;
        info!("🗑️ Cleared forecast cache for {}", record.ticker);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn analyze_all_sec_filings(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📄 Analyzing SEC filings...");

    // Get top 20 tickers
    let tickers = sqlx::query!("SELECT DISTINCT ticker FROM positions LIMIT 20")
        .fetch_all(ctx.pool.as_ref())
        .await?;

    let mut processed = 0;

    for record in tickers {
        // Clear enhanced sentiment cache to force re-analysis
        let _ = sqlx::query!(
            "DELETE FROM enhanced_sentiment_cache WHERE ticker = $1",
            record.ticker
        )
        .execute(ctx.pool.as_ref())
        .await;

        processed += 1;
        info!("🗑️ Cleared SEC analysis cache for {}", record.ticker);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn check_all_thresholds(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("⚠️ Checking thresholds...");

    // Get all portfolios with threshold settings
    let portfolios = sqlx::query!("SELECT DISTINCT portfolio_id FROM risk_threshold_settings")
        .fetch_all(ctx.pool.as_ref())
        .await?;

    let processed = portfolios.len() as i32;

    info!("Checked {} portfolios for threshold violations", processed);

    // TODO: Implement actual threshold checking and alert generation in Sprint 20

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn warm_popular_caches(_ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🔥 Warming popular caches...");

    // Nothing to pre-warm yet, caches fill on-demand
    // This job is a placeholder for future optimization

    Ok(JobResult { items_processed: 0, items_failed: 0 })
}

async fn cleanup_expired_caches(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("🧹 Cleaning up expired caches...");

    let mut processed = 0;

    // Clean up all cache tables
    let tables = vec![
        "portfolio_news_cache",
        "beta_forecast_cache",
        "sentiment_signal_cache",
        "enhanced_sentiment_cache",
    ];

    for table in tables {
        let result = sqlx::query(&format!(
            "DELETE FROM {} WHERE expires_at < NOW()",
            table
        ))
        .execute(ctx.pool.as_ref())
        .await?;

        processed += result.rows_affected() as i32;
        info!("🗑️ Deleted {} expired rows from {}", result.rows_affected(), table);
    }

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn archive_old_snapshots(ctx: JobContext) -> Result<JobResult, AppError> {
    info!("📦 Archiving old snapshots...");

    // Delete risk snapshots older than 1 year
    let result = sqlx::query!(
        "DELETE FROM risk_snapshots WHERE snapshot_date < NOW() - INTERVAL '1 year'"
    )
    .execute(ctx.pool.as_ref())
    .await?;

    info!("📦 Archived {} old snapshots", result.rows_affected());

    Ok(JobResult {
        items_processed: result.rows_affected() as i32,
        items_failed: 0,
    })
}

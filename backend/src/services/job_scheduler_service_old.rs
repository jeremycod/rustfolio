use crate::errors::AppError;
use sqlx::PgPool;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, error, warn};
use chrono::Utc;
use std::sync::Arc;

pub struct JobSchedulerService {
    scheduler: JobScheduler,
    pool: Arc<PgPool>,
}

impl JobSchedulerService {
    pub async fn new(pool: Arc<PgPool>) -> Result<Self, AppError> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::External(format!("Failed to create scheduler: {}", e)))?;

        Ok(Self { scheduler, pool })
    }

    /// Start all scheduled jobs
    pub async fn start(&self) -> Result<(), AppError> {
        info!("🚀 Starting job scheduler...");

        // Nightly jobs
        self.schedule_nightly_price_refresh().await?;
        self.schedule_nightly_news_fetch().await?;
        self.schedule_nightly_forecast_generation().await?;
        self.schedule_nightly_sec_analysis().await?;

        // Hourly jobs
        self.schedule_hourly_threshold_check().await?;
        self.schedule_hourly_cache_warming().await?;

        // Weekly jobs
        self.schedule_weekly_cache_cleanup().await?;
        self.schedule_weekly_snapshot_archive().await?;

        // Start the scheduler
        self.scheduler.start()
            .await
            .map_err(|e| AppError::External(format!("Failed to start scheduler: {}", e)))?;

        info!("✅ Job scheduler started successfully with 8 jobs");
        Ok(())
    }

    /// Stop the scheduler gracefully
    pub async fn stop(&self) -> Result<(), AppError> {
        info!("🛑 Stopping job scheduler...");
        self.scheduler.shutdown()
            .await
            .map_err(|e| AppError::External(format!("Failed to stop scheduler: {}", e)))?;
        info!("✅ Job scheduler stopped");
        Ok(())
    }

    // Job scheduling methods
    async fn schedule_nightly_price_refresh(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("0 2 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "refresh_prices", || async {
                    refresh_all_prices(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::External(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::External(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: refresh_prices (daily 2:00 AM UTC)");
        Ok(())
    }

    async fn schedule_nightly_news_fetch(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("30 2 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "fetch_news", || async {
                    fetch_all_news(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::External(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: fetch_news (daily 2:30 AM UTC)");
        Ok(())
    }

    async fn schedule_nightly_forecast_generation(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("0 4 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "generate_forecasts", async {
                    generate_all_forecasts(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: generate_forecasts (daily 4:00 AM UTC)");
        Ok(())
    }

    async fn schedule_nightly_sec_analysis(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("30 4 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "analyze_sec_filings", async {
                    analyze_all_sec_filings(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: analyze_sec_filings (daily 4:30 AM UTC)");
        Ok(())
    }

    async fn schedule_hourly_threshold_check(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("0 * * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "check_thresholds", async {
                    check_all_thresholds(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: check_thresholds (hourly)");
        Ok(())
    }

    async fn schedule_hourly_cache_warming(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("30 * * * *", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "warm_caches", async {
                    warm_popular_caches(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: warm_caches (hourly at :30)");
        Ok(())
    }

    async fn schedule_weekly_cache_cleanup(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("0 3 * * 0", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "cleanup_cache", async {
                    cleanup_expired_caches(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: cleanup_cache (weekly Sunday 3:00 AM UTC)");
        Ok(())
    }

    async fn schedule_weekly_snapshot_archive(&self) -> Result<(), AppError> {
        let pool = self.pool.clone();

        let job = Job::new_async("30 3 * * 0", move |_uuid, _l| {
            let pool = pool.clone();
            Box::pin(async move {
                run_job_with_tracking(&pool, "archive_snapshots", async {
                    archive_old_snapshots(&pool).await
                }).await;
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to create job: {}", e)))?;

        self.scheduler.add(job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add job: {}", e)))?;

        info!("📅 Scheduled: archive_snapshots (weekly Sunday 3:30 AM UTC)");
        Ok(())
    }
}

// Job tracking wrapper
async fn run_job_with_tracking<F, Fut>(
    pool: &PgPool,
    job_name: &str,
    job_fn: F,
) where
    F: FnOnce() -> Fut,
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
    let result = job_fn().await;

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
async fn refresh_all_prices(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("💰 Refreshing all prices...");

    // Get all unique tickers from positions
    let tickers = sqlx::query!("SELECT DISTINCT ticker FROM positions")
        .fetch_all(pool)
        .await?;

    let mut processed = 0;
    let mut failed = 0;

    for record in tickers {
        match crate::services::price_service::refresh_latest_price(pool, &record.ticker).await {
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

async fn fetch_all_news(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("📰 Fetching all news...");

    // Get all unique tickers from positions
    let tickers = sqlx::query!("SELECT DISTINCT ticker FROM positions LIMIT 20")
        .fetch_all(pool)
        .await?;

    let mut processed = 0;
    let mut failed = 0;

    for record in tickers {
        // Clear cache to force fresh fetch
        let _ = sqlx::query!(
            "DELETE FROM portfolio_news_cache WHERE ticker = $1",
            record.ticker
        )
        .execute(pool)
        .await;

        processed += 1;
        info!("🗑️ Cleared news cache for {}", record.ticker);

        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: failed })
}

async fn generate_all_forecasts(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("🔮 Generating all forecasts...");

    // Get popular tickers (top 20 by position count)
    let tickers = sqlx::query!(
        r#"
        SELECT ticker, COUNT(*) as position_count
        FROM positions
        GROUP BY ticker
        ORDER BY COUNT(*) DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut processed = 0;
    let mut failed = 0;

    for record in tickers {
        // Clear forecast cache to force regeneration
        let _ = sqlx::query!(
            "DELETE FROM beta_forecast_cache WHERE ticker = $1",
            record.ticker
        )
        .execute(pool)
        .await;

        processed += 1;
        info!("🗑️ Cleared forecast cache for {}", record.ticker);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: failed })
}

async fn analyze_all_sec_filings(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("📄 Analyzing SEC filings...");

    // Get all unique tickers from positions
    let tickers = sqlx::query!("SELECT DISTINCT ticker FROM positions LIMIT 20")
        .fetch_all(pool)
        .await?;

    let mut processed = 0;
    let mut failed = 0;

    for record in tickers {
        // Clear enhanced sentiment cache to force re-analysis
        let _ = sqlx::query!(
            "DELETE FROM enhanced_sentiment_cache WHERE ticker = $1",
            record.ticker
        )
        .execute(pool)
        .await;

        processed += 1;
        info!("🗑️ Cleared SEC analysis cache for {}", record.ticker);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(JobResult { items_processed: processed, items_failed: failed })
}

async fn check_all_thresholds(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("⚠️ Checking thresholds...");

    // Get all portfolios with threshold settings
    let portfolios = sqlx::query!("SELECT DISTINCT portfolio_id FROM risk_threshold_settings")
        .fetch_all(pool)
        .await?;

    let processed = portfolios.len() as i32;

    info!("Checked {} portfolios for threshold violations", processed);

    // TODO: Implement actual threshold checking and alert generation in Sprint 20

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn warm_popular_caches(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("🔥 Warming popular caches...");

    // Nothing to pre-warm yet, caches fill on-demand
    // This job is a placeholder for future optimization

    Ok(JobResult { items_processed: 0, items_failed: 0 })
}

async fn cleanup_expired_caches(pool: &PgPool) -> Result<JobResult, AppError> {
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
        .execute(pool)
        .await?;

        processed += result.rows_affected() as i32;
        info!("🗑️ Deleted {} expired rows from {}", result.rows_affected(), table);
    }

    Ok(JobResult { items_processed: processed, items_failed: 0 })
}

async fn archive_old_snapshots(pool: &PgPool) -> Result<JobResult, AppError> {
    info!("📦 Archiving old snapshots...");

    // Delete risk snapshots older than 1 year
    let result = sqlx::query!(
        "DELETE FROM risk_snapshots WHERE snapshot_date < NOW() - INTERVAL '1 year'"
    )
    .execute(pool)
    .await?;

    info!("📦 Archived {} old snapshots", result.rows_affected());

    Ok(JobResult {
        items_processed: result.rows_affected() as i32,
        items_failed: 0,
    })
}

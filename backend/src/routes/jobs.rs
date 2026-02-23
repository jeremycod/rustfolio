use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use crate::{errors::AppError, state::AppState};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs))
        .route("/runs/recent", get(recent_job_runs))
        .route("/trigger-all", post(trigger_all_jobs))
        .route("/:job_name/history", get(job_history))
        .route("/:job_name/stats", get(job_stats))
        .route("/:job_name/trigger", post(trigger_job))
}

#[derive(Serialize)]
struct JobInfo {
    #[serde(rename = "name")]
    job_name: String,
    enabled: bool,
    schedule: String,
    description: String,
    last_run: Option<String>,
    last_status: Option<String>,
    next_run: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct JobRun {
    id: i32,
    job_name: String,
    started_at: String,
    completed_at: Option<String>,
    status: String,
    error_message: Option<String>,
    items_processed: Option<i32>,
    items_failed: Option<i32>,
    duration_ms: Option<i64>,
}

#[derive(Serialize)]
struct JobStats {
    job_name: String,
    total_runs: i64,
    successful_runs: i64,
    failed_runs: i64,
    avg_duration_ms: Option<f64>,
    avg_items_processed: Option<f64>,
    last_run: Option<String>,
    last_status: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TriggerJobResponse {
    job_name: String,
    status: String,
    message: String,
    job_id: Option<i32>,
    items_processed: Option<i32>,
    items_failed: Option<i32>,
    duration_ms: Option<i64>,
    error_message: Option<String>,
}

/// GET /api/admin/jobs - List all configured jobs
async fn list_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobInfo>>, AppError> {
    info!("GET /api/admin/jobs - Listing all scheduled jobs");

    // Check if we're in test mode
    let test_mode = std::env::var("JOB_SCHEDULER_TEST_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    // Define all jobs with their schedules and descriptions
    let job_definitions = vec![
        ("refresh_prices", if test_mode { "0 */1 * * * *" } else { "0 0 2 * * *" }, if test_mode { "Every minute (TEST MODE)" } else { "Daily at 2:00 AM" }),
        ("fetch_news", if test_mode { "0 */2 * * * *" } else { "0 30 2 * * *" }, if test_mode { "Every 2 minutes (TEST MODE)" } else { "Daily at 2:30 AM" }),
        ("generate_forecasts", "0 0 4 * * *", "Daily at 4:00 AM"),
        ("analyze_sec_filings", "0 30 4 * * *", "Daily at 4:30 AM"),
        ("check_thresholds", "0 0 * * * *", "Every hour at :00"),
        ("warm_caches", "0 30 * * * *", "Every hour at :30"),
        ("calculate_portfolio_risks", "0 15 * * * *", "Every hour at :15"),
        ("calculate_portfolio_correlations", "0 45 */2 * * *", "Every 2 hours at :45"),
        ("create_daily_risk_snapshots", "0 0 17 * * *", "Daily at 5:00 PM ET"),
        ("update_market_regime", "0 5 17 * * *", "Daily at 5:05 PM ET"),
        ("train_hmm_model", "0 0 0 1 * *", "Monthly on 1st at midnight"),
        ("populate_optimization_cache", if test_mode { "0 */15 * * * *" } else { "0 0 */6 * * *" }, if test_mode { "Every 15 minutes (TEST MODE)" } else { "Every 6 hours" }),
        ("populate_rolling_beta_cache", "0 30 */6 * * *", "Every 6 hours at :30"),
        ("populate_downside_risk_cache", "0 45 */6 * * *", "Every 6 hours at :45"),
        ("cleanup_cache", if test_mode { "0 */3 * * * *" } else { "0 0 3 * * SUN" }, if test_mode { "Every 3 minutes (TEST MODE)" } else { "Every Sunday at 3:00 AM" }),
        ("archive_snapshots", "0 30 3 * * SUN", "Every Sunday at 3:30 AM"),
    ];

    let mut jobs_info = Vec::new();

    for (job_name, schedule, description) in job_definitions {
        // Get last run info from database
        let last_run_info = sqlx::query!(
            r#"
            SELECT started_at::TEXT as "started_at?", status::TEXT as "status?"
            FROM job_runs
            WHERE job_name = $1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
            job_name
        )
        .fetch_optional(&state.pool)
        .await?;

        let (last_run, last_status) = if let Some(info) = last_run_info {
            (info.started_at, info.status)
        } else {
            (None, None)
        };

        jobs_info.push(JobInfo {
            job_name: job_name.to_string(),
            enabled: true,
            schedule: schedule.to_string(),
            description: description.to_string(),
            last_run,
            last_status,
            next_run: None, // TODO: Calculate next run time from cron schedule
        });
    }

    Ok(Json(jobs_info))
}

/// GET /api/admin/jobs/recent - Get recent job runs
async fn recent_job_runs(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobRun>>, AppError> {
    let runs = sqlx::query_as!(
        JobRun,
        r#"
        SELECT
            id,
            job_name,
            started_at::TEXT as "started_at!",
            completed_at::TEXT as "completed_at?",
            status::TEXT as "status!",
            error_message,
            items_processed,
            items_failed,
            duration_ms
        FROM job_runs
        ORDER BY started_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(runs))
}

/// GET /api/admin/jobs/:job_name/history - Get history for a specific job
async fn job_history(
    Path(job_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<JobRun>>, AppError> {
    let runs = sqlx::query_as!(
        JobRun,
        r#"
        SELECT
            id,
            job_name,
            started_at::TEXT as "started_at!",
            completed_at::TEXT as "completed_at?",
            status::TEXT as "status!",
            error_message,
            items_processed,
            items_failed,
            duration_ms
        FROM job_runs
        WHERE job_name = $1
        ORDER BY started_at DESC
        LIMIT 100
        "#,
        job_name
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(runs))
}

/// GET /api/admin/jobs/:job_name/stats - Get statistics for a specific job
async fn job_stats(
    Path(job_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<JobStats>, AppError> {
    let stats = sqlx::query!(
        r#"
        SELECT
            job_name,
            COUNT(*) as "total_runs!",
            COUNT(*) FILTER (WHERE status = 'success') as "successful_runs!",
            COUNT(*) FILTER (WHERE status = 'failed') as "failed_runs!",
            AVG(duration_ms) as "avg_duration_ms",
            AVG(items_processed) as "avg_items_processed",
            MAX(started_at)::TEXT as "last_run?",
            (SELECT status::TEXT FROM job_runs WHERE job_name = $1 ORDER BY started_at DESC LIMIT 1) as "last_status?"
        FROM job_runs
        WHERE job_name = $1
        GROUP BY job_name
        "#,
        job_name
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(JobStats {
        job_name: stats.job_name,
        total_runs: stats.total_runs,
        successful_runs: stats.successful_runs,
        failed_runs: stats.failed_runs,
        avg_duration_ms: stats.avg_duration_ms.and_then(|d| d.to_string().parse::<f64>().ok()),
        avg_items_processed: stats.avg_items_processed.and_then(|d| d.to_string().parse::<f64>().ok()),
        last_run: stats.last_run,
        last_status: stats.last_status,
    }))
}

/// POST /api/admin/jobs/:job_name/trigger - Manually trigger a job execution
///
/// This endpoint allows manual triggering of background jobs for testing and maintenance.
/// The job is executed immediately and tracked in the job_runs table.
///
/// # Supported Jobs
///
/// - `calculate_portfolio_risks` - Recalculate risk metrics for all portfolios
/// - `calculate_portfolio_correlations` - Recalculate correlations for all portfolios
/// - `create_daily_risk_snapshots` - Create risk snapshots for all portfolios
///
/// # Example Request
///
/// ```
/// POST /api/admin/jobs/calculate_portfolio_risks/trigger
/// ```
///
/// # Example Response
///
/// ```json
/// {
///   "job_name": "calculate_portfolio_risks",
///   "status": "success",
///   "message": "Job completed successfully",
///   "job_id": 123,
///   "items_processed": 15,
///   "items_failed": 0,
///   "duration_ms": 12345,
///   "error_message": null
/// }
/// ```
async fn trigger_job(
    Path(job_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<TriggerJobResponse>, AppError> {
    info!("🎯 Manual trigger requested for job: {}", job_name);

    // Validate that the job exists in our known jobs list
    let known_jobs = vec![
        "refresh_prices", "fetch_news", "generate_forecasts", "analyze_sec_filings",
        "check_thresholds", "warm_caches", "calculate_portfolio_risks",
        "calculate_portfolio_correlations", "populate_rolling_beta_cache",
        "create_daily_risk_snapshots", "populate_optimization_cache",
        "update_market_regime", "train_hmm_model",
        "populate_downside_risk_cache",
        "cleanup_cache", "archive_snapshots"
    ];

    if !known_jobs.contains(&job_name.as_str()) {
        return Err(AppError::Validation(format!(
            "Unknown job '{}'. Available jobs: {}",
            job_name, known_jobs.join(", ")
        )));
    }

    // Record job start in job_runs table
    let job_id = sqlx::query!(
        r#"
        INSERT INTO job_runs (job_name, status)
        VALUES ($1, 'running'::job_status)
        RETURNING id
        "#,
        job_name
    )
    .fetch_one(&state.pool)
    .await?
    .id;

    info!("📝 Started job run with ID: {}", job_id);

    let started_at = chrono::Utc::now();

    // Create job context from AppState
    let job_context = crate::services::job_scheduler_service::JobContext {
        pool: Arc::new(state.pool.clone()),
        price_provider: state.price_provider.clone(),
        failure_cache: Arc::new(state.failure_cache.clone()),
        rate_limiter: state.rate_limiter.clone(),
    };

    // Execute the appropriate job function
    let result = match job_name.as_str() {
        "refresh_prices" => {
            info!("💰 Executing refresh prices job...");
            crate::services::job_scheduler_service::refresh_all_prices(job_context).await
        }
        "fetch_news" => {
            info!("📰 Executing fetch news job...");
            crate::services::job_scheduler_service::fetch_all_news(job_context).await
        }
        "generate_forecasts" => {
            info!("🔮 Executing generate forecasts job...");
            crate::services::job_scheduler_service::generate_all_forecasts(job_context).await
        }
        "analyze_sec_filings" => {
            info!("📄 Executing analyze SEC filings job...");
            crate::services::job_scheduler_service::analyze_all_sec_filings(job_context).await
        }
        "check_thresholds" => {
            info!("⚠️ Executing check thresholds job...");
            crate::services::job_scheduler_service::check_all_thresholds(job_context).await
        }
        "warm_caches" => {
            info!("🔥 Executing warm caches job...");
            crate::services::job_scheduler_service::warm_popular_caches(job_context).await
        }
        "calculate_portfolio_risks" => {
            info!("🔍 Executing portfolio risk calculation job...");
            crate::jobs::portfolio_risk_job::calculate_all_portfolio_risks(job_context).await
        }
        "calculate_portfolio_correlations" => {
            info!("🔗 Executing portfolio correlations calculation job...");
            crate::jobs::portfolio_correlations_job::calculate_all_portfolio_correlations(job_context).await
        }
        "populate_rolling_beta_cache" => {
            info!("📊 Executing rolling beta cache population job...");
            crate::jobs::rolling_beta_cache_job::populate_rolling_beta_caches(job_context).await
        }
        "create_daily_risk_snapshots" => {
            info!("📸 Executing daily risk snapshots job...");
            crate::jobs::daily_risk_snapshots_job::create_all_daily_risk_snapshots(job_context).await
        }
        "populate_optimization_cache" => {
            info!("🎯 Executing optimization cache population job...");
            crate::jobs::populate_optimization_cache_job::populate_all_optimization_caches(job_context).await
        }
        "update_market_regime" => {
            info!("📊 Executing market regime update job...");
            crate::jobs::market_regime_update_job::update_market_regime(job_context).await
        }
        "train_hmm_model" => {
            info!("🧠 Executing HMM model training job...");
            crate::services::job_scheduler_service::train_hmm_wrapper(job_context).await
        }
        "generate_regime_forecasts" => {
            info!("🔮 Executing regime forecast generation job...");
            crate::jobs::regime_forecast_job::generate_all_regime_forecasts(job_context).await
        }
        "populate_downside_risk_cache" => {
            info!("📉 Executing downside risk cache population job...");
            crate::jobs::downside_risk_cache_job::populate_downside_risk_caches(job_context).await
        }
        "cleanup_cache" => {
            info!("🧹 Executing cleanup cache job...");
            crate::services::job_scheduler_service::cleanup_expired_caches(job_context).await
        }
        "archive_snapshots" => {
            info!("📦 Executing archive snapshots job...");
            crate::services::job_scheduler_service::archive_old_snapshots(job_context).await
        }
        _ => {
            // Unknown job
            let error_msg = format!(
                "Unknown job '{}'. Available jobs: {}",
                job_name,
                known_jobs.join(", ")
            );
            error!("{}", error_msg);

            // Record the failure
            let duration_ms = (chrono::Utc::now() - started_at).num_milliseconds();
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
                error_msg,
                duration_ms
            )
            .execute(&state.pool)
            .await?;

            return Ok(Json(TriggerJobResponse {
                job_name,
                status: "failed".to_string(),
                message: "Unknown job".to_string(),
                job_id: Some(job_id),
                items_processed: None,
                items_failed: None,
                duration_ms: Some(duration_ms),
                error_message: Some(error_msg),
            }));
        }
    };

    let duration_ms = (chrono::Utc::now() - started_at).num_milliseconds();

    // Update job_runs with results
    match result {
        Ok(job_result) => {
            info!(
                "✅ Job '{}' completed successfully: {} processed, {} failed, duration: {}ms",
                job_name, job_result.items_processed, job_result.items_failed, duration_ms
            );

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
                job_result.items_processed,
                job_result.items_failed,
                duration_ms
            )
            .execute(&state.pool)
            .await?;

            Ok(Json(TriggerJobResponse {
                job_name,
                status: "success".to_string(),
                message: format!(
                    "Job completed successfully: {} items processed, {} items failed",
                    job_result.items_processed, job_result.items_failed
                ),
                job_id: Some(job_id),
                items_processed: Some(job_result.items_processed),
                items_failed: Some(job_result.items_failed),
                duration_ms: Some(duration_ms),
                error_message: None,
            }))
        }
        Err(e) => {
            error!("❌ Job '{}' failed: {}", job_name, e);

            let error_message = e.to_string();
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
            .execute(&state.pool)
            .await?;

            Ok(Json(TriggerJobResponse {
                job_name,
                status: "failed".to_string(),
                message: "Job execution failed".to_string(),
                job_id: Some(job_id),
                items_processed: None,
                items_failed: None,
                duration_ms: Some(duration_ms),
                error_message: Some(error_message),
            }))
        }
    }
}

#[derive(Serialize)]
struct TriggerAllJobsResponse {
    total_jobs: usize,
    successful: usize,
    failed: usize,
    job_results: Vec<TriggerJobResponse>,
    total_duration_ms: i64,
}

/// POST /api/admin/jobs/trigger-all - Trigger all critical cache population jobs
async fn trigger_all_jobs(
    State(state): State<AppState>,
) -> Result<Json<TriggerAllJobsResponse>, AppError> {
    info!("🚀 Manual trigger requested for ALL jobs");
    let overall_start = chrono::Utc::now();

    // Define critical jobs to run in sequence (order matters for dependencies)
    let jobs_to_run = vec![
        "refresh_prices",                    // Get latest prices first
        "calculate_portfolio_risks",         // Calculate risk metrics
        "populate_downside_risk_cache",      // Downside risk analysis
        "calculate_portfolio_correlations",  // Correlation analysis
        "populate_rolling_beta_cache",       // Beta calculations
        "update_market_regime",              // Market regime detection
        "generate_regime_forecasts",         // Regime forecasts (requires trained HMM)
        "populate_optimization_cache",       // Portfolio optimization
        "create_daily_risk_snapshots",       // Risk snapshots
    ];

    info!("📋 Will execute {} jobs in sequence", jobs_to_run.len());

    // Create job context from AppState
    let job_context = crate::services::job_scheduler_service::JobContext {
        pool: Arc::new(state.pool.clone()),
        price_provider: state.price_provider.clone(),
        failure_cache: Arc::new(state.failure_cache.clone()),
        rate_limiter: state.rate_limiter.clone(),
    };

    let mut job_results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    // Execute each job in sequence
    for job_name in jobs_to_run.iter() {
        info!("⏳ Executing job: {}", job_name);
        let job_start = chrono::Utc::now();

        // Record job start
        let job_id = sqlx::query!(
            r#"
            INSERT INTO job_runs (job_name, status)
            VALUES ($1, 'running'::job_status)
            RETURNING id
            "#,
            job_name
        )
        .fetch_one(&state.pool)
        .await?
        .id;

        // Execute the job
        let result = match *job_name {
            "refresh_prices" => {
                crate::services::job_scheduler_service::refresh_all_prices(job_context.clone()).await
            }
            "calculate_portfolio_risks" => {
                crate::jobs::portfolio_risk_job::calculate_all_portfolio_risks(job_context.clone()).await
            }
            "populate_downside_risk_cache" => {
                crate::jobs::downside_risk_cache_job::populate_downside_risk_caches(job_context.clone()).await
            }
            "calculate_portfolio_correlations" => {
                crate::jobs::portfolio_correlations_job::calculate_all_portfolio_correlations(job_context.clone()).await
            }
            "populate_rolling_beta_cache" => {
                crate::jobs::rolling_beta_cache_job::populate_rolling_beta_caches(job_context.clone()).await
            }
            "update_market_regime" => {
                crate::jobs::market_regime_update_job::update_market_regime(job_context.clone()).await
            }
            "generate_regime_forecasts" => {
                crate::jobs::regime_forecast_job::generate_all_regime_forecasts(job_context.clone()).await
            }
            "populate_optimization_cache" => {
                crate::jobs::populate_optimization_cache_job::populate_all_optimization_caches(job_context.clone()).await
            }
            "create_daily_risk_snapshots" => {
                crate::jobs::daily_risk_snapshots_job::create_all_daily_risk_snapshots(job_context.clone()).await
            }
            _ => {
                error!("Unknown job: {}", job_name);
                Err(AppError::External(format!("Unknown job: {}", job_name)))
            }
        };

        let duration_ms = (chrono::Utc::now() - job_start).num_milliseconds();

        // Process result and update database
        let job_response = match result {
            Ok(job_result) => {
                successful += 1;
                info!("✅ {} completed successfully ({}ms)", job_name, duration_ms);

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
                    job_result.items_processed,
                    job_result.items_failed,
                    duration_ms
                )
                .execute(&state.pool)
                .await?;

                TriggerJobResponse {
                    job_name: job_name.to_string(),
                    status: "success".to_string(),
                    message: format!("Processed {} items", job_result.items_processed),
                    job_id: Some(job_id),
                    items_processed: Some(job_result.items_processed),
                    items_failed: Some(job_result.items_failed),
                    duration_ms: Some(duration_ms),
                    error_message: None,
                }
            }
            Err(e) => {
                failed += 1;
                let error_msg = e.to_string();
                error!("❌ {} failed: {} ({}ms)", job_name, error_msg, duration_ms);

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
                    error_msg,
                    duration_ms
                )
                .execute(&state.pool)
                .await?;

                TriggerJobResponse {
                    job_name: job_name.to_string(),
                    status: "failed".to_string(),
                    message: "Job execution failed".to_string(),
                    job_id: Some(job_id),
                    items_processed: None,
                    items_failed: None,
                    duration_ms: Some(duration_ms),
                    error_message: Some(error_msg),
                }
            }
        };

        job_results.push(job_response);
    }

    let total_duration_ms = (chrono::Utc::now() - overall_start).num_milliseconds();

    info!(
        "🏁 All jobs completed: {} successful, {} failed, total duration: {}s",
        successful, failed, total_duration_ms / 1000
    );

    Ok(Json(TriggerAllJobsResponse {
        total_jobs: jobs_to_run.len(),
        successful,
        failed,
        job_results,
        total_duration_ms,
    }))
}

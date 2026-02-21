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
        .route("/:job_name/history", get(job_history))
        .route("/:job_name/stats", get(job_stats))
        .route("/:job_name/trigger", post(trigger_job))
}

#[derive(Serialize, sqlx::FromRow)]
struct JobInfo {
    job_name: String,
    enabled: bool,
    schedule: String,
    last_run: Option<String>,
    next_run: Option<String>,
    max_duration_minutes: Option<i32>,
    retry_count: Option<i32>,
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
    let jobs = sqlx::query_as!(
        JobInfo,
        r#"
        SELECT
            job_name,
            enabled,
            schedule,
            last_run::TEXT as "last_run?",
            next_run::TEXT as "next_run?",
            max_duration_minutes,
            retry_count
        FROM job_config
        ORDER BY job_name
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(jobs))
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

    // Validate that the job exists in the job_config table
    let job_config = sqlx::query!(
        r#"
        SELECT job_name, enabled
        FROM job_config
        WHERE job_name = $1
        "#,
        job_name
    )
    .fetch_optional(&state.pool)
    .await?;

    if job_config.is_none() {
        return Err(AppError::External(format!(
            "Job '{}' not found. Available jobs can be listed via GET /api/admin/jobs",
            job_name
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
        "calculate_portfolio_risks" => {
            info!("🔍 Executing portfolio risk calculation job...");
            crate::jobs::portfolio_risk_job::calculate_all_portfolio_risks(job_context).await
        }
        "calculate_portfolio_correlations" => {
            info!("🔗 Executing portfolio correlations calculation job...");
            crate::jobs::portfolio_correlations_job::calculate_all_portfolio_correlations(job_context).await
        }
        "create_daily_risk_snapshots" => {
            info!("📸 Executing daily risk snapshots job...");
            crate::jobs::daily_risk_snapshots_job::create_all_daily_risk_snapshots(job_context).await
        }
        _ => {
            // Job exists in config but isn't implemented for manual triggering
            let error_msg = format!(
                "Job '{}' cannot be manually triggered. Only analytics jobs (calculate_portfolio_risks, \
                calculate_portfolio_correlations, create_daily_risk_snapshots) support manual execution.",
                job_name
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
                message: "Job cannot be manually triggered".to_string(),
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

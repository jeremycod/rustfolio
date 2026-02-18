use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use crate::{errors::AppError, AppState};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs))
        .route("/recent", get(recent_job_runs))
        .route("/:job_name/history", get(job_history))
        .route("/:job_name/stats", get(job_stats))
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

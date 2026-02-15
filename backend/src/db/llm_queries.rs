use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{LlmUsage, CreateLlmUsage, LlmUsageStats};

/// Log LLM usage to database
pub async fn log_usage(
    pool: &PgPool,
    usage: CreateLlmUsage,
) -> Result<LlmUsage, sqlx::Error> {
    sqlx::query_as::<_, LlmUsage>(
        r#"
        INSERT INTO llm_usage (user_id, portfolio_id, model, prompt_tokens, completion_tokens, total_cost)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, portfolio_id, model, prompt_tokens, completion_tokens, total_cost, created_at
        "#
    )
    .bind(usage.user_id)
    .bind(usage.portfolio_id)
    .bind(usage.model)
    .bind(usage.prompt_tokens)
    .bind(usage.completion_tokens)
    .bind(usage.total_cost)
    .fetch_one(pool)
    .await
}

/// Get LLM usage for a specific user within a date range
pub async fn get_user_usage(
    pool: &PgPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<LlmUsage>, sqlx::Error> {
    sqlx::query_as::<_, LlmUsage>(
        r#"
        SELECT id, user_id, portfolio_id, model, prompt_tokens, completion_tokens, total_cost, created_at
        FROM llm_usage
        WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
        ORDER BY created_at DESC
        "#
    )
    .bind(user_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

/// Get LLM usage statistics for a user
pub async fn get_user_usage_stats(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<LlmUsageStats, sqlx::Error> {
    // Get overall stats
    let overall_row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total_requests,
            COALESCE(SUM(prompt_tokens), 0) as total_prompt_tokens,
            COALESCE(SUM(completion_tokens), 0) as total_completion_tokens,
            COALESCE(SUM(total_cost), 0) as total_cost
        FROM llm_usage
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // Get current month stats
    let month_row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(total_cost), 0) as current_month_cost
        FROM llm_usage
        WHERE user_id = $1
          AND created_at >= date_trunc('month', CURRENT_DATE)
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(LlmUsageStats {
        total_requests: overall_row.get("total_requests"),
        total_prompt_tokens: overall_row.get("total_prompt_tokens"),
        total_completion_tokens: overall_row.get("total_completion_tokens"),
        total_cost: overall_row.get("total_cost"),
        current_month_cost: month_row.get("current_month_cost"),
    })
}

/// Get total LLM cost across all users (admin function)
pub async fn get_total_cost(pool: &PgPool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(total_cost), 0) as total
        FROM llm_usage
        "#
    )
    .fetch_one(pool)
    .await?;

    let total: sqlx::types::BigDecimal = row.get("total");
    Ok(total.to_string().parse().unwrap_or(0.0))
}

/// Get recent LLM usage (admin function)
pub async fn get_recent_usage(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<LlmUsage>, sqlx::Error> {
    sqlx::query_as::<_, LlmUsage>(
        r#"
        SELECT id, user_id, portfolio_id, model, prompt_tokens, completion_tokens, total_cost, created_at
        FROM llm_usage
        ORDER BY created_at DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::CachedExplanation;

/// Retrieve the latest non-expired explanation for a symbol
pub async fn get_cached_explanation(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<CachedExplanation>, sqlx::Error> {
    sqlx::query_as::<_, CachedExplanation>(
        r#"
        SELECT id, symbol, explanation, factors_snapshot, generated_at, expires_at
        FROM recommendation_explanations
        WHERE symbol = $1
          AND expires_at > NOW()
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

/// Store a generated explanation in the cache
pub async fn cache_explanation(
    pool: &PgPool,
    symbol: &str,
    explanation: &str,
    factors_snapshot: &serde_json::Value,
    generated_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<CachedExplanation, sqlx::Error> {
    sqlx::query_as::<_, CachedExplanation>(
        r#"
        INSERT INTO recommendation_explanations
            (symbol, explanation, factors_snapshot, generated_at, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, symbol, explanation, factors_snapshot, generated_at, expires_at
        "#,
    )
    .bind(symbol)
    .bind(explanation)
    .bind(factors_snapshot)
    .bind(generated_at)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Delete expired explanations (for periodic cleanup)
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM recommendation_explanations
        WHERE expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

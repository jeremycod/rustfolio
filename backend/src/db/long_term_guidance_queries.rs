use sqlx::PgPool;
use uuid::Uuid;

use crate::models::long_term_guidance::LongTermGuidanceResponse;

/// Retrieve cached long-term guidance for a portfolio
pub async fn get_cached_guidance(
    pool: &PgPool,
    portfolio_id: Uuid,
    goal: &str,
    horizon_years: i32,
    risk_tolerance: &str,
) -> Result<Option<LongTermGuidanceResponse>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT guidance_data
        FROM long_term_guidance_cache
        WHERE portfolio_id = $1
          AND goal = $2
          AND horizon_years = $3
          AND risk_tolerance = $4
          AND expires_at > NOW()
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
        portfolio_id,
        goal,
        horizon_years,
        risk_tolerance,
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let response: LongTermGuidanceResponse = serde_json::from_value(r.guidance_data)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            Ok(Some(response))
        }
        None => Ok(None),
    }
}

/// Cache long-term guidance results (1 hour TTL)
pub async fn cache_guidance(
    pool: &PgPool,
    portfolio_id: Uuid,
    goal: &str,
    horizon_years: i32,
    risk_tolerance: &str,
    response: &LongTermGuidanceResponse,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    let guidance_data = serde_json::to_value(response)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    sqlx::query!(
        r#"
        INSERT INTO long_term_guidance_cache (
            id, portfolio_id, goal, horizon_years, risk_tolerance,
            guidance_data, generated_at, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW() + INTERVAL '1 hour')
        ON CONFLICT (portfolio_id, goal, horizon_years, risk_tolerance)
        DO UPDATE SET
            guidance_data = EXCLUDED.guidance_data,
            generated_at = EXCLUDED.generated_at,
            expires_at = EXCLUDED.expires_at
        "#,
        id,
        portfolio_id,
        goal,
        horizon_years,
        risk_tolerance,
        guidance_data,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired cache entries
#[allow(dead_code)]
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM long_term_guidance_cache WHERE expires_at < NOW()"
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

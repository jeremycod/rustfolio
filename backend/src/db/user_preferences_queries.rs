use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{UserPreferences, UpdateUserPreferences};

/// Get user preferences by user ID
pub async fn get_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserPreferences>, sqlx::Error> {
    sqlx::query_as::<_, UserPreferences>(
        r#"
        SELECT id, user_id, llm_enabled, consent_given_at, narrative_cache_hours, created_at, updated_at
        FROM user_preferences
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Create or update user preferences
pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    preferences: UpdateUserPreferences,
) -> Result<UserPreferences, sqlx::Error> {
    let consent_given_at = if preferences.llm_enabled {
        Some(Utc::now())
    } else {
        None
    };

    let narrative_cache_hours = preferences.narrative_cache_hours.unwrap_or(24);

    sqlx::query_as::<_, UserPreferences>(
        r#"
        INSERT INTO user_preferences (user_id, llm_enabled, consent_given_at, narrative_cache_hours, updated_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
            llm_enabled = EXCLUDED.llm_enabled,
            consent_given_at = CASE
                WHEN EXCLUDED.llm_enabled = TRUE AND user_preferences.consent_given_at IS NULL
                THEN EXCLUDED.consent_given_at
                ELSE user_preferences.consent_given_at
            END,
            narrative_cache_hours = EXCLUDED.narrative_cache_hours,
            updated_at = NOW()
        RETURNING id, user_id, llm_enabled, consent_given_at, narrative_cache_hours, created_at, updated_at
        "#
    )
    .bind(user_id)
    .bind(preferences.llm_enabled)
    .bind(consent_given_at)
    .bind(narrative_cache_hours)
    .fetch_one(pool)
    .await
}

/// Update LLM consent for a user
pub async fn update_llm_consent(
    pool: &PgPool,
    user_id: Uuid,
    consent: bool,
) -> Result<UserPreferences, sqlx::Error> {
    let consent_given_at = if consent {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query_as::<_, UserPreferences>(
        r#"
        INSERT INTO user_preferences (user_id, llm_enabled, consent_given_at, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
            llm_enabled = $2,
            consent_given_at = CASE
                WHEN $2 = TRUE AND user_preferences.consent_given_at IS NULL
                THEN $3
                ELSE user_preferences.consent_given_at
            END,
            updated_at = NOW()
        RETURNING id, user_id, llm_enabled, consent_given_at, narrative_cache_hours, created_at, updated_at
        "#
    )
    .bind(user_id)
    .bind(consent)
    .bind(consent_given_at)
    .fetch_one(pool)
    .await
}

/// Delete user preferences (revoke all consent and preferences)
#[allow(dead_code)]
pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM user_preferences
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

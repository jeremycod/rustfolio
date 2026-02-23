use bigdecimal::{BigDecimal, FromPrimitive};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{RiskPreferences, UpdateRiskPreferences};

/// Get user risk preferences by user ID
pub async fn get_preferences_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<RiskPreferences>, sqlx::Error> {
    sqlx::query_as::<_, RiskPreferences>(
        r#"
        SELECT
            id,
            user_id,
            llm_enabled,
            consent_given_at,
            narrative_cache_hours,
            risk_appetite,
            forecast_horizon_preference,
            signal_sensitivity,
            sentiment_weight,
            technical_weight,
            fundamental_weight,
            custom_settings,
            created_at,
            updated_at
        FROM user_preferences
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Upsert user risk preferences (create or update)
pub async fn upsert_preferences(
    pool: &PgPool,
    user_id: Uuid,
    update: &UpdateRiskPreferences,
) -> Result<RiskPreferences, sqlx::Error> {
    // Convert weights to BigDecimal
    let sentiment_weight = update
        .sentiment_weight
        .and_then(|w| BigDecimal::from_f64(w));
    let technical_weight = update
        .technical_weight
        .and_then(|w| BigDecimal::from_f64(w));
    let fundamental_weight = update
        .fundamental_weight
        .and_then(|w| BigDecimal::from_f64(w));

    // Convert enums to strings
    let risk_appetite_str = update
        .risk_appetite
        .map(|ra| ra.to_string());
    let signal_sensitivity_str = update
        .signal_sensitivity
        .map(|ss| ss.to_string());

    sqlx::query_as::<_, RiskPreferences>(
        r#"
        INSERT INTO user_preferences (
            user_id,
            llm_enabled,
            consent_given_at,
            narrative_cache_hours,
            risk_appetite,
            forecast_horizon_preference,
            signal_sensitivity,
            sentiment_weight,
            technical_weight,
            fundamental_weight,
            custom_settings,
            updated_at
        )
        VALUES (
            $1,
            COALESCE($2, false),
            CASE WHEN COALESCE($2, false) = TRUE THEN NOW() ELSE NULL END,
            COALESCE($3, 24),
            COALESCE($4, 'Balanced'),
            COALESCE($5, 6),
            COALESCE($6, 'Medium'),
            COALESCE($7, 0.3),
            COALESCE($8, 0.4),
            COALESCE($9, 0.3),
            $10,
            NOW()
        )
        ON CONFLICT (user_id)
        DO UPDATE SET
            llm_enabled = COALESCE($2, user_preferences.llm_enabled),
            consent_given_at = CASE
                WHEN COALESCE($2, false) = TRUE AND user_preferences.consent_given_at IS NULL
                THEN NOW()
                ELSE user_preferences.consent_given_at
            END,
            narrative_cache_hours = COALESCE($3, user_preferences.narrative_cache_hours),
            risk_appetite = COALESCE($4, user_preferences.risk_appetite),
            forecast_horizon_preference = COALESCE($5, user_preferences.forecast_horizon_preference),
            signal_sensitivity = COALESCE($6, user_preferences.signal_sensitivity),
            sentiment_weight = COALESCE($7, user_preferences.sentiment_weight),
            technical_weight = COALESCE($8, user_preferences.technical_weight),
            fundamental_weight = COALESCE($9, user_preferences.fundamental_weight),
            custom_settings = COALESCE($10, user_preferences.custom_settings),
            updated_at = NOW()
        RETURNING
            id,
            user_id,
            llm_enabled,
            consent_given_at,
            narrative_cache_hours,
            risk_appetite,
            forecast_horizon_preference,
            signal_sensitivity,
            sentiment_weight,
            technical_weight,
            fundamental_weight,
            custom_settings,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(update.llm_enabled)
    .bind(update.narrative_cache_hours)
    .bind(risk_appetite_str)
    .bind(update.forecast_horizon_preference)
    .bind(signal_sensitivity_str)
    .bind(sentiment_weight)
    .bind(technical_weight)
    .bind(fundamental_weight)
    .bind(&update.custom_settings)
    .fetch_one(pool)
    .await
}

/// Upsert preferences from a full RiskPreferences object (for defaults)
pub async fn upsert_full_preferences(
    pool: &PgPool,
    user_id: Uuid,
    prefs: &RiskPreferences,
) -> Result<RiskPreferences, sqlx::Error> {
    let risk_appetite_str = prefs.risk_appetite.to_string();
    let signal_sensitivity_str = prefs.signal_sensitivity.to_string();

    sqlx::query_as::<_, RiskPreferences>(
        r#"
        INSERT INTO user_preferences (
            user_id,
            llm_enabled,
            consent_given_at,
            narrative_cache_hours,
            risk_appetite,
            forecast_horizon_preference,
            signal_sensitivity,
            sentiment_weight,
            technical_weight,
            fundamental_weight,
            custom_settings,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
            llm_enabled = $2,
            consent_given_at = $3,
            narrative_cache_hours = $4,
            risk_appetite = $5,
            forecast_horizon_preference = $6,
            signal_sensitivity = $7,
            sentiment_weight = $8,
            technical_weight = $9,
            fundamental_weight = $10,
            custom_settings = $11,
            updated_at = NOW()
        RETURNING
            id,
            user_id,
            llm_enabled,
            consent_given_at,
            narrative_cache_hours,
            risk_appetite,
            forecast_horizon_preference,
            signal_sensitivity,
            sentiment_weight,
            technical_weight,
            fundamental_weight,
            custom_settings,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .bind(prefs.llm_enabled)
    .bind(prefs.consent_given_at)
    .bind(prefs.narrative_cache_hours)
    .bind(risk_appetite_str)
    .bind(prefs.forecast_horizon_preference)
    .bind(signal_sensitivity_str)
    .bind(&prefs.sentiment_weight)
    .bind(&prefs.technical_weight)
    .bind(&prefs.fundamental_weight)
    .bind(&prefs.custom_settings)
    .fetch_one(pool)
    .await
}

/// Get default preferences (returns None, defaults are created on-demand)
#[allow(dead_code)]
pub async fn get_default_preferences() -> Option<RiskPreferences> {
    None
}

/// Delete user preferences
#[allow(dead_code)]
pub async fn delete_preferences(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM user_preferences
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

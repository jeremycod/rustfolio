use crate::models::risk::{RiskThresholdSettings, UpdateRiskThresholds};
use sqlx::PgPool;
use uuid::Uuid;

/// Get risk threshold settings for a portfolio.
/// Returns default settings if none exist.
pub async fn get_thresholds(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<RiskThresholdSettings, sqlx::Error> {
    let result = sqlx::query_as::<_, RiskThresholdSettings>(
        r#"
        SELECT
            id::text,
            portfolio_id::text,
            volatility_warning_threshold,
            volatility_critical_threshold,
            drawdown_warning_threshold,
            drawdown_critical_threshold,
            beta_warning_threshold,
            beta_critical_threshold,
            risk_score_warning_threshold,
            risk_score_critical_threshold,
            var_warning_threshold,
            var_critical_threshold,
            created_at,
            updated_at
        FROM risk_threshold_settings
        WHERE portfolio_id = $1
        "#,
    )
    .bind(portfolio_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(settings) => Ok(settings),
        Err(sqlx::Error::RowNotFound) => {
            // Create default settings if none exist
            create_default_thresholds(pool, portfolio_id).await
        }
        Err(e) => Err(e),
    }
}

/// Create default threshold settings for a portfolio.
async fn create_default_thresholds(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<RiskThresholdSettings, sqlx::Error> {
    sqlx::query_as::<_, RiskThresholdSettings>(
        r#"
        INSERT INTO risk_threshold_settings (
            portfolio_id,
            volatility_warning_threshold,
            volatility_critical_threshold,
            drawdown_warning_threshold,
            drawdown_critical_threshold,
            beta_warning_threshold,
            beta_critical_threshold,
            risk_score_warning_threshold,
            risk_score_critical_threshold,
            var_warning_threshold,
            var_critical_threshold
        ) VALUES ($1, 30.0, 50.0, -20.0, -35.0, 1.5, 2.0, 60.0, 80.0, -5.0, -10.0)
        RETURNING
            id::text,
            portfolio_id::text,
            volatility_warning_threshold,
            volatility_critical_threshold,
            drawdown_warning_threshold,
            drawdown_critical_threshold,
            beta_warning_threshold,
            beta_critical_threshold,
            risk_score_warning_threshold,
            risk_score_critical_threshold,
            var_warning_threshold,
            var_critical_threshold,
            created_at,
            updated_at
        "#,
    )
    .bind(portfolio_id)
    .fetch_one(pool)
    .await
}

/// Update risk threshold settings for a portfolio.
/// Uses UPSERT pattern to create if doesn't exist or update if exists.
pub async fn upsert_thresholds(
    pool: &PgPool,
    portfolio_id: Uuid,
    update: &UpdateRiskThresholds,
) -> Result<RiskThresholdSettings, sqlx::Error> {
    // First, get existing settings or create defaults
    let existing = get_thresholds(pool, portfolio_id).await?;

    // Apply updates (use existing values if not provided)
    let volatility_warning = update.volatility_warning_threshold.unwrap_or(existing.volatility_warning_threshold);
    let volatility_critical = update.volatility_critical_threshold.unwrap_or(existing.volatility_critical_threshold);
    let drawdown_warning = update.drawdown_warning_threshold.unwrap_or(existing.drawdown_warning_threshold);
    let drawdown_critical = update.drawdown_critical_threshold.unwrap_or(existing.drawdown_critical_threshold);
    let beta_warning = update.beta_warning_threshold.unwrap_or(existing.beta_warning_threshold);
    let beta_critical = update.beta_critical_threshold.unwrap_or(existing.beta_critical_threshold);
    let risk_score_warning = update.risk_score_warning_threshold.unwrap_or(existing.risk_score_warning_threshold);
    let risk_score_critical = update.risk_score_critical_threshold.unwrap_or(existing.risk_score_critical_threshold);
    let var_warning = update.var_warning_threshold.unwrap_or(existing.var_warning_threshold);
    let var_critical = update.var_critical_threshold.unwrap_or(existing.var_critical_threshold);

    // Update the record
    sqlx::query_as::<_, RiskThresholdSettings>(
        r#"
        UPDATE risk_threshold_settings
        SET
            volatility_warning_threshold = $2,
            volatility_critical_threshold = $3,
            drawdown_warning_threshold = $4,
            drawdown_critical_threshold = $5,
            beta_warning_threshold = $6,
            beta_critical_threshold = $7,
            risk_score_warning_threshold = $8,
            risk_score_critical_threshold = $9,
            var_warning_threshold = $10,
            var_critical_threshold = $11
        WHERE portfolio_id = $1
        RETURNING
            id::text,
            portfolio_id::text,
            volatility_warning_threshold,
            volatility_critical_threshold,
            drawdown_warning_threshold,
            drawdown_critical_threshold,
            beta_warning_threshold,
            beta_critical_threshold,
            risk_score_warning_threshold,
            risk_score_critical_threshold,
            var_warning_threshold,
            var_critical_threshold,
            created_at,
            updated_at
        "#,
    )
    .bind(portfolio_id)
    .bind(volatility_warning)
    .bind(volatility_critical)
    .bind(drawdown_warning)
    .bind(drawdown_critical)
    .bind(beta_warning)
    .bind(beta_critical)
    .bind(risk_score_warning)
    .bind(risk_score_critical)
    .bind(var_warning)
    .bind(var_critical)
    .fetch_one(pool)
    .await
}

/// Delete risk threshold settings for a portfolio (revert to defaults).
#[allow(dead_code)]
pub async fn delete_thresholds(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM risk_threshold_settings
        WHERE portfolio_id = $1
        "#,
    )
    .bind(portfolio_id)
    .execute(pool)
    .await?;

    Ok(())
}

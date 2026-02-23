use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::StateProbabilities;
use crate::services::hmm_training_service::TrainedHmmModel;

/// Database record for HMM model
#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub struct HmmModelRecord {
    pub id: Uuid,
    pub model_name: String,
    pub market: String,
    pub num_states: i32,
    pub state_names: sqlx::types::Json<Vec<String>>,
    pub transition_matrix: sqlx::types::Json<Vec<Vec<f64>>>,
    pub emission_params: sqlx::types::Json<Vec<Vec<f64>>>,
    pub observation_symbols: sqlx::types::Json<Vec<String>>,
    pub trained_on_date: NaiveDate,
    pub training_data_start: NaiveDate,
    pub training_data_end: NaiveDate,
    pub model_accuracy: Option<bigdecimal::BigDecimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database record for regime forecast
#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub struct RegimeForecastRecord {
    pub id: Uuid,
    pub forecast_date: NaiveDate,
    pub horizon_days: i32,
    pub predicted_regime: String,
    pub regime_probabilities: sqlx::types::Json<serde_json::Value>,
    pub transition_probability: bigdecimal::BigDecimal,
    pub confidence_level: String,
    pub hmm_model_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Save trained HMM model to database
#[allow(dead_code)]
pub async fn save_hmm_model(
    pool: &PgPool,
    model: &TrainedHmmModel,
) -> Result<Uuid, AppError> {
    let model_accuracy = bigdecimal::BigDecimal::from_str(&model.model_accuracy.to_string())
        .map_err(|e| AppError::External(format!("Failed to convert accuracy: {}", e)))?;

    let record = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO hmm_models (
            model_name,
            market,
            num_states,
            state_names,
            transition_matrix,
            emission_params,
            observation_symbols,
            trained_on_date,
            training_data_start,
            training_data_end,
            model_accuracy
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (model_name, market, trained_on_date)
        DO UPDATE SET
            num_states = EXCLUDED.num_states,
            state_names = EXCLUDED.state_names,
            transition_matrix = EXCLUDED.transition_matrix,
            emission_params = EXCLUDED.emission_params,
            observation_symbols = EXCLUDED.observation_symbols,
            training_data_start = EXCLUDED.training_data_start,
            training_data_end = EXCLUDED.training_data_end,
            model_accuracy = EXCLUDED.model_accuracy,
            updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(&model.model_name)
    .bind(&model.market)
    .bind(model.num_states as i32)
    .bind(sqlx::types::Json(&model.state_names))
    .bind(sqlx::types::Json(&model.transition_matrix))
    .bind(sqlx::types::Json(&model.emission_params))
    .bind(sqlx::types::Json(&model.observation_symbols))
    .bind(chrono::Utc::now().date_naive())
    .bind(model.training_data_start)
    .bind(model.training_data_end)
    .bind(model_accuracy)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Load the latest HMM model for a specific market
#[allow(dead_code)]
pub async fn load_latest_hmm_model(
    pool: &PgPool,
    market: &str,
) -> Result<HmmModelRecord, AppError> {
    let record = sqlx::query_as::<_, HmmModelRecord>(
        r#"
        SELECT
            id,
            model_name,
            market,
            num_states,
            state_names,
            transition_matrix,
            emission_params,
            observation_symbols,
            trained_on_date,
            training_data_start,
            training_data_end,
            model_accuracy,
            created_at,
            updated_at
        FROM hmm_models
        WHERE market = $1
        ORDER BY trained_on_date DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(market)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Load HMM model by ID
#[allow(dead_code)]
pub async fn load_hmm_model_by_id(
    pool: &PgPool,
    model_id: Uuid,
) -> Result<HmmModelRecord, AppError> {
    let record = sqlx::query_as::<_, HmmModelRecord>(
        r#"
        SELECT
            id,
            model_name,
            market,
            num_states,
            state_names,
            transition_matrix,
            emission_params,
            observation_symbols,
            trained_on_date,
            training_data_start,
            training_data_end,
            model_accuracy,
            created_at,
            updated_at
        FROM hmm_models
        WHERE id = $1
        "#,
    )
    .bind(model_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Update market regime with HMM probabilities
#[allow(dead_code)]
pub async fn update_regime_with_hmm(
    pool: &PgPool,
    date: NaiveDate,
    hmm_probabilities: &StateProbabilities,
    predicted_regime: Option<&str>,
    transition_probability: Option<f64>,
) -> Result<(), AppError> {
    let probs_json = json!({
        "bull": hmm_probabilities.bull,
        "bear": hmm_probabilities.bear,
        "high_volatility": hmm_probabilities.high_volatility,
        "normal": hmm_probabilities.normal,
    });

    let transition_prob_decimal = transition_probability
        .map(|p| bigdecimal::BigDecimal::from_str(&p.to_string()))
        .transpose()
        .map_err(|e| AppError::External(format!("Failed to convert transition prob: {}", e)))?;

    sqlx::query(
        r#"
        UPDATE market_regimes
        SET
            hmm_probabilities = $2,
            predicted_regime = $3,
            transition_probability = $4,
            updated_at = NOW()
        WHERE date = $1
        "#,
    )
    .bind(date)
    .bind(sqlx::types::Json(probs_json))
    .bind(predicted_regime)
    .bind(transition_prob_decimal)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(())
}

/// Save regime forecast
#[allow(dead_code)]
pub async fn save_regime_forecast(
    pool: &PgPool,
    forecast_date: NaiveDate,
    horizon_days: i32,
    predicted_regime: &str,
    regime_probabilities: &StateProbabilities,
    transition_probability: f64,
    confidence_level: &str,
    hmm_model_id: Option<Uuid>,
) -> Result<Uuid, AppError> {
    let probs_json = json!({
        "bull": regime_probabilities.bull,
        "bear": regime_probabilities.bear,
        "high_volatility": regime_probabilities.high_volatility,
        "normal": regime_probabilities.normal,
    });

    let transition_prob_decimal =
        bigdecimal::BigDecimal::from_str(&transition_probability.to_string())
            .map_err(|e| AppError::External(format!("Failed to convert transition prob: {}", e)))?;

    let record = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO regime_forecasts (
            forecast_date,
            horizon_days,
            predicted_regime,
            regime_probabilities,
            transition_probability,
            confidence_level,
            hmm_model_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (forecast_date, horizon_days)
        DO UPDATE SET
            predicted_regime = EXCLUDED.predicted_regime,
            regime_probabilities = EXCLUDED.regime_probabilities,
            transition_probability = EXCLUDED.transition_probability,
            confidence_level = EXCLUDED.confidence_level,
            hmm_model_id = EXCLUDED.hmm_model_id,
            created_at = NOW()
        RETURNING id
        "#,
    )
    .bind(forecast_date)
    .bind(horizon_days)
    .bind(predicted_regime)
    .bind(sqlx::types::Json(probs_json))
    .bind(transition_prob_decimal)
    .bind(confidence_level)
    .bind(hmm_model_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Get regime forecast for a specific date and horizon
#[allow(dead_code)]
pub async fn get_regime_forecast(
    pool: &PgPool,
    forecast_date: NaiveDate,
    horizon_days: i32,
) -> Result<RegimeForecastRecord, AppError> {
    let record = sqlx::query_as::<_, RegimeForecastRecord>(
        r#"
        SELECT
            id,
            forecast_date,
            horizon_days,
            predicted_regime,
            regime_probabilities,
            transition_probability,
            confidence_level,
            hmm_model_id,
            created_at
        FROM regime_forecasts
        WHERE forecast_date = $1 AND horizon_days = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(forecast_date)
    .bind(horizon_days)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Get latest regime forecast for a specific horizon
#[allow(dead_code)]
pub async fn get_latest_regime_forecast(
    pool: &PgPool,
    horizon_days: i32,
) -> Result<RegimeForecastRecord, AppError> {
    let record = sqlx::query_as::<_, RegimeForecastRecord>(
        r#"
        SELECT
            id,
            forecast_date,
            horizon_days,
            predicted_regime,
            regime_probabilities,
            transition_probability,
            confidence_level,
            hmm_model_id,
            created_at
        FROM regime_forecasts
        WHERE horizon_days = $1
        ORDER BY forecast_date DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(horizon_days)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(record)
}

/// Get all HMM models for a market
#[allow(dead_code)]
pub async fn get_hmm_models_by_market(
    pool: &PgPool,
    market: &str,
    limit: i64,
) -> Result<Vec<HmmModelRecord>, AppError> {
    let records = sqlx::query_as::<_, HmmModelRecord>(
        r#"
        SELECT
            id,
            model_name,
            market,
            num_states,
            state_names,
            transition_matrix,
            emission_params,
            observation_symbols,
            trained_on_date,
            training_data_start,
            training_data_end,
            model_accuracy,
            created_at,
            updated_at
        FROM hmm_models
        WHERE market = $1
        ORDER BY trained_on_date DESC
        LIMIT $2
        "#,
    )
    .bind(market)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(records)
}

/// Delete old HMM models (keep only the N most recent per market)
#[allow(dead_code)]
pub async fn cleanup_old_hmm_models(
    pool: &PgPool,
    market: &str,
    keep_count: i64,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM hmm_models
        WHERE market = $1
        AND id NOT IN (
            SELECT id
            FROM hmm_models
            WHERE market = $1
            ORDER BY trained_on_date DESC
            LIMIT $2
        )
        "#,
    )
    .bind(market)
    .bind(keep_count)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(result.rows_affected())
}

/// Delete old regime forecasts (keep only the last N days)
#[allow(dead_code)]
pub async fn cleanup_old_regime_forecasts(
    pool: &PgPool,
    keep_days: i64,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM regime_forecasts
        WHERE forecast_date < CURRENT_DATE - $1
        "#,
    )
    .bind(keep_days as i32)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_queries_compile() {
        // This test ensures queries compile
        // Integration tests would require test database
    }
}

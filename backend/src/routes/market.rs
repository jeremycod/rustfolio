// Enhanced market regime routes with HMM support

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use bigdecimal::ToPrimitive;
use serde::Serialize;
use tracing::warn;

use crate::db::{hmm_queries, market_regime_queries};
use crate::models::hmm_regime::{RegimeForecastParams, StateProbabilities};
use crate::models::{RegimeHistoryParams, RegimeType};
use crate::state::AppState;

// ==============================================================================
// Router
// ==============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/market/regime", get(get_current_regime_enhanced))
        .route("/market/regime/history", get(get_regime_history))
        .route("/market/regime/forecast", get(get_regime_forecast))
}

// ==============================================================================
// Response Types
// ==============================================================================

#[derive(Debug, Serialize)]
struct CurrentRegimeResponse {
    regime_type: String,
    confidence: f64,
    volatility_level: f64,
    market_return: Option<f64>,
    benchmark_ticker: String,
    threshold_multiplier: f64,
    date: String,
}

// ==============================================================================
// Handlers
// ==============================================================================

/// GET /api/market/regime
///
/// Get current market regime
async fn get_current_regime_enhanced(State(state): State<AppState>) -> impl IntoResponse {
    // Get current regime from database
    let regime = match market_regime_queries::get_current_regime(&state.pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get current regime: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve market regime"
                })),
            )
                .into_response();
        }
    };

    // Build response
    let regime_type = RegimeType::from_string(&regime.regime_type);
    let threshold_multiplier = regime_type.threshold_multiplier();

    let response = CurrentRegimeResponse {
        date: regime.date.to_string(),
        regime_type: regime.regime_type.clone(),
        confidence: regime.confidence.to_f64().unwrap_or(0.0),
        volatility_level: regime.volatility_level.to_f64().unwrap_or(0.0),
        market_return: regime.market_return.and_then(|r| r.to_f64()),
        benchmark_ticker: regime.benchmark_ticker.clone(),
        threshold_multiplier,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/market/regime/history?days=90
///
/// Historical regime data
async fn get_regime_history(
    State(state): State<AppState>,
    Query(params): Query<RegimeHistoryParams>,
) -> impl IntoResponse {
    match crate::services::market_regime_service::get_regime_history(&state.pool, params.days).await
    {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get regime history: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /api/market/regime/forecast?days=10
///
/// Forecast regime N days ahead using HMM
async fn get_regime_forecast(
    State(state): State<AppState>,
    Query(params): Query<RegimeForecastParams>,
) -> impl IntoResponse {
    let validated_params = params.validated();
    let max_days = validated_params.days;

    // Collect forecasts for each day from 1 to max_days
    let mut forecasts = Vec::new();

    for day in 1..=max_days {
        match hmm_queries::get_latest_regime_forecast(&state.pool, day).await {
            Ok(forecast_record) => {
                // Parse probabilities from JSONB
                let regime_probs = parse_regime_probabilities(&forecast_record.regime_probabilities.0);

                // Convert confidence string to number
                let confidence_num = match forecast_record.confidence_level.as_str() {
                    "high" => 0.8,
                    "medium" => 0.6,
                    "low" => 0.4,
                    _ => 0.5,
                };

                let forecast = serde_json::json!({
                    "days_ahead": forecast_record.horizon_days,
                    "predicted_regime": forecast_record.predicted_regime,
                    "confidence": confidence_num,
                    "state_probabilities": regime_probs,
                    "transition_probability": forecast_record.transition_probability.to_f64().unwrap_or(0.0)
                });

                forecasts.push(forecast);
            }
            Err(_) => {
                // Skip missing forecasts but continue with others
                continue;
            }
        }
    }

    if forecasts.is_empty() {
        // No cached forecast available for any horizon
        warn!(
            "No cached forecast available for up to {} days horizon",
            max_days
        );
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "No forecast available for this horizon",
                "message": "Forecasts are generated daily. Please try again later or use a different horizon (5, 10, or 30 days)."
            })),
        )
            .into_response();
    }

    // Return response with forecasts array
    let response = serde_json::json!({
        "forecasts": forecasts
    });

    (StatusCode::OK, Json(response)).into_response()
}

// ==============================================================================
// Helper Functions
// ==============================================================================

/// Parse regime probabilities from JSONB value
fn parse_regime_probabilities(json: &serde_json::Value) -> StateProbabilities {
    let map = json.as_object().unwrap();
    StateProbabilities {
        bull: map
            .get("bull")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25),
        bear: map
            .get("bear")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25),
        high_volatility: map
            .get("high_volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25),
        normal: map
            .get("normal")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_compile() {
        // This test ensures the routes compile correctly
        let _router = router();
    }
}

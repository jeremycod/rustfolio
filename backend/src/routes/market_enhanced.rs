// Enhanced market regime routes with HMM support
// This file contains updated handlers that will replace existing market.rs handlers

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::db::{hmm_queries, market_regime_queries};
use crate::models::{
    EnhancedRegimeResponse, RegimeForecast, RegimeForecastParams, RegimeHistoryParams,
    RegimeType, StateProbabilities,
};
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
struct CurrentRegimeEnhancedResponse {
    regime: RegimeData,
    adjusted_thresholds: AdjustedThresholdsData,
}

#[derive(Debug, Serialize)]
struct RegimeData {
    date: String,
    regime_type: String,
    confidence: f64,
    volatility_level: f64,
    market_return: Option<f64>,
    benchmark_ticker: String,

    // HMM additions
    #[serde(skip_serializing_if = "Option::is_none")]
    volatility_based: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hmm_most_likely: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hmm_probabilities: Option<StateProbabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    predicted_regime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition_probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ensemble_confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
struct AdjustedThresholdsData {
    multiplier: f64,
    description: String,
    example_volatility_warning: f64,
    example_volatility_critical: f64,
    example_drawdown_warning: f64,
    example_drawdown_critical: f64,
}

// ==============================================================================
// Handlers
// ==============================================================================

/// GET /api/market/regime
///
/// Enhanced endpoint with HMM probabilities and ensemble detection
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

    // Extract HMM data if available
    let hmm_probabilities = regime.hmm_probabilities.as_ref().and_then(|json| {
        let map = json.as_object()?;
        Some(StateProbabilities {
            bull: map.get("bull")?.as_f64().unwrap_or(0.0),
            bear: map.get("bear")?.as_f64().unwrap_or(0.0),
            high_volatility: map.get("high_volatility")?.as_f64().unwrap_or(0.0),
            normal: map.get("normal")?.as_f64().unwrap_or(0.0),
        })
    });

    let hmm_most_likely = hmm_probabilities.as_ref().map(|p| p.most_likely_state().to_string());

    // Calculate ensemble confidence if HMM data is available
    let rule_based_confidence = regime.confidence.to_f64().unwrap_or(0.0);
    let ensemble_confidence = hmm_probabilities.as_ref().map(|probs| {
        let hmm_conf = probs.confidence() * 100.0;
        calculate_ensemble_confidence(&regime.regime_type, hmm_most_likely.as_deref(), rule_based_confidence, hmm_conf)
    });

    // Build response
    let regime_type = RegimeType::from_string(&regime.regime_type);
    let threshold_multiplier = regime_type.threshold_multiplier();

    let response = CurrentRegimeEnhancedResponse {
        regime: RegimeData {
            date: regime.date.to_string(),
            regime_type: regime.regime_type.clone(),
            confidence: rule_based_confidence,
            volatility_level: regime.volatility_level.to_f64().unwrap_or(0.0),
            market_return: regime.market_return.and_then(|r| r.to_f64()),
            benchmark_ticker: regime.benchmark_ticker.clone(),
            volatility_based: Some(regime.regime_type.clone()),
            hmm_most_likely,
            hmm_probabilities,
            predicted_regime: regime.predicted_regime.clone(),
            transition_probability: regime
                .transition_probability
                .and_then(|p| p.to_f64()),
            ensemble_confidence,
        },
        adjusted_thresholds: AdjustedThresholdsData {
            multiplier: threshold_multiplier,
            description: get_threshold_description(&regime_type),
            example_volatility_warning: 30.0 * threshold_multiplier,
            example_volatility_critical: 50.0 * threshold_multiplier,
            example_drawdown_warning: -20.0 * threshold_multiplier,
            example_drawdown_critical: -35.0 * threshold_multiplier,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/market/regime/history?days=90
///
/// Historical regime data (unchanged from original)
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

                let forecast = RegimeForecast {
                    forecast_horizon_days: forecast_record.horizon_days,
                    predicted_regime: forecast_record.predicted_regime,
                    regime_probabilities: regime_probs,
                    transition_probability: forecast_record
                        .transition_probability
                        .to_f64()
                        .unwrap_or(0.0),
                    confidence: forecast_record.confidence_level,
                };

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

/// Calculate ensemble confidence when both rule-based and HMM agree/disagree
fn calculate_ensemble_confidence(
    rule_based_regime: &str,
    hmm_regime: Option<&str>,
    rule_confidence: f64,
    hmm_confidence: f64,
) -> f64 {
    match hmm_regime {
        Some(hmm) if hmm == rule_based_regime => {
            // Both agree: boost confidence
            (rule_confidence.max(hmm_confidence) * 1.2).min(100.0)
        }
        Some(_) => {
            // Disagree: average confidence, slightly reduced
            ((rule_confidence + hmm_confidence) / 2.0) * 0.9
        }
        None => {
            // No HMM data: use rule-based confidence
            rule_confidence
        }
    }
}

/// Get threshold description for a regime type
fn get_threshold_description(regime_type: &RegimeType) -> String {
    match regime_type {
        RegimeType::Bull => "Stricter thresholds to catch early risk signals".to_string(),
        RegimeType::Bear => "Relaxed thresholds to reduce noise during volatility".to_string(),
        RegimeType::HighVolatility => "Significantly relaxed to avoid alert fatigue".to_string(),
        RegimeType::Normal => "Standard thresholds for normal market conditions".to_string(),
    }
}

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
    fn test_ensemble_confidence_agreement() {
        let confidence = calculate_ensemble_confidence("bull", Some("bull"), 80.0, 75.0);
        assert!(confidence > 80.0); // Should be boosted
        assert!(confidence <= 100.0); // Should be capped
    }

    #[test]
    fn test_ensemble_confidence_disagreement() {
        let confidence = calculate_ensemble_confidence("bull", Some("bear"), 80.0, 70.0);
        assert!(confidence < 80.0); // Should be reduced
        assert!(confidence > 60.0); // Should still be reasonable
    }

    #[test]
    fn test_ensemble_confidence_no_hmm() {
        let confidence = calculate_ensemble_confidence("bull", None, 80.0, 0.0);
        assert_eq!(confidence, 80.0); // Should use rule-based only
    }

    #[test]
    fn test_routes_compile() {
        let _router = router();
    }
}

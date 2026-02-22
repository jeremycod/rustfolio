use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::models::RegimeHistoryParams;
use crate::services::market_regime_service;
use crate::state::AppState;

// ==============================================================================
// Router
// ==============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/market/regime", get(get_current_regime))
        .route("/market/regime/history", get(get_regime_history))
}

// ==============================================================================
// Handlers
// ==============================================================================

/// GET /api/market/regime
///
/// Get the current market regime with adjusted risk thresholds.
///
/// This endpoint returns the most recent regime classification along with
/// the threshold multipliers that should be applied to risk alerts.
///
/// # Response
///
/// Returns a `CurrentRegimeWithThresholds` object containing:
/// - Current regime details (type, volatility, confidence)
/// - Adjusted threshold information with examples
///
/// # Example
///
/// ```json
/// {
///   "regime": {
///     "id": "...",
///     "date": "2026-02-22",
///     "regime_type": "bull",
///     "volatility_level": 15.5,
///     "market_return": 5.2,
///     "confidence": 85.0,
///     "benchmark_ticker": "SPY",
///     "threshold_multiplier": 0.8
///   },
///   "adjusted_thresholds": {
///     "multiplier": 0.8,
///     "description": "Stricter thresholds to catch early risk signals",
///     "example_volatility_warning": 24.0,
///     "example_volatility_critical": 40.0
///   }
/// }
/// ```
async fn get_current_regime(State(state): State<AppState>) -> impl IntoResponse {
    match market_regime_service::get_current_regime_with_thresholds(&state.pool).await {
        Ok(regime) => (StatusCode::OK, Json(regime)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get current regime: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /api/market/regime/history?days=90
///
/// Get historical market regime classifications.
///
/// This endpoint returns a time series of market regimes over a specified
/// period, useful for analyzing regime changes and their impact on portfolios.
///
/// # Query Parameters
///
/// - `days` (optional): Number of days to look back (default: 90)
///
/// # Response
///
/// Returns an array of `MarketRegime` objects, ordered by date (most recent first).
///
/// # Example
///
/// ```json
/// [
///   {
///     "id": "...",
///     "date": "2026-02-22",
///     "regime_type": "bull",
///     "volatility_level": 15.5,
///     "market_return": 5.2,
///     "confidence": 85.0,
///     "benchmark_ticker": "SPY",
///     "threshold_multiplier": 0.8
///   },
///   {
///     "id": "...",
///     "date": "2026-02-21",
///     "regime_type": "normal",
///     "volatility_level": 18.0,
///     "market_return": 2.1,
///     "confidence": 70.0,
///     "benchmark_ticker": "SPY",
///     "threshold_multiplier": 1.0
///   }
/// ]
/// ```
async fn get_regime_history(
    State(state): State<AppState>,
    Query(params): Query<RegimeHistoryParams>,
) -> impl IntoResponse {
    match market_regime_service::get_regime_history(&state.pool, params.days).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get regime history: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
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

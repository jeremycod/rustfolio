use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::Deserialize;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{RiskAssessment, RiskThresholds, SetThresholdsRequest};
use crate::services::risk_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions/:ticker", get(get_position_risk))
        .route("/portfolios/:portfolio_id", get(get_portfolio_risk))
        .route("/thresholds", get(get_thresholds))
        .route("/thresholds", post(set_thresholds))
}

/// Query parameters for risk calculation
#[derive(Debug, Deserialize)]
pub struct RiskQueryParams {
    /// Number of days for the rolling window (default: 90)
    #[serde(default = "default_days")]
    pub days: i64,

    /// Benchmark ticker for beta calculation (default: "SPY")
    #[serde(default = "default_benchmark")]
    pub benchmark: String,
}

fn default_days() -> i64 {
    90
}

fn default_benchmark() -> String {
    "SPY".to_string()
}

/// GET /api/risk/positions/:ticker
///
/// Calculate and return risk metrics for a specific ticker.
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
/// - `benchmark`: Benchmark ticker for beta (default: "SPY")
///
/// Example: GET /api/risk/positions/AAPL?days=60&benchmark=SPY
#[axum::debug_handler]
pub async fn get_position_risk(
    Path(ticker): Path<String>,
    Query(params): Query<RiskQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<RiskAssessment>, AppError> {
    info!(
        "GET /api/risk/positions/{} - Computing risk metrics (days={}, benchmark={})",
        ticker, params.days, params.benchmark
    );

    let risk_assessment = risk_service::compute_risk_metrics(
        &state.pool,
        &ticker,
        params.days,
        &params.benchmark,
        state.price_provider.as_ref(),
    )
    .await
    .map_err(|e| {
        // Use warn instead of error for missing data - this is expected for
        // mutual funds, bonds, and other non-stock securities
        match &e {
            AppError::External(msg) if msg.contains("No price data") => {
                warn!("No price data available for {}: {}", ticker, msg);
            }
            _ => {
                error!("Failed to compute risk metrics for {}: {}", ticker, e);
            }
        }
        e
    })?;

    Ok(Json(risk_assessment))
}

/// GET /api/risk/portfolios/:portfolio_id
///
/// Calculate aggregated risk metrics for a portfolio.
///
/// Query parameters:
/// - `days`: Rolling window in days (default: 90)
/// - `benchmark`: Benchmark ticker for beta (default: "SPY")
///
/// Example: GET /api/risk/portfolios/{uuid}?days=60
pub async fn get_portfolio_risk(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<RiskQueryParams>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(
        "GET /api/risk/portfolios/{} - Computing portfolio risk (days={}, benchmark={})",
        portfolio_id, params.days, params.benchmark
    );

    // TODO: Implement portfolio-level aggregation
    // This will need to:
    // 1. Fetch all positions in the portfolio
    // 2. Compute risk metrics for each position
    // 3. Aggregate metrics weighted by position size
    // 4. Return PortfolioRisk struct

    error!("Portfolio risk aggregation not yet implemented");
    Err(AppError::External(
        "Portfolio risk aggregation not yet implemented".to_string(),
    ))
}

/// GET /api/risk/thresholds
///
/// Retrieve user-configured risk warning thresholds.
///
/// Returns default thresholds if none are configured.
pub async fn get_thresholds(
    State(_state): State<AppState>,
) -> Result<Json<RiskThresholds>, AppError> {
    info!("GET /api/risk/thresholds - Retrieving risk thresholds");

    // TODO: Fetch from database when risk_thresholds table is created
    // For now, return default thresholds
    Ok(Json(RiskThresholds::default()))
}

/// POST /api/risk/thresholds
///
/// Set user-configured risk warning thresholds.
///
/// Request body: SetThresholdsRequest containing RiskThresholds
pub async fn set_thresholds(
    State(_state): State<AppState>,
    Json(request): Json<SetThresholdsRequest>,
) -> Result<StatusCode, AppError> {
    info!("POST /api/risk/thresholds - Setting risk thresholds");

    // TODO: Save to database when risk_thresholds table is created
    // For now, just log and return success
    info!("Would save thresholds: {:?}", request.thresholds);

    Ok(StatusCode::OK)
}

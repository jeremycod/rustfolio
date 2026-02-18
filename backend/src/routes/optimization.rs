use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::OptimizationAnalysis;
use crate::services::optimization_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id", get(get_portfolio_optimization))
}

/// GET /api/optimization/portfolios/:portfolio_id
///
/// Analyze portfolio and return optimization recommendations
///
/// Example: GET /api/optimization/portfolios/{uuid}
#[axum::debug_handler]
pub async fn get_portfolio_optimization(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<OptimizationAnalysis>, AppError> {
    info!(
        "GET /api/optimization/portfolios/{} - Analyzing portfolio for optimization",
        portfolio_id
    );

    let analysis = optimization_service::analyze_portfolio(
        &state.pool,
        portfolio_id,
        state.price_provider.as_ref(),
        &state.failure_cache,
        &state.rate_limiter,
        state.risk_free_rate,
    )
    .await
    .map_err(|e| {
        error!("Failed to analyze portfolio {}: {}", portfolio_id, e);
        e
    })?;

    info!(
        "Successfully analyzed portfolio {} - {} recommendations generated",
        portfolio_id,
        analysis.recommendations.len()
    );

    Ok(Json(analysis))
}

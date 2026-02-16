use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;
use crate::errors::AppError;
use crate::models::{ForecastMethod, PortfolioForecast};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:portfolio_id", get(get_analytics))
        .route("/:portfolio_id/forecast", get(get_portfolio_forecast))
}

#[derive(Debug, Deserialize)]
struct ForecastQuery {
    days: Option<i32>,
    method: Option<String>,
}

async fn get_analytics(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::AnalyticsResponse>, AppError> {
    services::analytics_service::get_analytics(&state.pool, portfolio_id)
        .await
        .map(Json)
}

async fn get_portfolio_forecast(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<ForecastQuery>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioForecast>, AppError> {
    let days_ahead = params.days.unwrap_or(30).min(7300); // Cap at 20 years (7300 days)

    let method = params.method.as_ref().and_then(|m| match m.as_str() {
        "linear_regression" => Some(ForecastMethod::LinearRegression),
        "exponential_smoothing" => Some(ForecastMethod::ExponentialSmoothing),
        "moving_average" => Some(ForecastMethod::MovingAverage),
        "ensemble" => Some(ForecastMethod::Ensemble),
        _ => None,
    });

    // Use benchmark-based forecasting (uses current holdings + benchmark price history)
    // This approach works even without historical portfolio snapshots
    services::forecasting_service::generate_benchmark_based_forecast(
        &state.pool,
        portfolio_id,
        days_ahead,
        method,
        state.price_provider.as_ref(),
        &state.failure_cache,
    )
    .await
    .map(Json)
}
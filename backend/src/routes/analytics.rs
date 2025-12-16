use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use uuid::Uuid;
use crate::errors::AppError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/:portfolio_id", get(get_analytics))
}

async fn get_analytics(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::AnalyticsResponse>, AppError> {
    services::analytics_service::get_analytics(&state.pool, portfolio_id)
        .await
        .map(Json)
}
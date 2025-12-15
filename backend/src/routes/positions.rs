use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, error};
use crate::errors::AppError;
use crate::models::{Position, CreatePosition, UpdatePosition};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()

        .route("/:id", get(get_position))
        .route("/:id", put(create_position))
        .route("/:id", delete(delete_position))
}

pub async fn create_position(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreatePosition>
) -> Result<Json<Position>, AppError>{
    info!("PUT /positions/{} - Creating position", portfolio_id);
    let position = services::position_service::create(&state.pool, portfolio_id, payload).await
        .map_err(|e| {
            error!("Failed to create position for portfolio {}: {}", portfolio_id, e);
            e
        })?;
    Ok(Json(position))
}

pub async fn get_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<Position>, AppError>{
    info!("GET /positions/{} - Getting position", position_id);
    let position = services::position_service::fetch_one(&state.pool, position_id).await
        .map_err(|e| {
            error!("Failed to get position {}: {}", position_id, e);
            e
        })?;
    Ok(Json(position))
}

pub async fn delete_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
) -> Result<StatusCode, AppError>{
    info!("DELETE /positions/{} - Deleting position", position_id);
    services::position_service::delete(&state.pool, position_id).await
        .map_err(|e| {
            error!("Failed to delete position {}: {}", position_id, e);
            e
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_position(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(input): Json<UpdatePosition>,
) -> Result<Json<Position>, AppError> {
    info!("PUT /positions/{} - Updating position", id);
    let updated = services::position_service::update(&state.pool, id, input).await
        .map_err(|e| {
            error!("Failed to update position {}: {}", id, e);
            e
        })?;
    Ok(Json(updated))
}

pub async fn list_positions(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>
) -> Result<Json<Vec<Position>>, AppError> {
    info!("GET /positions - Listing positions for portfolio {}", portfolio_id);
    let positions = services::position_service::list(&state.pool, portfolio_id).await
        .map_err(|e| {
            error!("Failed to list positions for portfolio {}: {}", portfolio_id, e);
            e
        })?;
    Ok(Json(positions))
}


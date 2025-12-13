use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::AppError;
use crate::models::{Position, CreatePosition, UpdatePosition};
use crate::services;

pub fn router() -> Router<PgPool> {
    Router::new()

        .route("/:id", get(get_position))
        .route("/:id", put(create_position))
        .route("/:id", delete(delete_position))
}

pub async fn create_position(
    Path(portfolio_id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePosition>
) -> Result<Json<Position>, AppError>{
    let position = services::position_service::create(&pool, portfolio_id, payload).await?;
    Ok(Json(position))
}

pub async fn get_position(
    State(pool): State<PgPool>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<Position>, AppError>{
    let position = services::position_service::fetch_one(&pool, position_id).await?;
    Ok(Json(position))
}

pub async fn delete_position(
    State(pool): State<PgPool>,
    Path(position_id): Path<Uuid>,
) -> Result<StatusCode, AppError>{
    services::position_service::delete(&pool, position_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_position(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(input): Json<UpdatePosition>,
) -> Result<Json<Position>, AppError> {
    let updated = services::position_service::update(&pool, id, input).await?;
    Ok(Json(updated))
}


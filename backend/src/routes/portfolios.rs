use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{db, services};

use crate::errors::AppError;
use crate::models::{CreatePortfolio, Portfolio, UpdatePortfolio};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_portfolio).get(fetch_portfolios))
        .route("/:id", get(get_portfolio))
        .route("/:id", put(update_portfolio))
        .route("/:id", delete(delete_portfolio))
}

#[axum::debug_handler]
pub async fn create_portfolio(
    State(pool): State<PgPool>,
    Json(data): Json<CreatePortfolio>
) -> Result<Json<Portfolio>, AppError> {
    let portfolio = services::portfolio_service::create(&pool, data).await?;
    Ok(Json(portfolio))

}

pub async fn fetch_portfolios(
    State(pool): State<PgPool>
) -> Result<Json<Vec<Portfolio>>, AppError> {
    let portfolios = services::portfolio_service::fetch_all(&pool).await?;
    Ok(Json(portfolios))
}

pub async fn get_portfolio(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>
) -> Result<Json<Portfolio>, AppError> {
    let portfolio = services::portfolio_service::fetch_one(&pool, id)
        .await?;
    Ok(Json(portfolio))
}

pub async fn update_portfolio(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePortfolio>
) -> Result<Json<Portfolio>, AppError> {
    let portfolio = services::portfolio_service::update(&pool, id, data).await?;
    Ok(Json(portfolio))
}

pub async fn delete_portfolio(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>
) -> Result<Json<()>, AppError> {
    match services::portfolio_service::delete(&pool, id).await {
        Ok(0) => Err(AppError::NotFound), // 0 rows affected = not found
        Ok(_) => Ok(Json(())),            // Success
        Err(e) => Err(e),                 // Propagate other AppErrors
    }
}

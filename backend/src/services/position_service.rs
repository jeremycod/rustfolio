use sqlx::PgPool;
use uuid::Uuid;
use crate::db;
use crate::errors::AppError;
use crate::models::{CreatePortfolio, CreatePosition, Portfolio, Position, UpdatePortfolio, UpdatePosition};

pub async fn create(
    pool: &PgPool,
    portfolio_id: Uuid,
    input: CreatePosition,
) -> Result<Position, AppError> {
    if input.ticker.trim().is_empty() {
        return Err(AppError::Validation("Ticker cannot be empty".into()));
    }
    if input.shares <= 0.0 {
        return Err(AppError::Validation("Shares must be > 0".into()));
    }
    if input.avg_buy_price <= 0.0 {
        return Err(AppError::Validation("Average price must be > 0".into()));
    }

    // ensure portfolio exists
    let exists = db::portfolio_queries::exists(pool, portfolio_id).await?;
    if !exists {
        return Err(AppError::NotFound);
    }
    match db::position_queries::create(pool, portfolio_id, input).await {
        Ok(position) => Ok(position),
        Err(e) => Err(AppError::Db(e)),
    }

}
pub async fn list(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<Position>, AppError> {
    todo!()
}

pub(crate) async fn fetch_one(pool: &PgPool, id: Uuid) -> Result<Position, AppError> {
    db::position_queries::fetch_one(pool, id).await?
        .ok_or(AppError::NotFound)
}

pub(crate) async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, AppError>{
    match db::position_queries::delete(pool, id).await {
        Ok(0) => Err(AppError::NotFound),
        Ok(_) => Ok(1),
        Err(e) => Err(AppError::from(e)),
    }

}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: UpdatePosition,
) -> Result<Position, AppError> {

    if input.shares < 0.0 {
        return Err(AppError::Validation("Shares cannot be negative".into()));
    }

    db::position_queries::update(pool, id, input)
        .await?
        .ok_or(AppError::NotFound)
}
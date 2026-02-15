use sqlx::PgPool;
use uuid::Uuid;
use crate::db;
use crate::errors::AppError;
use crate::models::{CreatePortfolio, Portfolio, UpdatePortfolio};

pub async fn create(
    pool: &PgPool,
    input: CreatePortfolio,
) -> Result<Portfolio, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Portfolio name cannot be empty".into()));
    }
    let new_portfolio = Portfolio::new(input.name);
    let portfolio = db::portfolio_queries::insert(pool, new_portfolio).await?;
        Ok(portfolio)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: UpdatePortfolio,
) -> Result<Portfolio, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Portfolio name cannot be empty".into()));
    }
    let portfolio = db::portfolio_queries::update(pool, id, input).await?
        .ok_or(AppError::NotFound("Portfolio not found".to_string()))?;
    Ok(portfolio)

}

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Portfolio>, AppError> {
    let portfolios = db::portfolio_queries::fetch_all(pool).await?;
    Ok(portfolios)
}

pub(crate) async fn fetch_one(pool: &PgPool, id: Uuid) -> Result<Portfolio, AppError> {
    let portfolio = db::portfolio_queries::fetch_one(pool, id).await?
        .ok_or(AppError::NotFound("Portfolio not found".to_string()))?;
    Ok(portfolio)
}

pub(crate) async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, AppError>{
    match db::portfolio_queries::delete(pool, id).await {
        Ok(0) => Err(AppError::NotFound("Portfolio not found".to_string())),
        Ok(_) => Ok(1),
        Err(e) => Err(AppError::from(e)),
    }

}
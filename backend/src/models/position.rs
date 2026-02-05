use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents the current holdings of a particular stock within a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePosition {
    pub ticker: String,
    pub shares: BigDecimal,
    pub avg_buy_price: BigDecimal
}

#[derive(Debug, Deserialize)]
pub struct UpdatePosition {
    pub shares: BigDecimal,
    pub avg_buy_price: BigDecimal
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Position {
    id: uuid::Uuid,
    portfolio_id: uuid::Uuid,
    ticker: String,
    shares: BigDecimal,
    avg_buy_price: BigDecimal,
    created_at: chrono::DateTime<chrono::Utc>
}

impl Position {
    fn new(portfolio_id: uuid::Uuid, ticker: String, shares: BigDecimal, avg_buy_price: BigDecimal) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            portfolio_id,
            ticker,
            shares,
            avg_buy_price,
            created_at: chrono::Utc::now()
        }
    }
}
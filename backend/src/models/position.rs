use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents the current holdings of a particular stock within a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePosition {
    pub ticker: String,
    pub shares: f64,
    pub avg_buy_price: f64
}

#[derive(Debug, Deserialize)]
pub struct UpdatePosition {
    pub shares: f64,
    pub avg_buy_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Position {
    id: uuid::Uuid,
    portfolio_id: uuid::Uuid,
    ticker: String,
    shares: f64,
    avg_buy_price: f64,
    created_at: chrono::DateTime<chrono::Utc>
}

impl Position {
    fn new(portfolio_id: uuid::Uuid, ticker: String, shares: f64, avg_buy_price: f64) -> Self {
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
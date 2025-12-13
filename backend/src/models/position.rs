use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents the current holdings of a particular stock within a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Position {
    id: uuid::Uuid,
    portfolio_id: uuid::Uuid,
    ticker: String,
    shares: i32,
    avg_buy_price: f32,
    created_at: chrono::DateTime<chrono::Utc>
}

impl Position {
    fn new(portfolio_id: uuid::Uuid, ticker: String, shares: i32, avg_buy_price: f32) -> Self {
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
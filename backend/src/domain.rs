use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tokio::join;

// Represents a logical grouping of investments (e.g., "Long-term", "Retirement", "Speculative").
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Portfolio {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>
}

impl Portfolio {
    fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            created_at: chrono::Utc::now()
        }
    }
}

// Represents the current holdings of a particular stock within a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Position {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Side {
    Buy,
    Sell
}

// Represents a buy or sell event that affects a portfolio's holdings.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Transaction {
    id: uuid::Uuid,
    portfolio_id: uuid::Uuid,
    ticker: String,
    quantity: i32,
    price: f32,
    side: Side,
    executed_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>
}
impl Transaction {
    fn new(portfolio_id: uuid::Uuid, ticker: String, quantity: i32, price: f32, side: Side) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            portfolio_id,
            ticker,
            quantity,
            price,
            side,
            executed_at: chrono::Utc::now(),
            created_at: chrono::Utc::now()
        }
    }
}

// Represents a historical price for a given ticker.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct PricePoint {
    id: uuid::Uuid,
    ticker: String,
    date: chrono::DateTime<chrono::Utc>,
    close_price: f32,
    timestamp: chrono::DateTime<chrono::Utc>
}

impl PricePoint {
    fn new(ticker: String, date: chrono::DateTime<chrono::Utc>, close_price: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ticker,
            date,
            close_price,
            timestamp: chrono::Utc::now()
        }
    }
}

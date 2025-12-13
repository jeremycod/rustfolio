use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
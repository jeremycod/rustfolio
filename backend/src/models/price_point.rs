use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

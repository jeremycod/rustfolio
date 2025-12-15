use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Represents a historical price for a given ticker.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PricePoint {
    pub id: Uuid,
    pub ticker: String,
    pub date: NaiveDate,        // ✅ DATE
    pub close_price: f64,       // see section 4 below
    pub created_at: DateTime<Utc>, // ✅ TIMESTAMPTZ
}

impl PricePoint {
    fn new(ticker: String, date: NaiveDate, close_price: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ticker,
            date,
            close_price,
            created_at: chrono::Utc::now()
        }
    }
}

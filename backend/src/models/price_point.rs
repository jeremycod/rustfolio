use bigdecimal::BigDecimal;
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
    pub close_price: BigDecimal,
    pub created_at: DateTime<Utc>, // ✅ TIMESTAMPTZ
}

impl PricePoint {
    #[allow(dead_code)]
    fn new(ticker: String, date: NaiveDate, close_price: BigDecimal) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ticker,
            date,
            close_price,
            created_at: chrono::Utc::now()
        }
    }
}

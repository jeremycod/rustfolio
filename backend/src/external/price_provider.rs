use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ExternalPricePoint {
    pub date: NaiveDate,
    pub close: BigDecimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalTickerMatch {
    pub symbol: String,
    pub name: String,
    pub _type: String,
    pub region: String,
    pub currency: String,
    pub matchScore: f64,
}

#[derive(Debug, Error)]
pub enum PriceProviderError {
    #[error("network error: {0}")]
    Network(String),

    #[error("bad response: {0}")]
    BadResponse(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("rate limited")]
    RateLimited,

    #[error("ticker not found")]
    NotFound,
}

#[async_trait]
pub trait PriceProvider: Send + Sync {
    async fn fetch_daily_history(
        &self,
        ticker: &str,
        days: u32,
    ) -> Result<Vec<ExternalPricePoint>, PriceProviderError>;

    async fn search_ticker_by_keyword(
        &self,
        keyword: &str
    ) -> Result<Vec<ExternalTickerMatch>, PriceProviderError>;
}

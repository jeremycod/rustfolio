use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// SEC Filing types we track
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "filing_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FilingType {
    #[sqlx(rename = "8-k")]
    #[serde(rename = "8-k")]
    EightK,
    #[sqlx(rename = "form4")]
    Form4,
    #[sqlx(rename = "10-q")]
    #[serde(rename = "10-q")]
    TenQ,
    #[sqlx(rename = "10-k")]
    #[serde(rename = "10-k")]
    TenK,
}

/// Raw SEC filing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFiling {
    pub ticker: String,
    pub filing_type: FilingType,
    pub filing_date: NaiveDate,
    pub accession_number: String,
    pub filing_url: String,
    pub description: Option<String>,
}

/// Event importance classification
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "event_importance", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EventImportance {
    Critical,
    High,
    Medium,
    Low,
}

/// 8-K material event analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialEvent {
    pub ticker: String,
    pub event_date: NaiveDate,
    pub event_type: String,
    pub sentiment_score: f64,
    pub summary: String,
    pub importance: EventImportance,
    pub filing_url: String,
}

/// Transaction type for Form 4 (insider trades)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InsiderTransactionType {
    Purchase,
    Sale,
    Grant,
    Exercise,
}

/// Insider trading transaction (Form 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTransaction {
    pub ticker: String,
    pub transaction_date: NaiveDate,
    pub reporting_person: String,
    pub title: Option<String>,
    pub transaction_type: InsiderTransactionType,
    pub shares: i64,
    pub price_per_share: Option<BigDecimal>,
    pub ownership_after: Option<i64>,
}

/// Insider confidence level
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "insider_confidence", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InsiderConfidence {
    High,
    Medium,
    Low,
    None,
}

/// Aggregated insider sentiment for a ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderSentiment {
    pub ticker: String,
    pub period_days: i32,
    pub net_shares_traded: i64,
    pub total_transactions: i32,
    pub buying_transactions: i32,
    pub selling_transactions: i32,
    pub sentiment_score: f64,
    pub confidence: InsiderConfidence,
    pub notable_transactions: Vec<InsiderTransaction>,
}

/// Overall confidence level for combined sentiment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "confidence_level", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    VeryHigh,
    High,
    Medium,
    Low,
}

/// Enhanced sentiment combining all sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSentimentSignal {
    pub ticker: String,

    // Original news sentiment
    pub news_sentiment: f64,
    pub news_confidence: String,

    // SEC Edgar intelligence
    pub material_events: Vec<MaterialEvent>,
    pub sec_filing_score: Option<f64>,

    // Insider activity
    pub insider_sentiment: InsiderSentiment,

    // Combined analysis
    pub combined_sentiment: f64,
    pub confidence_level: ConfidenceLevel,
    pub divergence_flags: Vec<String>,

    pub calculated_at: DateTime<Utc>,
}

impl Default for InsiderSentiment {
    fn default() -> Self {
        Self {
            ticker: String::new(),
            period_days: 30,
            net_shares_traded: 0,
            total_transactions: 0,
            buying_transactions: 0,
            selling_transactions: 0,
            sentiment_score: 0.0,
            confidence: InsiderConfidence::None,
            notable_transactions: Vec::new(),
        }
    }
}

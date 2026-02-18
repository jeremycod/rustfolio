use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskSnapshot {
    pub id: Uuid,
    pub portfolio_id: Uuid,
    pub ticker: Option<String>,
    pub snapshot_date: NaiveDate,
    pub snapshot_type: String,
    pub volatility: BigDecimal,
    pub max_drawdown: BigDecimal,
    pub beta: Option<BigDecimal>,
    pub sharpe: Option<BigDecimal>,
    pub value_at_risk: Option<BigDecimal>,
    pub risk_score: BigDecimal,
    pub risk_level: String,
    pub total_value: Option<BigDecimal>,
    pub market_value: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRiskSnapshot {
    pub portfolio_id: Uuid,
    pub ticker: Option<String>,
    pub snapshot_date: NaiveDate,
    pub snapshot_type: String,
    pub volatility: BigDecimal,
    pub max_drawdown: BigDecimal,
    pub beta: Option<BigDecimal>,
    pub sharpe: Option<BigDecimal>,
    pub value_at_risk: Option<BigDecimal>,
    pub risk_score: BigDecimal,
    pub risk_level: String,
    pub total_value: Option<BigDecimal>,
    pub market_value: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RiskAlert {
    pub portfolio_id: String,
    pub ticker: Option<String>,
    pub alert_type: String,  // "risk_increase", "threshold_breach"
    pub previous_value: f64,
    pub current_value: f64,
    pub change_percent: f64,
    pub date: NaiveDate,
    pub metric_name: String,  // "risk_score", "volatility", etc.
}

#[derive(Debug, Deserialize)]
pub struct RiskHistoryParams {
    #[serde(default = "default_days")]
    pub days: i64,
    pub ticker: Option<String>,
}

fn default_days() -> i64 {
    90
}

#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    #[serde(default = "default_alert_days")]
    pub days: i64,
    #[serde(default = "default_threshold")]
    pub threshold: f64,  // Percentage change threshold (default 20%)
}

fn default_alert_days() -> i64 {
    30
}

fn default_threshold() -> f64 {
    20.0
}

#[derive(Debug, Clone, Copy)]
pub enum Aggregation {
    Daily,
    #[allow(dead_code)]
    Weekly,
    #[allow(dead_code)]
    Monthly,
}

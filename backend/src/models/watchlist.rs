use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==============================================================================
// Watchlist Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Watchlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWatchlistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWatchlistRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==============================================================================
// Watchlist Item Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchlistItem {
    pub id: Uuid,
    pub watchlist_id: Uuid,
    pub ticker: String,
    pub notes: Option<String>,
    pub added_price: Option<BigDecimal>,
    pub target_price: Option<BigDecimal>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWatchlistItemRequest {
    pub ticker: String,
    pub notes: Option<String>,
    pub target_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWatchlistItemRequest {
    pub notes: Option<String>,
    pub target_price: Option<f64>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItemResponse {
    pub id: Uuid,
    pub watchlist_id: Uuid,
    #[serde(rename = "symbol")]
    pub ticker: String,
    pub company_name: Option<String>,
    pub notes: Option<String>,
    pub added_price: Option<f64>,
    pub target_price: Option<f64>,
    pub current_price: Option<f64>,
    pub price_change_pct: Option<f64>,
    pub sort_order: i32,
    pub custom_thresholds: Option<serde_json::Value>,
    pub risk_level: Option<String>,
    pub thresholds: Vec<WatchlistThresholdResponse>,
    #[serde(rename = "added_at")]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==============================================================================
// Watchlist Threshold Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchlistThreshold {
    pub id: Uuid,
    pub watchlist_item_id: Uuid,
    pub threshold_type: String,
    pub comparison: String,
    pub value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetThresholdRequest {
    pub threshold_type: ThresholdType,
    pub comparison: ThresholdComparison,
    pub value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistThresholdResponse {
    pub id: Uuid,
    pub threshold_type: String,
    pub comparison: String,
    pub value: f64,
    pub enabled: bool,
}

impl From<WatchlistThreshold> for WatchlistThresholdResponse {
    fn from(t: WatchlistThreshold) -> Self {
        Self {
            id: t.id,
            threshold_type: t.threshold_type,
            comparison: t.comparison,
            value: t.value,
            enabled: t.enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdType {
    #[serde(rename = "price_above")]
    PriceAbove,
    #[serde(rename = "price_below")]
    PriceBelow,
    #[serde(rename = "price_change_pct")]
    PriceChangePct,
    #[serde(rename = "volatility")]
    Volatility,
    #[serde(rename = "volume_spike")]
    VolumeSpike,
    #[serde(rename = "rsi_overbought")]
    RsiOverbought,
    #[serde(rename = "rsi_oversold")]
    RsiOversold,
}

impl ThresholdType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThresholdType::PriceAbove => "price_above",
            ThresholdType::PriceBelow => "price_below",
            ThresholdType::PriceChangePct => "price_change_pct",
            ThresholdType::Volatility => "volatility",
            ThresholdType::VolumeSpike => "volume_spike",
            ThresholdType::RsiOverbought => "rsi_overbought",
            ThresholdType::RsiOversold => "rsi_oversold",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdComparison {
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "lte")]
    LessThanOrEqual,
}

impl ThresholdComparison {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThresholdComparison::GreaterThan => "gt",
            ThresholdComparison::GreaterThanOrEqual => "gte",
            ThresholdComparison::LessThan => "lt",
            ThresholdComparison::LessThanOrEqual => "lte",
        }
    }

    pub fn evaluate(&self, actual: f64, threshold: f64) -> bool {
        match self {
            ThresholdComparison::GreaterThan => actual > threshold,
            ThresholdComparison::GreaterThanOrEqual => actual >= threshold,
            ThresholdComparison::LessThan => actual < threshold,
            ThresholdComparison::LessThanOrEqual => actual <= threshold,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gt" => Some(ThresholdComparison::GreaterThan),
            "gte" => Some(ThresholdComparison::GreaterThanOrEqual),
            "lt" => Some(ThresholdComparison::LessThan),
            "lte" => Some(ThresholdComparison::LessThanOrEqual),
            _ => None,
        }
    }
}

// ==============================================================================
// Watchlist Alert Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchlistAlert {
    pub id: Uuid,
    pub watchlist_item_id: Uuid,
    pub user_id: Uuid,
    pub ticker: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub actual_value: Option<f64>,
    pub threshold_value: Option<f64>,
    pub metadata: serde_json::Value,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistAlertResponse {
    pub id: Uuid,
    pub ticker: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub actual_value: Option<f64>,
    pub threshold_value: Option<f64>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

impl From<WatchlistAlert> for WatchlistAlertResponse {
    fn from(a: WatchlistAlert) -> Self {
        Self {
            id: a.id,
            ticker: a.ticker,
            alert_type: a.alert_type,
            severity: a.severity,
            message: a.message,
            actual_value: a.actual_value,
            threshold_value: a.threshold_value,
            read: a.read,
            created_at: a.created_at,
        }
    }
}

// ==============================================================================
// Monitoring State Model
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchlistMonitoringState {
    pub watchlist_item_id: Uuid,
    pub last_checked_at: DateTime<Utc>,
    pub last_price: Option<f64>,
    pub last_rsi: Option<f64>,
    pub last_volume_ratio: Option<f64>,
    pub last_volatility: Option<f64>,
    pub last_sentiment_score: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

// ==============================================================================
// Watchlist with Items Response (combined)
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub items: Vec<WatchlistItemResponse>,
    pub recent_alerts: Vec<WatchlistAlertResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==============================================================================
// Monitoring Detection Types
// ==============================================================================

#[derive(Debug, Clone)]
pub struct MonitoringResult {
    pub ticker: String,
    pub watchlist_item_id: Uuid,
    pub user_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub actual_value: f64,
    pub threshold_value: Option<f64>,
    pub metadata: serde_json::Value,
}

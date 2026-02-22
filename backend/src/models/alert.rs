use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==============================================================================
// Alert Rule Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub portfolio_id: Option<Uuid>,
    pub ticker: Option<String>,
    pub rule_type: String,
    pub threshold: f64,
    pub comparison: String,
    pub enabled: bool,
    pub name: String,
    pub description: Option<String>,
    pub notification_channels: Vec<String>,
    pub cooldown_hours: i32,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub portfolio_id: Option<Uuid>,
    pub ticker: Option<String>,
    pub rule_type: AlertType,
    pub threshold: f64,
    pub comparison: Comparison,
    pub name: String,
    pub description: Option<String>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub cooldown_hours: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub threshold: Option<f64>,
    pub comparison: Option<Comparison>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub cooldown_hours: Option<i32>,
}

// ==============================================================================
// Alert Types
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum AlertType {
    #[serde(rename = "price_change")]
    PriceChange {
        percentage: f64,
        direction: Direction,
        timeframe: Timeframe,
    },
    #[serde(rename = "volatility_spike")]
    VolatilitySpike {
        threshold: f64,
    },
    #[serde(rename = "drawdown_exceeded")]
    DrawdownExceeded {
        percentage: f64,
    },
    #[serde(rename = "risk_threshold")]
    RiskThreshold {
        metric: RiskMetric,
        threshold: f64,
    },
    #[serde(rename = "sentiment_change")]
    SentimentChange {
        sentiment_threshold: f64,
        trend: Option<SentimentTrend>,
    },
    #[serde(rename = "divergence")]
    Divergence {
        divergence_type: DivergenceType,
    },
}

impl AlertType {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            AlertType::PriceChange { .. } => "price_change".to_string(),
            AlertType::VolatilitySpike { .. } => "volatility_spike".to_string(),
            AlertType::DrawdownExceeded { .. } => "drawdown_exceeded".to_string(),
            AlertType::RiskThreshold { .. } => "risk_threshold".to_string(),
            AlertType::SentimentChange { .. } => "sentiment_change".to_string(),
            AlertType::Divergence { .. } => "divergence".to_string(),
        }
    }
}

// ==============================================================================
// Alert Enums
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Comparison {
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lte")]
    LessThanOrEqual,
    #[serde(rename = "eq")]
    Equal,
}

impl ToString for Comparison {
    fn to_string(&self) -> String {
        match self {
            Comparison::GreaterThan => "gt".to_string(),
            Comparison::LessThan => "lt".to_string(),
            Comparison::GreaterThanOrEqual => "gte".to_string(),
            Comparison::LessThanOrEqual => "lte".to_string(),
            Comparison::Equal => "eq".to_string(),
        }
    }
}

impl Comparison {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gt" => Some(Comparison::GreaterThan),
            "lt" => Some(Comparison::LessThan),
            "gte" => Some(Comparison::GreaterThanOrEqual),
            "lte" => Some(Comparison::LessThanOrEqual),
            "eq" => Some(Comparison::Equal),
            _ => None,
        }
    }

    pub fn evaluate(&self, actual: f64, threshold: f64) -> bool {
        match self {
            Comparison::GreaterThan => actual > threshold,
            Comparison::LessThan => actual < threshold,
            Comparison::GreaterThanOrEqual => actual >= threshold,
            Comparison::LessThanOrEqual => actual <= threshold,
            Comparison::Equal => (actual - threshold).abs() < f64::EPSILON,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "either")]
    Either,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Timeframe {
    #[serde(rename = "intraday")]
    Intraday,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "monthly")]
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskMetric {
    #[serde(rename = "risk_score")]
    RiskScore,
    #[serde(rename = "volatility")]
    Volatility,
    #[serde(rename = "sharpe")]
    Sharpe,
    #[serde(rename = "sortino")]
    Sortino,
    #[serde(rename = "var_95")]
    Var95,
    #[serde(rename = "var_99")]
    Var99,
    #[serde(rename = "expected_shortfall")]
    ExpectedShortfall,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "drawdown")]
    Drawdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "in_app")]
    InApp,
    #[serde(rename = "webhook")]
    Webhook,
}

impl ToString for NotificationChannel {
    fn to_string(&self) -> String {
        match self {
            NotificationChannel::Email => "email".to_string(),
            NotificationChannel::InApp => "in_app".to_string(),
            NotificationChannel::Webhook => "webhook".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentimentTrend {
    #[serde(rename = "improving")]
    Improving,
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "deteriorating")]
    Deteriorating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DivergenceType {
    #[serde(rename = "bullish_divergence")]
    BullishDivergence,
    #[serde(rename = "bearish_divergence")]
    BearishDivergence,
    #[serde(rename = "insider_selling")]
    InsiderSelling,
    #[serde(rename = "insider_buying")]
    InsiderBuying,
}

// ==============================================================================
// Alert History Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertHistory {
    pub id: Uuid,
    pub alert_rule_id: Uuid,
    pub user_id: Uuid,
    pub portfolio_id: Option<Uuid>,
    pub ticker: Option<String>,
    pub rule_type: String,
    pub threshold: f64,
    pub actual_value: f64,
    pub message: String,
    pub severity: String,
    pub triggered_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AlertHistoryResponse {
    pub id: Uuid,
    pub alert_rule_id: Uuid,
    pub rule_name: String,
    pub portfolio_id: Option<Uuid>,
    pub ticker: Option<String>,
    pub rule_type: String,
    pub threshold: f64,
    pub actual_value: f64,
    pub message: String,
    pub severity: AlertSeverity,
    pub triggered_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

// ==============================================================================
// Alert Evaluation Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvaluationResult {
    pub rule_id: Uuid,
    pub triggered: bool,
    pub actual_value: f64,
    pub threshold: f64,
    pub message: String,
    pub severity: AlertSeverity,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl ToString for AlertSeverity {
    fn to_string(&self) -> String {
        match self {
            AlertSeverity::Low => "low".to_string(),
            AlertSeverity::Medium => "medium".to_string(),
            AlertSeverity::High => "high".to_string(),
            AlertSeverity::Critical => "critical".to_string(),
        }
    }
}

impl AlertSeverity {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(AlertSeverity::Low),
            "medium" => Some(AlertSeverity::Medium),
            "high" => Some(AlertSeverity::High),
            "critical" => Some(AlertSeverity::Critical),
            _ => None,
        }
    }
}

// ==============================================================================
// Notification Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub alert_history_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub alert_history_id: Option<Uuid>,
    pub link: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum NotificationType {
    #[serde(rename = "alert")]
    Alert,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "success")]
    Success,
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        match self {
            NotificationType::Alert => "alert".to_string(),
            NotificationType::Info => "info".to_string(),
            NotificationType::Warning => "warning".to_string(),
            NotificationType::Error => "error".to_string(),
            NotificationType::Success => "success".to_string(),
        }
    }
}

// ==============================================================================
// Notification Preferences Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub webhook_enabled: bool,
    pub webhook_url: Option<String>,
    pub quiet_hours_start: Option<NaiveTime>,
    pub quiet_hours_end: Option<NaiveTime>,
    pub timezone: String,
    pub max_daily_emails: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub email_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub webhook_enabled: Option<bool>,
    pub webhook_url: Option<String>,
    pub quiet_hours_start: Option<String>, // "HH:MM" format
    pub quiet_hours_end: Option<String>,   // "HH:MM" format
    pub timezone: Option<String>,
    pub max_daily_emails: Option<i32>,
}

// ==============================================================================
// User Model (minimal)
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==============================================================================
// Response Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleResponse {
    pub id: Uuid,
    pub portfolio_id: Option<Uuid>,
    pub ticker: Option<String>,
    pub rule_type: String,
    pub threshold: f64,
    pub comparison: String,
    pub enabled: bool,
    pub name: String,
    pub description: Option<String>,
    pub notification_channels: Vec<String>,
    pub cooldown_hours: i32,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AlertRule> for AlertRuleResponse {
    fn from(rule: AlertRule) -> Self {
        Self {
            id: rule.id,
            portfolio_id: rule.portfolio_id,
            ticker: rule.ticker,
            rule_type: rule.rule_type,
            threshold: rule.threshold,
            comparison: rule.comparison,
            enabled: rule.enabled,
            name: rule.name,
            description: rule.description,
            notification_channels: rule.notification_channels,
            cooldown_hours: rule.cooldown_hours,
            last_triggered_at: rule.last_triggered_at,
            created_at: rule.created_at,
            updated_at: rule.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCountResponse {
    pub total: i64,
    pub unread: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvaluationResponse {
    pub evaluated_rules: usize,
    pub triggered_alerts: usize,
    pub results: Vec<AlertEvaluationResult>,
}

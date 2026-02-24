use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;
use uuid::Uuid;

/// Type of trading signal based on analysis method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SignalType {
    /// Momentum-based signals (RSI, MACD, price momentum)
    #[serde(rename = "momentum")]
    Momentum,

    /// Mean reversion signals (Bollinger bands, oversold/overbought)
    #[serde(rename = "mean_reversion")]
    MeanReversion,

    /// Trend-following signals (SMA crossovers, EMA alignment)
    #[serde(rename = "trend")]
    Trend,

    /// Combined multi-factor signals
    #[serde(rename = "combined")]
    Combined,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Momentum => write!(f, "momentum"),
            SignalType::MeanReversion => write!(f, "mean_reversion"),
            SignalType::Trend => write!(f, "trend"),
            SignalType::Combined => write!(f, "combined"),
        }
    }
}

/// Direction of the trading signal
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SignalDirection {
    /// Bullish signal - expect price increase
    #[serde(rename = "bullish")]
    Bullish,

    /// Bearish signal - expect price decrease
    #[serde(rename = "bearish")]
    Bearish,

    /// Neutral signal - no clear direction
    #[serde(rename = "neutral")]
    Neutral,
}

impl std::fmt::Display for SignalDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalDirection::Bullish => write!(f, "bullish"),
            SignalDirection::Bearish => write!(f, "bearish"),
            SignalDirection::Neutral => write!(f, "neutral"),
        }
    }
}

/// Confidence level of the signal
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ConfidenceLevel {
    /// High confidence (probability >= 0.7)
    #[serde(rename = "high")]
    High,

    /// Medium confidence (0.55 <= probability < 0.7)
    #[serde(rename = "medium")]
    Medium,

    /// Low confidence (probability < 0.55)
    #[serde(rename = "low")]
    Low,
}

impl ConfidenceLevel {
    /// Determine confidence level from probability score
    pub fn from_probability(probability: f64) -> Self {
        if probability >= 0.7 {
            ConfidenceLevel::High
        } else if probability >= 0.55 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High => write!(f, "high"),
            ConfidenceLevel::Medium => write!(f, "medium"),
            ConfidenceLevel::Low => write!(f, "low"),
        }
    }
}

/// Individual factor contributing to a trading signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalFactor {
    /// Name of the indicator (e.g., "RSI", "MACD", "SMA_50")
    pub indicator: String,

    /// Current value of the indicator
    pub value: f64,

    /// Contribution weight to overall signal (0.0 to 1.0)
    pub weight: f64,

    /// Direction this factor contributes (bullish, bearish, neutral)
    pub direction: SignalDirection,

    /// Human-readable interpretation
    pub interpretation: String,
}

/// Collection of factors contributing to a signal
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalFactors {
    /// List of individual factors
    pub factors: Vec<SignalFactor>,

    /// Overall bullish score (0.0 to 1.0)
    pub bullish_score: f64,

    /// Overall bearish score (0.0 to 1.0)
    pub bearish_score: f64,

    /// Number of factors analyzed
    pub total_factors: i32,
}

impl SignalFactors {
    /// Calculate net signal direction and probability
    pub fn calculate_signal(&self) -> (SignalDirection, f64) {
        let net_score = self.bullish_score - self.bearish_score;

        let direction = if net_score > 0.1 {
            SignalDirection::Bullish
        } else if net_score < -0.1 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        // Probability is the strength of the signal (0.5 = neutral, 1.0 = very strong)
        let probability = 0.5 + (net_score.abs() * 0.5);
        let probability = probability.min(1.0).max(0.0);

        (direction, probability)
    }
}

/// Trading signal for a stock symbol
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TradingSignal {
    /// Unique identifier
    pub id: Uuid,

    /// Stock ticker symbol
    pub ticker: String,

    /// Type of signal
    #[sqlx(try_from = "String")]
    pub signal_type: SignalType,

    /// Time horizon in months (1, 3, 6, 12)
    pub horizon_months: i32,

    /// Probability score (0.0 to 1.0)
    pub probability: f64,

    /// Signal direction
    #[sqlx(try_from = "String")]
    pub direction: SignalDirection,

    /// Confidence level
    #[sqlx(try_from = "String")]
    pub confidence_level: ConfidenceLevel,

    /// Contributing factors as JSON
    pub contributing_factors: Json<SignalFactors>,

    /// Human-readable explanation
    pub explanation: String,

    /// When the signal was generated
    pub generated_at: DateTime<Utc>,

    /// When the signal expires (for caching)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request to generate trading signals
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SignalRequest {
    /// Stock ticker symbol
    pub ticker: String,

    /// Time horizon in months (default: 3)
    #[serde(default = "default_horizon")]
    pub horizon: i32,

    /// Filter by signal types (optional)
    pub signal_types: Option<Vec<SignalType>>,

    /// Minimum probability threshold (0.0 to 1.0, default: 0.5)
    #[serde(default = "default_min_probability")]
    pub min_probability: f64,
}

#[allow(dead_code)]
fn default_horizon() -> i32 {
    3
}

#[allow(dead_code)]
fn default_min_probability() -> f64 {
    0.5
}

/// Response containing generated signals
#[derive(Debug, Clone, Serialize)]
pub struct SignalResponse {
    /// Stock ticker symbol
    pub ticker: String,

    /// Current price (if available)
    pub current_price: Option<f64>,

    /// List of generated signals
    pub signals: Vec<TradingSignal>,

    /// Overall recommendation (strongest signal)
    pub recommendation: Option<SignalDirection>,

    /// Overall confidence
    pub confidence: Option<ConfidenceLevel>,

    /// When the analysis was performed
    pub analyzed_at: DateTime<Utc>,
}

/// Parameters for multi-horizon signal generation
#[derive(Debug, Clone)]
pub struct SignalGenerationParams {
    /// Stock ticker symbol
    pub ticker: String,

    /// Historical prices (most recent first)
    pub prices: Vec<f64>,

    /// Historical volumes (most recent first)
    pub volumes: Vec<f64>,

    /// Time horizons to analyze (in months)
    pub horizons: Vec<i32>,

    /// Current price
    pub current_price: f64,
}

// Implement TryFrom for custom enum types to work with sqlx
impl TryFrom<String> for SignalType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "momentum" => Ok(SignalType::Momentum),
            "mean_reversion" => Ok(SignalType::MeanReversion),
            "trend" => Ok(SignalType::Trend),
            "combined" => Ok(SignalType::Combined),
            _ => Err(format!("Unknown signal type: {}", value)),
        }
    }
}

impl TryFrom<String> for SignalDirection {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "bullish" => Ok(SignalDirection::Bullish),
            "bearish" => Ok(SignalDirection::Bearish),
            "neutral" => Ok(SignalDirection::Neutral),
            _ => Err(format!("Unknown signal direction: {}", value)),
        }
    }
}

impl TryFrom<String> for ConfidenceLevel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "high" => Ok(ConfidenceLevel::High),
            "medium" => Ok(ConfidenceLevel::Medium),
            "low" => Ok(ConfidenceLevel::Low),
            _ => Err(format!("Unknown confidence level: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_from_probability() {
        assert_eq!(ConfidenceLevel::from_probability(0.8), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_probability(0.7), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_probability(0.65), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_probability(0.55), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_probability(0.5), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_probability(0.3), ConfidenceLevel::Low);
    }

    #[test]
    fn test_signal_factors_calculate_signal() {
        let factors = SignalFactors {
            factors: vec![],
            bullish_score: 0.7,
            bearish_score: 0.2,
            total_factors: 3,
        };

        let (direction, probability) = factors.calculate_signal();
        assert_eq!(direction, SignalDirection::Bullish);
        assert!(probability > 0.7);

        let factors = SignalFactors {
            factors: vec![],
            bullish_score: 0.2,
            bearish_score: 0.7,
            total_factors: 3,
        };

        let (direction, probability) = factors.calculate_signal();
        assert_eq!(direction, SignalDirection::Bearish);
        assert!(probability > 0.7);

        let factors = SignalFactors {
            factors: vec![],
            bullish_score: 0.5,
            bearish_score: 0.5,
            total_factors: 3,
        };

        let (direction, probability) = factors.calculate_signal();
        assert_eq!(direction, SignalDirection::Neutral);
        assert!((probability - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_signal_type_conversions() {
        assert_eq!(SignalType::try_from("momentum".to_string()).unwrap(), SignalType::Momentum);
        assert_eq!(SignalType::try_from("mean_reversion".to_string()).unwrap(), SignalType::MeanReversion);
        assert_eq!(SignalType::try_from("trend".to_string()).unwrap(), SignalType::Trend);
        assert_eq!(SignalType::try_from("combined".to_string()).unwrap(), SignalType::Combined);
        assert!(SignalType::try_from("invalid".to_string()).is_err());
    }

    #[test]
    fn test_signal_direction_conversions() {
        assert_eq!(SignalDirection::try_from("bullish".to_string()).unwrap(), SignalDirection::Bullish);
        assert_eq!(SignalDirection::try_from("bearish".to_string()).unwrap(), SignalDirection::Bearish);
        assert_eq!(SignalDirection::try_from("neutral".to_string()).unwrap(), SignalDirection::Neutral);
        assert!(SignalDirection::try_from("invalid".to_string()).is_err());
    }
}

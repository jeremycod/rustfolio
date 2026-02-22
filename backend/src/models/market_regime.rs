use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Market regime classification for adaptive risk management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketRegime {
    pub id: Uuid,
    pub date: NaiveDate,
    pub regime_type: String,
    pub volatility_level: BigDecimal,
    pub market_return: Option<BigDecimal>,
    pub confidence: BigDecimal,
    pub benchmark_ticker: String,
    pub lookback_days: i32,
    pub threshold_multiplier: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new market regime entry
#[derive(Debug, Deserialize)]
pub struct CreateMarketRegime {
    pub date: NaiveDate,
    pub regime_type: String,
    pub volatility_level: BigDecimal,
    pub market_return: Option<BigDecimal>,
    pub confidence: BigDecimal,
    pub benchmark_ticker: String,
    pub lookback_days: i32,
    pub threshold_multiplier: BigDecimal,
}

/// Market regime type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegimeType {
    Bull,
    Bear,
    HighVolatility,
    Normal,
}

impl RegimeType {
    /// Convert regime type to string for database storage
    pub fn to_string(&self) -> String {
        match self {
            RegimeType::Bull => "bull".to_string(),
            RegimeType::Bear => "bear".to_string(),
            RegimeType::HighVolatility => "high_volatility".to_string(),
            RegimeType::Normal => "normal".to_string(),
        }
    }

    /// Parse regime type from string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bull" => RegimeType::Bull,
            "bear" => RegimeType::Bear,
            "high_volatility" => RegimeType::HighVolatility,
            _ => RegimeType::Normal,
        }
    }

    /// Get the threshold multiplier for this regime type
    /// - Bull: 0.8x (tighter thresholds, more sensitive to risk)
    /// - Bear: 1.3x (looser thresholds, reduce noise)
    /// - High Volatility: 1.5x (much looser, avoid excessive alerts)
    /// - Normal: 1.0x (standard thresholds)
    pub fn threshold_multiplier(&self) -> f64 {
        match self {
            RegimeType::Bull => 0.8,
            RegimeType::Bear => 1.3,
            RegimeType::HighVolatility => 1.5,
            RegimeType::Normal => 1.0,
        }
    }

    /// Get human-readable description of the regime
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        match self {
            RegimeType::Bull => "Bull Market: Positive returns with low volatility",
            RegimeType::Bear => "Bear Market: Negative returns with elevated volatility",
            RegimeType::HighVolatility => "High Volatility: Extreme market movements",
            RegimeType::Normal => "Normal Market: Stable trading conditions",
        }
    }
}

/// Regime detection parameters
#[derive(Debug, Clone)]
pub struct RegimeDetectionParams {
    /// Benchmark ticker to analyze (default: SPY)
    pub benchmark_ticker: String,
    /// Lookback period in days (default: 30)
    pub lookback_days: i64,
    /// Bull market volatility threshold (default: 20%)
    pub bull_volatility_threshold: f64,
    /// Bear market volatility threshold (default: 25%)
    pub bear_volatility_threshold: f64,
    /// High volatility threshold (default: 35%)
    pub high_volatility_threshold: f64,
}

impl Default for RegimeDetectionParams {
    fn default() -> Self {
        Self {
            benchmark_ticker: "SPY".to_string(),
            lookback_days: 30,
            bull_volatility_threshold: 20.0,
            bear_volatility_threshold: 25.0,
            high_volatility_threshold: 35.0,
        }
    }
}

/// Query parameters for regime history endpoint
#[derive(Debug, Deserialize)]
pub struct RegimeHistoryParams {
    #[serde(default = "default_history_days")]
    pub days: i64,
}

fn default_history_days() -> i64 {
    90
}

/// Current regime with adjusted thresholds
#[derive(Debug, Serialize)]
pub struct CurrentRegimeWithThresholds {
    pub regime: MarketRegime,
    pub adjusted_thresholds: AdjustedThresholds,
}

/// Adjusted risk thresholds based on market regime
#[derive(Debug, Serialize)]
pub struct AdjustedThresholds {
    pub multiplier: f64,
    pub description: String,
    pub example_volatility_warning: f64,  // Example: 30.0 * multiplier
    pub example_volatility_critical: f64, // Example: 50.0 * multiplier
    pub example_drawdown_warning: f64,    // Example: -20.0 * multiplier
    pub example_drawdown_critical: f64,   // Example: -35.0 * multiplier
}

impl AdjustedThresholds {
    /// Create adjusted thresholds from a regime type
    pub fn from_regime_type(regime_type: &RegimeType) -> Self {
        let multiplier = regime_type.threshold_multiplier();
        let description = match regime_type {
            RegimeType::Bull => "Stricter thresholds to catch early risk signals".to_string(),
            RegimeType::Bear => "Relaxed thresholds to reduce noise during volatility".to_string(),
            RegimeType::HighVolatility => "Significantly relaxed to avoid alert fatigue".to_string(),
            RegimeType::Normal => "Standard thresholds for normal market conditions".to_string(),
        };

        Self {
            multiplier,
            description,
            example_volatility_warning: 30.0 * multiplier,
            example_volatility_critical: 50.0 * multiplier,
            example_drawdown_warning: -20.0 * multiplier,
            example_drawdown_critical: -35.0 * multiplier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_type_string_conversion() {
        assert_eq!(RegimeType::Bull.to_string(), "bull");
        assert_eq!(RegimeType::Bear.to_string(), "bear");
        assert_eq!(RegimeType::HighVolatility.to_string(), "high_volatility");
        assert_eq!(RegimeType::Normal.to_string(), "normal");

        assert_eq!(RegimeType::from_string("bull"), RegimeType::Bull);
        assert_eq!(RegimeType::from_string("BEAR"), RegimeType::Bear);
        assert_eq!(RegimeType::from_string("high_volatility"), RegimeType::HighVolatility);
        assert_eq!(RegimeType::from_string("unknown"), RegimeType::Normal);
    }

    #[test]
    fn test_threshold_multipliers() {
        assert_eq!(RegimeType::Bull.threshold_multiplier(), 0.8);
        assert_eq!(RegimeType::Bear.threshold_multiplier(), 1.3);
        assert_eq!(RegimeType::HighVolatility.threshold_multiplier(), 1.5);
        assert_eq!(RegimeType::Normal.threshold_multiplier(), 1.0);
    }

    #[test]
    fn test_adjusted_thresholds() {
        let bull_thresholds = AdjustedThresholds::from_regime_type(&RegimeType::Bull);
        assert_eq!(bull_thresholds.multiplier, 0.8);
        assert_eq!(bull_thresholds.example_volatility_warning, 24.0); // 30 * 0.8
        assert_eq!(bull_thresholds.example_volatility_critical, 40.0); // 50 * 0.8

        let bear_thresholds = AdjustedThresholds::from_regime_type(&RegimeType::Bear);
        assert_eq!(bear_thresholds.multiplier, 1.3);
        assert_eq!(bear_thresholds.example_volatility_warning, 39.0); // 30 * 1.3

        let high_vol_thresholds = AdjustedThresholds::from_regime_type(&RegimeType::HighVolatility);
        assert_eq!(high_vol_thresholds.multiplier, 1.5);
        assert_eq!(high_vol_thresholds.example_volatility_warning, 45.0); // 30 * 1.5
    }

    #[test]
    fn test_default_detection_params() {
        let params = RegimeDetectionParams::default();
        assert_eq!(params.benchmark_ticker, "SPY");
        assert_eq!(params.lookback_days, 30);
        assert_eq!(params.bull_volatility_threshold, 20.0);
        assert_eq!(params.bear_volatility_threshold, 25.0);
        assert_eq!(params.high_volatility_threshold, 35.0);
    }
}

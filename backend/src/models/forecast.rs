use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Single point in a forecast time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub date: String,
    pub predicted_value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f64, // e.g., 0.95 for 95%
}

/// Complete portfolio value forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioForecast {
    pub portfolio_id: String,
    pub current_value: f64,
    pub forecast_points: Vec<ForecastPoint>,
    pub methodology: ForecastMethod,
    pub confidence_level: f64,
    pub warnings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Forecasting methodology used
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastMethod {
    LinearRegression,
    ExponentialSmoothing,
    MovingAverage,
    Ensemble, // Combination of multiple methods
}

impl ForecastMethod {
    pub fn description(&self) -> &'static str {
        match self {
            ForecastMethod::LinearRegression => {
                "Linear trend extrapolation based on historical performance"
            }
            ForecastMethod::ExponentialSmoothing => {
                "Exponential smoothing with trend and seasonality"
            }
            ForecastMethod::MovingAverage => {
                "Simple moving average projection"
            }
            ForecastMethod::Ensemble => {
                "Weighted average of multiple forecasting methods"
            }
        }
    }
}

/// Historical data point for forecasting
#[derive(Debug, Clone)]
pub struct HistoricalDataPoint {
    pub date: String,
    pub value: f64,
}

/// Single point in a beta forecast time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaForecastPoint {
    pub date: String,
    pub predicted_beta: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f64,
}

/// Beta regime change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaRegimeChange {
    pub date: String,
    pub beta_before: f64,
    pub beta_after: f64,
    pub z_score: f64,
    pub regime_type: String, // "mean_reversion", "structural_break", "high_volatility"
}

/// Complete beta forecast for a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaForecast {
    pub ticker: String,
    pub benchmark: String,
    pub current_beta: f64,
    pub beta_volatility: f64,
    pub forecast_points: Vec<BetaForecastPoint>,
    pub methodology: ForecastMethod,
    pub confidence_level: f64,
    pub regime_changes: Vec<BetaRegimeChange>,
    pub warnings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

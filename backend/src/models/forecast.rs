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
    #[allow(dead_code)]
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

/// Sentiment factors that affect forecasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentFactors {
    pub news_sentiment: f64,
    pub sec_filing_sentiment: Option<f64>,
    pub insider_sentiment: f64,
    pub combined_sentiment: f64,
    pub sentiment_momentum: f64,
    pub spike_detected: bool,
    pub divergence_detected: bool,
}

/// Sentiment-aware forecast that includes base forecast + sentiment adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAwareForecast {
    pub ticker: String,
    pub base_forecast: Vec<ForecastPoint>,
    pub sentiment_adjusted_forecast: Vec<ForecastPoint>,
    pub sentiment_factors: SentimentFactors,
    pub divergence_flags: Vec<String>,
    pub reversal_probability: f64,
    pub confidence_adjustment: f64,
    pub methodology: String,
    pub generated_at: DateTime<Utc>,
}

/// Single point in a volatility forecast time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityForecastPoint {
    pub date: String,
    pub predicted_volatility: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f64,
}

/// GARCH(1,1) model parameters
///
/// The GARCH(1,1) model: σ²ₜ = ω + α·ε²ₜ₋₁ + β·σ²ₜ₋₁
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarchParameters {
    /// Constant term (ω) - long-run variance component
    pub omega: f64,
    /// ARCH coefficient (α) - weight on recent shocks
    pub alpha: f64,
    /// GARCH coefficient (β) - weight on past variance
    pub beta: f64,
    /// Long-run unconditional variance: ω / (1 - α - β)
    pub long_run_variance: f64,
    /// Persistence parameter: α + β (measures volatility clustering)
    pub persistence: f64,
}

/// Complete volatility forecast using GARCH model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityForecast {
    pub ticker: String,
    pub current_volatility: f64, // Annualized percentage
    pub forecast_points: Vec<VolatilityForecastPoint>,
    pub garch_parameters: GarchParameters,
    pub confidence_level: f64,
    pub warnings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

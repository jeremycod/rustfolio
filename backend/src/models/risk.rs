use serde::{Deserialize, Serialize};

/// Risk metrics for a single position.
///
/// All percentage values are expressed in percent (e.g., 10.5 for 10.5%).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRisk {
    /// Annualized volatility (standard deviation of daily returns), as a percentage
    pub volatility: f64,

    /// Maximum peak-to-trough decline, as a negative percentage
    pub max_drawdown: f64,

    /// Beta coefficient relative to benchmark (correlation scaled to variance)
    pub beta: Option<f64>,

    /// Annualized Sharpe ratio (risk-adjusted return)
    pub sharpe: Option<f64>,

    /// 5% Value at Risk (1-day horizon), as a negative percentage
    pub value_at_risk: Option<f64>,
}

/// A comprehensive risk assessment including metrics and score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub ticker: String,
    pub metrics: PositionRisk,

    /// Risk score from 0-100, where higher values indicate higher risk
    pub risk_score: f64,

    /// Risk level classification
    pub risk_level: RiskLevel,
}

/// Risk level classification based on score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
}

impl RiskLevel {
    pub fn from_score(score: f64) -> Self {
        if score < 40.0 {
            RiskLevel::Low
        } else if score < 70.0 {
            RiskLevel::Moderate
        } else {
            RiskLevel::High
        }
    }
}

/// User-defined thresholds for risk warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    /// Volatility threshold percentage (e.g., 30.0 for 30%)
    pub volatility_threshold: Option<f64>,

    /// Maximum drawdown threshold as negative percentage (e.g., -15.0 for -15%)
    pub drawdown_threshold: Option<f64>,

    /// Beta threshold (e.g., 1.5)
    pub beta_threshold: Option<f64>,

    /// VaR threshold as negative percentage
    pub var_threshold: Option<f64>,

    /// Overall risk score threshold (0-100)
    pub risk_score_threshold: Option<f64>,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            volatility_threshold: Some(30.0),
            drawdown_threshold: Some(-15.0),
            beta_threshold: Some(1.5),
            var_threshold: Some(-5.0),
            risk_score_threshold: Some(70.0),
        }
    }
}

/// Portfolio-level aggregated risk metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRisk {
    pub portfolio_id: String,

    /// Weighted average volatility across positions
    pub portfolio_volatility: f64,

    /// Portfolio beta (weighted average)
    pub portfolio_beta: Option<f64>,

    /// Overall portfolio risk score
    pub portfolio_risk_score: f64,

    /// Risk level classification
    pub risk_level: RiskLevel,

    /// Individual position risk assessments
    pub position_risks: Vec<PositionRisk>,
}

/// Request body for setting custom risk thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetThresholdsRequest {
    pub thresholds: RiskThresholds,
}

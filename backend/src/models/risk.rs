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

    /// Annualized Sortino ratio (downside risk-adjusted return)
    pub sortino: Option<f64>,

    /// Annualized return (mean return extrapolated to one year), as a percentage
    pub annualized_return: Option<f64>,

    /// 5% Value at Risk (1-day horizon), as a negative percentage
    /// Kept for backward compatibility - equivalent to var_95
    pub value_at_risk: Option<f64>,

    /// 95% confidence VaR (1-day horizon), as a negative percentage
    /// 5% chance of losing more than this in a single day
    pub var_95: Option<f64>,

    /// 99% confidence VaR (1-day horizon), as a negative percentage
    /// 1% chance of losing more than this in a single day
    pub var_99: Option<f64>,

    /// Expected Shortfall at 95% confidence (CVaR), as a negative percentage
    /// Average loss when the 95% VaR threshold is exceeded
    pub expected_shortfall_95: Option<f64>,

    /// Expected Shortfall at 99% confidence (CVaR), as a negative percentage
    /// Average loss when the 99% VaR threshold is exceeded
    pub expected_shortfall_99: Option<f64>,
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

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Moderate => write!(f, "moderate"),
            RiskLevel::High => write!(f, "high"),
        }
    }
}

/// Portfolio-level aggregated risk metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRisk {
    pub portfolio_id: String,

    /// Total portfolio market value
    pub total_value: f64,

    /// Weighted average volatility across positions
    pub portfolio_volatility: f64,

    /// Maximum drawdown across portfolio
    pub portfolio_max_drawdown: f64,

    /// Portfolio beta (weighted average)
    pub portfolio_beta: Option<f64>,

    /// Portfolio Sharpe ratio (weighted average)
    pub portfolio_sharpe: Option<f64>,

    /// Overall portfolio risk score
    pub portfolio_risk_score: f64,

    /// Risk level classification
    pub risk_level: RiskLevel,

    /// Individual position risk contributions
    pub position_risks: Vec<PositionRiskContribution>,
}

/// Individual position's contribution to portfolio risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRiskContribution {
    pub ticker: String,
    pub market_value: f64,
    pub weight: f64, // Position weight in portfolio (0-1)
    pub risk_assessment: RiskAssessment,
}


/// Correlation pair between two tickers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub ticker1: String,
    pub ticker2: String,
    /// Correlation coefficient (-1.0 to 1.0)
    /// +1.0 = perfect positive correlation
    ///  0.0 = no correlation
    /// -1.0 = perfect negative correlation
    pub correlation: f64,
}

/// Complete correlation matrix for a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub portfolio_id: String,
    /// List of all tickers in the portfolio
    pub tickers: Vec<String>,
    /// Correlation pairs (only upper triangle, excluding diagonal)
    pub correlations: Vec<CorrelationPair>,
}

/// Risk threshold settings stored in database (per portfolio).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RiskThresholdSettings {
    pub id: String,
    pub portfolio_id: String,

    // Volatility thresholds
    pub volatility_warning_threshold: f64,
    pub volatility_critical_threshold: f64,

    // Drawdown thresholds
    pub drawdown_warning_threshold: f64,
    pub drawdown_critical_threshold: f64,

    // Beta thresholds
    pub beta_warning_threshold: f64,
    pub beta_critical_threshold: f64,

    // Risk score thresholds
    pub risk_score_warning_threshold: f64,
    pub risk_score_critical_threshold: f64,

    // VaR thresholds
    pub var_warning_threshold: f64,
    pub var_critical_threshold: f64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to update risk threshold settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRiskThresholds {
    pub volatility_warning_threshold: Option<f64>,
    pub volatility_critical_threshold: Option<f64>,
    pub drawdown_warning_threshold: Option<f64>,
    pub drawdown_critical_threshold: Option<f64>,
    pub beta_warning_threshold: Option<f64>,
    pub beta_critical_threshold: Option<f64>,
    pub risk_score_warning_threshold: Option<f64>,
    pub risk_score_critical_threshold: Option<f64>,
    pub var_warning_threshold: Option<f64>,
    pub var_critical_threshold: Option<f64>,
}

/// Severity level for threshold violations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    Warning,
    Critical,
}

/// A position that violates risk thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub threshold_type: ViolationSeverity,
}

/// Enhanced portfolio risk response with threshold violations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskWithViolations {
    #[serde(flatten)]
    pub portfolio_risk: PortfolioRisk,
    pub thresholds: RiskThresholdSettings,
    pub violations: Vec<ThresholdViolation>,
}


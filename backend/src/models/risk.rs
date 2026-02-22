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

    /// Beta coefficient relative to SPY benchmark (correlation scaled to variance)
    /// Kept for backward compatibility
    pub beta: Option<f64>,

    /// Multi-benchmark beta analysis (optional, computed on demand)
    pub beta_spy: Option<f64>,
    pub beta_qqq: Option<f64>,  // Nasdaq 100
    pub beta_iwm: Option<f64>,  // Russell 2000

    /// Risk decomposition (optional, computed on demand)
    pub risk_decomposition: Option<RiskDecomposition>,

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

    /// Portfolio VaR at 95% confidence (weighted average)
    pub portfolio_var_95: Option<f64>,

    /// Portfolio VaR at 99% confidence (weighted average)
    pub portfolio_var_99: Option<f64>,

    /// Portfolio Expected Shortfall at 95% confidence (weighted average)
    pub portfolio_expected_shortfall_95: Option<f64>,

    /// Portfolio Expected Shortfall at 99% confidence (weighted average)
    pub portfolio_expected_shortfall_99: Option<f64>,

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
    /// 2D correlation matrix for heatmap visualization
    /// matrix_2d[i][j] = correlation between tickers[i] and tickers[j]
    /// Diagonal values are 1.0 (perfect self-correlation)
    pub matrix_2d: Vec<Vec<f64>>,
    /// Clustering results (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters: Option<Vec<AssetCluster>>,
    /// Map of ticker symbol to cluster ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_labels: Option<std::collections::HashMap<String, usize>>,
    /// Correlation matrix between cluster centroids
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inter_cluster_correlations: Option<Vec<Vec<f64>>>,
}

/// A cluster of correlated assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCluster {
    /// Cluster ID (0-indexed)
    pub cluster_id: usize,
    /// Tickers in this cluster
    pub tickers: Vec<String>,
    /// Average intra-cluster correlation
    pub avg_correlation: f64,
    /// Suggested visualization color (hex)
    pub color: String,
    /// Cluster name based on dominant characteristics
    pub name: String,
}

/// Risk decomposition into systematic and idiosyncratic components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecomposition {
    /// Systematic risk (variance explained by market/beta)
    pub systematic_risk: f64,
    /// Idiosyncratic risk (stock-specific variance)
    pub idiosyncratic_risk: f64,
    /// R-squared: proportion of variance explained by market
    pub r_squared: f64,
    /// Total risk (volatility)
    pub total_risk: f64,
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

/// Portfolio-level correlation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationStatistics {
    /// Average correlation across all pairs
    pub average_correlation: f64,
    /// Maximum correlation in portfolio
    pub max_correlation: f64,
    /// Minimum correlation in portfolio
    pub min_correlation: f64,
    /// Standard deviation of correlations
    pub correlation_std_dev: f64,
    /// Number of high correlation pairs (> 0.7)
    pub high_correlation_pairs: usize,
    /// Correlation-adjusted diversification score (0-10)
    pub adjusted_diversification_score: f64,
}

/// Correlation matrix with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrixWithStats {
    #[serde(flatten)]
    pub matrix: CorrelationMatrix,
    pub statistics: CorrelationStatistics,
}

/// Single point in rolling beta time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaPoint {
    /// Date of the beta calculation
    pub date: String,
    /// Beta coefficient for this window
    pub beta: f64,
    /// R-squared (variance explained by benchmark)
    pub r_squared: f64,
    /// Alpha (excess return), optional
    pub alpha: Option<f64>,
}

/// Rolling beta analysis with multiple window sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingBetaAnalysis {
    /// Ticker symbol being analyzed
    pub ticker: String,
    /// Benchmark ticker (e.g., SPY, QQQ)
    pub benchmark: String,
    /// 30-day rolling beta time series
    pub beta_30d: Vec<BetaPoint>,
    /// 60-day rolling beta time series
    pub beta_60d: Vec<BetaPoint>,
    /// 90-day rolling beta time series
    pub beta_90d: Vec<BetaPoint>,
    /// Current beta (most recent value)
    pub current_beta: f64,
    /// Beta volatility (standard deviation of 90d beta)
    pub beta_volatility: f64,
}

/// Downside risk metrics for a position or portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownsideRiskMetrics {
    /// Downside deviation (semi-deviation), as a percentage
    /// Only considers returns below the minimum acceptable return (MAR)
    pub downside_deviation: f64,

    /// Sortino ratio (risk-adjusted return using downside deviation)
    /// Higher is better. Sortino > 2.0 is considered excellent
    pub sortino_ratio: Option<f64>,

    /// Minimum Acceptable Return (MAR) used in calculations, as a percentage
    /// Typically the risk-free rate
    pub mar: f64,

    /// Sharpe ratio for comparison
    pub sharpe_ratio: Option<f64>,

    /// Interpretation guidance
    pub interpretation: DownsideInterpretation,
}

/// Interpretation guidance for downside risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownsideInterpretation {
    /// Risk level based on downside deviation
    pub downside_risk_level: String,

    /// Sortino ratio rating (e.g., "Excellent", "Good", "Fair", "Poor")
    pub sortino_rating: String,

    /// Comparison between Sortino and Sharpe
    pub sortino_vs_sharpe: String,

    /// Summary message
    pub summary: String,
}

/// Portfolio-level downside risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioDownsideRisk {
    pub portfolio_id: String,

    /// Aggregated portfolio-level downside metrics
    pub portfolio_metrics: DownsideRiskMetrics,

    /// Individual position downside risk contributions
    pub position_downside_risks: Vec<PositionDownsideContribution>,

    /// Analysis period in days
    pub days: i64,

    /// Benchmark used
    pub benchmark: String,
}

/// Individual position's downside risk contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDownsideContribution {
    pub ticker: String,
    pub weight: f64,
    pub downside_metrics: DownsideRiskMetrics,
}


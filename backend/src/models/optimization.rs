use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Type of optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    ReduceConcentration,
    RebalanceSectors,
    ReduceRisk,
    ImproveEfficiency,
    IncreaseDiversification,
}

/// Severity level of the recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,      // FYI, no urgency
    Warning,   // Should address soon
    High,      // Address within week
    Critical,  // Address immediately
}

/// Action to take on a position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AdjustmentAction {
    Buy,
    Sell,
    Hold,
}

/// Adjustment for a specific position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAdjustment {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub current_value: f64,
    pub current_weight: f64,
    pub recommended_value: f64,
    pub recommended_weight: f64,
    pub action: AdjustmentAction,
    pub amount_change: f64,
    pub shares_change: Option<f64>,
}

/// Expected impact of implementing recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub risk_score_before: f64,
    pub risk_score_after: f64,
    pub risk_score_change: f64,
    pub volatility_before: f64,
    pub volatility_after: f64,
    pub volatility_change: f64,
    pub sharpe_before: Option<f64>,
    pub sharpe_after: Option<f64>,
    pub sharpe_change: Option<f64>,
    pub diversification_before: f64,
    pub diversification_after: f64,
    pub diversification_change: f64,
    pub max_drawdown_before: f64,
    pub max_drawdown_after: f64,
}

/// A single optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub id: String,
    pub recommendation_type: RecommendationType,
    pub severity: Severity,
    pub title: String,
    pub rationale: String,
    pub affected_positions: Vec<PositionAdjustment>,
    pub expected_impact: ExpectedImpact,
    pub suggested_actions: Vec<String>,
}

/// Complete optimization analysis for a portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAnalysis {
    pub portfolio_id: String,
    pub portfolio_name: String,
    pub total_value: f64,
    pub analysis_date: String,
    pub current_metrics: CurrentMetrics,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub summary: AnalysisSummary,
}

/// Current portfolio metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMetrics {
    pub risk_score: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: Option<f64>,
    pub diversification_score: f64,
    pub position_count: usize,
    pub largest_position_weight: f64,
    pub top_3_concentration: f64,
}

/// Summary of analysis findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_recommendations: usize,
    pub critical_issues: usize,
    pub high_priority: usize,
    pub warnings: usize,
    pub overall_health: PortfolioHealth,
    pub key_findings: Vec<String>,
}

/// Overall portfolio health rating
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PortfolioHealth {
    Excellent,  // Well-diversified, balanced risk
    Good,       // Minor issues, generally sound
    Fair,       // Some concerns, needs attention
    Poor,       // Significant issues, rebalance recommended
    Critical,   // Major problems, immediate action needed
}

/// Request to simulate portfolio changes
#[derive(Debug, Clone, Deserialize)]
pub struct SimulationRequest {
    pub adjustments: Vec<SimulationAdjustment>,
}

/// Position adjustment for simulation
#[derive(Debug, Clone, Deserialize)]
pub struct SimulationAdjustment {
    pub ticker: String,
    pub new_weight: f64,  // Target weight as percentage (0-100)
}

/// Result of portfolio simulation
#[derive(Debug, Clone, Serialize)]
pub struct SimulationResult {
    pub original_metrics: PortfolioMetrics,
    pub simulated_metrics: PortfolioMetrics,
    pub changes: MetricChanges,
    pub is_improvement: bool,
    pub warnings: Vec<String>,
}

/// Portfolio metrics for comparison
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioMetrics {
    pub risk_score: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: Option<f64>,
    pub diversification_score: f64,
    pub largest_position: f64,
}

/// Changes between original and simulated metrics
#[derive(Debug, Clone, Serialize)]
pub struct MetricChanges {
    pub risk_score_change: f64,
    pub risk_score_pct: f64,
    pub volatility_change: f64,
    pub volatility_pct: f64,
    pub diversification_change: f64,
}

/// Risk contribution of a single position
#[derive(Debug, Clone, Serialize)]
pub struct RiskContribution {
    pub ticker: String,
    pub weight: f64,
    pub volatility: f64,
    pub risk_contribution: f64,  // Percentage of total portfolio risk
    pub is_excessive: bool,       // Contributing >20% of total risk
}

/// Concentration risk analysis
#[derive(Debug, Clone, Serialize)]
pub struct ConcentrationAnalysis {
    pub largest_position: PositionConcentration,
    pub top_3_positions: Vec<PositionConcentration>,
    pub top_5_positions: Vec<PositionConcentration>,
    pub top_3_total: f64,
    pub top_5_total: f64,
    pub concentration_risk: ConcentrationRisk,
}

/// Concentration info for a position
#[derive(Debug, Clone, Serialize)]
pub struct PositionConcentration {
    pub ticker: String,
    pub weight: f64,
    pub value: f64,
}

/// Concentration risk level
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConcentrationRisk {
    Low,       // Well-diversified
    Moderate,  // Some concentration but acceptable
    High,      // Significant concentration
    Extreme,   // Dangerous concentration
}

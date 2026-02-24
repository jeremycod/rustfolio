use serde::{Deserialize, Serialize};

// ============================================================================
// Factor Types
// ============================================================================

/// The five canonical investment factors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FactorType {
    Value,
    Growth,
    Momentum,
    Quality,
    LowVolatility,
}

impl FactorType {
    pub fn label(&self) -> &'static str {
        match self {
            FactorType::Value => "Value",
            FactorType::Growth => "Growth",
            FactorType::Momentum => "Momentum",
            FactorType::Quality => "Quality",
            FactorType::LowVolatility => "Low Volatility",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FactorType::Value => "Stocks trading below intrinsic value based on fundamental ratios",
            FactorType::Growth => "Companies with above-average revenue and earnings growth",
            FactorType::Momentum => "Securities exhibiting strong recent price performance",
            FactorType::Quality => "Profitable companies with low debt and stable earnings",
            FactorType::LowVolatility => "Securities with below-average price fluctuations",
        }
    }

    pub fn all() -> Vec<FactorType> {
        vec![
            FactorType::Value,
            FactorType::Growth,
            FactorType::Momentum,
            FactorType::Quality,
            FactorType::LowVolatility,
        ]
    }
}

impl std::fmt::Display for FactorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================================
// Factor Scores
// ============================================================================

/// Scores for a single holding across all factor dimensions (0-100 each)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerFactorScores {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub weight: f64,
    pub value_score: f64,
    pub growth_score: f64,
    pub momentum_score: f64,
    pub quality_score: f64,
    pub low_volatility_score: f64,
    /// Composite score using the multi-factor combination weights
    pub composite_score: f64,
}

/// Portfolio-level factor exposure (aggregate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioFactorExposure {
    pub factor: FactorType,
    pub label: String,
    pub description: String,
    /// Weighted average score across all holdings (0-100)
    pub score: f64,
    /// Interpretation: "underweight", "neutral", "overweight"
    pub exposure_level: ExposureLevel,
    /// Historical expected risk premium (annualised, in percentage points)
    pub expected_risk_premium: f64,
    /// Recommendations specific to this factor
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExposureLevel {
    Underweight,
    Neutral,
    Overweight,
}

impl ExposureLevel {
    pub fn from_score(score: f64) -> Self {
        if score < 35.0 {
            ExposureLevel::Underweight
        } else if score > 65.0 {
            ExposureLevel::Overweight
        } else {
            ExposureLevel::Neutral
        }
    }
}

// ============================================================================
// Multi-Factor Combination
// ============================================================================

/// Weights for combining individual factors into a composite score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorWeights {
    pub value: f64,
    pub growth: f64,
    pub momentum: f64,
    pub quality: f64,
    pub low_volatility: f64,
}

impl Default for FactorWeights {
    fn default() -> Self {
        // Equal-weight default; can be overridden by optimizer
        Self {
            value: 0.20,
            growth: 0.20,
            momentum: 0.20,
            quality: 0.20,
            low_volatility: 0.20,
        }
    }
}

impl FactorWeights {
    pub fn composite(&self, scores: &TickerFactorScores) -> f64 {
        self.value * scores.value_score
            + self.growth * scores.growth_score
            + self.momentum * scores.momentum_score
            + self.quality * scores.quality_score
            + self.low_volatility * scores.low_volatility_score
    }
}

// ============================================================================
// ETF Suggestions
// ============================================================================

/// A suggested ETF for gaining factor exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorEtfSuggestion {
    pub ticker: String,
    pub name: String,
    pub factor: FactorType,
    pub expense_ratio: f64,
    /// Approximate AUM in billions USD
    pub aum_billions: f64,
    pub description: String,
}

// ============================================================================
// Back-test Results
// ============================================================================

/// Results of historical factor back-testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorBacktestResult {
    pub factor: FactorType,
    /// Annualised return of the top-quintile factor portfolio (%)
    pub annualized_return: f64,
    /// Annualised volatility (%)
    pub annualized_volatility: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Maximum drawdown (%)
    pub max_drawdown: f64,
    /// Number of trading days used
    pub observation_days: usize,
    /// Cumulative return over the period (%)
    pub cumulative_return: f64,
}

// ============================================================================
// Top-level API Response
// ============================================================================

/// Full response for GET /api/recommendations/factors/:portfolio_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorAnalysisResponse {
    pub portfolio_id: String,
    pub portfolio_name: String,
    pub analysis_date: String,
    /// Per-holding factor scores
    pub holdings_scores: Vec<TickerFactorScores>,
    /// Portfolio-level factor exposures
    pub factor_exposures: Vec<PortfolioFactorExposure>,
    /// Optimized multi-factor weights
    pub factor_weights: FactorWeights,
    /// ETF suggestions to improve factor tilts
    pub etf_suggestions: Vec<FactorEtfSuggestion>,
    /// Back-test results per factor
    pub backtest_results: Vec<FactorBacktestResult>,
    /// Summary text
    pub summary: FactorAnalysisSummary,
}

/// High-level summary of the factor analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorAnalysisSummary {
    pub dominant_factor: String,
    pub weakest_factor: String,
    pub overall_composite_score: f64,
    pub key_findings: Vec<String>,
}

/// Query parameters for the factors endpoint
#[derive(Debug, Deserialize)]
pub struct FactorQueryParams {
    /// Number of days of price history to use (default: 252 ~ 1 year)
    pub days: Option<i64>,
    /// Whether to include back-test results (default: true)
    pub include_backtest: Option<bool>,
    /// Whether to include ETF suggestions (default: true)
    pub include_etfs: Option<bool>,
}

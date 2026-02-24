use serde::{Deserialize, Serialize};

// ── Investment Goal Types ────────────────────────────────────────────

/// Investment goal for long-term guidance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InvestmentGoal {
    Retirement,
    College,
    Wealth,
}

impl std::fmt::Display for InvestmentGoal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvestmentGoal::Retirement => write!(f, "retirement"),
            InvestmentGoal::College => write!(f, "college"),
            InvestmentGoal::Wealth => write!(f, "wealth"),
        }
    }
}

impl InvestmentGoal {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "retirement" => Some(InvestmentGoal::Retirement),
            "college" => Some(InvestmentGoal::College),
            "wealth" => Some(InvestmentGoal::Wealth),
            _ => None,
        }
    }
}

/// Risk tolerance level for long-term recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTolerance {
    Conservative,
    Moderate,
    Aggressive,
}

impl RiskTolerance {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "conservative" => Some(RiskTolerance::Conservative),
            "moderate" => Some(RiskTolerance::Moderate),
            "aggressive" => Some(RiskTolerance::Aggressive),
            _ => None,
        }
    }
}

// ── Quality Scoring ──────────────────────────────────────────────────

/// Growth potential metrics derived from price history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthMetrics {
    /// Annualized return over available history
    pub annualized_return: f64,
    /// Return consistency (R-squared of log-return regression)
    pub return_consistency: f64,
    /// 1-year return if enough data
    pub return_1y: Option<f64>,
    /// 3-year annualized return if enough data
    pub return_3y: Option<f64>,
    /// Compound annual growth rate
    pub cagr: f64,
}

/// Dividend analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendMetrics {
    /// Whether this holding has positive gain/loss suggesting income generation
    pub has_positive_income: bool,
    /// Estimated yield based on gain/loss relative to book value
    pub estimated_yield: Option<f64>,
    /// Payout sustainability indicator (0-1)
    pub payout_sustainability: f64,
    /// Dividend growth estimated from price trend stability
    pub growth_indicator: f64,
}

/// Competitive advantage (moat) indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoatIndicators {
    /// Price stability relative to market (lower vol = stronger moat proxy)
    pub price_stability: f64,
    /// Margin proxy from consistent positive returns
    pub margin_strength: f64,
    /// Relative strength vs benchmark
    pub relative_strength: f64,
    /// Market presence indicator (based on available data)
    pub market_presence: f64,
}

/// Management quality proxies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementMetrics {
    /// Capital efficiency proxy (return per unit of risk)
    pub capital_efficiency: f64,
    /// Drawdown recovery speed
    pub recovery_speed: f64,
    /// Consistency of positive returns (proportion of positive months)
    pub return_consistency: f64,
}

/// Composite quality score for a holding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub industry: Option<String>,

    /// Growth potential score (0-100)
    pub growth_score: f64,
    /// Dividend/income score (0-100)
    pub dividend_score: f64,
    /// Competitive advantage score (0-100)
    pub moat_score: f64,
    /// Management quality score (0-100)
    pub management_score: f64,

    /// Overall composite quality score (0-100)
    pub composite_score: f64,
    /// Quality tier classification
    pub quality_tier: QualityTier,

    /// Detailed metrics
    pub growth_metrics: GrowthMetrics,
    pub dividend_metrics: DividendMetrics,
    pub moat_indicators: MoatIndicators,
    pub management_metrics: ManagementMetrics,
}

/// Quality tier classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QualityTier {
    Premium,
    High,
    Medium,
    Low,
}

impl QualityTier {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 {
            QualityTier::Premium
        } else if score >= 60.0 {
            QualityTier::High
        } else if score >= 40.0 {
            QualityTier::Medium
        } else {
            QualityTier::Low
        }
    }
}

impl std::fmt::Display for QualityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityTier::Premium => write!(f, "premium"),
            QualityTier::High => write!(f, "high"),
            QualityTier::Medium => write!(f, "medium"),
            QualityTier::Low => write!(f, "low"),
        }
    }
}

// ── Risk Classification ──────────────────────────────────────────────

/// Risk classification for a holding in the context of long-term investing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HoldingRiskClass {
    /// Utilities, consumer staples, bonds - low volatility
    Low,
    /// Healthcare, industrials, financials - moderate volatility
    Medium,
    /// Technology, small-cap growth, emerging markets - high volatility
    High,
}

impl HoldingRiskClass {
    pub fn from_volatility_and_industry(volatility: f64, industry: Option<&str>) -> Self {
        // Industry-based classification first
        if let Some(ind) = industry {
            let ind_lower = ind.to_lowercase();
            if ind_lower.contains("utilit") || ind_lower.contains("staple")
                || ind_lower.contains("bond") || ind_lower.contains("treasury")
                || ind_lower.contains("fixed income")
            {
                return HoldingRiskClass::Low;
            }
            if ind_lower.contains("tech") || ind_lower.contains("crypto")
                || ind_lower.contains("biotech") || ind_lower.contains("small")
            {
                return HoldingRiskClass::High;
            }
        }

        // Fall back to volatility-based classification
        if volatility < 15.0 {
            HoldingRiskClass::Low
        } else if volatility < 30.0 {
            HoldingRiskClass::Medium
        } else {
            HoldingRiskClass::High
        }
    }
}

impl std::fmt::Display for HoldingRiskClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoldingRiskClass::Low => write!(f, "low"),
            HoldingRiskClass::Medium => write!(f, "medium"),
            HoldingRiskClass::High => write!(f, "high"),
        }
    }
}

// ── Retirement & Long-Term Recommendations ──────────────────────────

/// A single long-term recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermRecommendation {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub industry: Option<String>,

    /// Quality assessment
    pub quality_score: QualityScore,

    /// Risk classification for retirement/long-term
    pub risk_class: HoldingRiskClass,

    /// Whether this qualifies as a dividend aristocrat candidate
    pub dividend_aristocrat_candidate: bool,

    /// Whether this qualifies as a blue-chip candidate
    pub blue_chip_candidate: bool,

    /// Suitability for the requested goal (0-100)
    pub goal_suitability: f64,

    /// Human-readable recommendation rationale
    pub rationale: String,

    /// Suggested allocation weight for a long-term portfolio (0-1)
    pub suggested_weight: f64,
}

/// Allocation strategy breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStrategy {
    /// Recommended allocation to low-risk holdings (0-1)
    pub low_risk_allocation: f64,
    /// Recommended allocation to medium-risk holdings (0-1)
    pub medium_risk_allocation: f64,
    /// Recommended allocation to high-risk holdings (0-1)
    pub high_risk_allocation: f64,
    /// Strategy description
    pub description: String,
}

impl AllocationStrategy {
    /// Get allocation strategy based on risk tolerance and investment horizon
    pub fn for_profile(risk_tolerance: &RiskTolerance, horizon_years: i32) -> Self {
        // Age-based / horizon-based adjustment
        let horizon_factor = if horizon_years > 20 {
            1.0 // Long horizon, can take more risk
        } else if horizon_years > 10 {
            0.7
        } else if horizon_years > 5 {
            0.4
        } else {
            0.2 // Short horizon, be conservative
        };

        match risk_tolerance {
            RiskTolerance::Conservative => AllocationStrategy {
                low_risk_allocation: 0.60 - (horizon_factor * 0.10),
                medium_risk_allocation: 0.30 + (horizon_factor * 0.05),
                high_risk_allocation: 0.10 + (horizon_factor * 0.05),
                description: format!(
                    "Conservative allocation for {}-year horizon: \
                     Prioritizes capital preservation with stable, income-generating holdings. \
                     Focus on dividend aristocrats and blue-chip stocks.",
                    horizon_years
                ),
            },
            RiskTolerance::Moderate => AllocationStrategy {
                low_risk_allocation: 0.35 - (horizon_factor * 0.10),
                medium_risk_allocation: 0.40,
                high_risk_allocation: 0.25 + (horizon_factor * 0.10),
                description: format!(
                    "Moderate allocation for {}-year horizon: \
                     Balances growth potential with risk management. \
                     Mix of quality growth stocks and stable income producers.",
                    horizon_years
                ),
            },
            RiskTolerance::Aggressive => AllocationStrategy {
                low_risk_allocation: 0.15 - (horizon_factor * 0.05),
                medium_risk_allocation: 0.35,
                high_risk_allocation: 0.50 + (horizon_factor * 0.05),
                description: format!(
                    "Aggressive allocation for {}-year horizon: \
                     Maximizes long-term growth potential. \
                     Emphasizes high-quality growth stocks with strong competitive advantages.",
                    horizon_years
                ),
            },
        }
    }
}

/// Portfolio-level long-term guidance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioGuidanceSummary {
    /// Number of holdings that qualify as dividend aristocrat candidates
    pub dividend_aristocrat_count: usize,
    /// Number of holdings that qualify as blue-chip candidates
    pub blue_chip_count: usize,
    /// Average composite quality score
    pub average_quality_score: f64,
    /// Portfolio-level diversification assessment
    pub diversification_rating: String,
    /// Current allocation breakdown by risk class
    pub current_risk_allocation: CurrentRiskAllocation,
    /// Key improvement suggestions
    pub suggestions: Vec<String>,
}

/// Current portfolio allocation by risk class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentRiskAllocation {
    pub low_risk_pct: f64,
    pub medium_risk_pct: f64,
    pub high_risk_pct: f64,
}

// ── API Response ────────────────────────────────────────────────────

/// Full response for GET /api/recommendations/long-term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermGuidanceResponse {
    pub portfolio_id: String,
    pub goal: String,
    pub risk_tolerance: String,
    pub horizon_years: i32,

    /// Recommended allocation strategy
    pub allocation_strategy: AllocationStrategy,

    /// Top recommendations sorted by goal suitability
    pub recommendations: Vec<LongTermRecommendation>,

    /// Portfolio-level summary and guidance
    pub summary: PortfolioGuidanceSummary,

    /// Timestamp of analysis
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// Query parameters for the long-term guidance endpoint
#[derive(Debug, Deserialize)]
pub struct LongTermGuidanceQuery {
    /// Investment goal: retirement, college, wealth
    pub goal: Option<String>,
    /// Investment horizon in years (default: 20)
    pub horizon: Option<i32>,
    /// Risk tolerance: conservative, moderate, aggressive
    pub risk_tolerance: Option<String>,
    /// Minimum quality score filter (0-100)
    pub min_quality: Option<f64>,
    /// Force refresh (ignore cache)
    #[serde(default)]
    pub refresh: bool,
}

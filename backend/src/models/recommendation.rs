use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// The type of narrative to generate for a recommendation explanation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NarrativeType {
    /// Value-focused: P/E, P/B, undervalued relative to fundamentals
    Valuation,
    /// Growth-focused: revenue growth, earnings momentum, expanding margins
    Growth,
    /// Risk-focused: volatility, drawdown, beta, defensive positioning
    Risk,
    /// Contrarian: against-the-crowd thesis, mean-reversion potential
    Contrarian,
    /// Dividend/Income: yield, payout ratio, dividend growth
    Dividend,
    /// Balanced: considers multiple factors equally
    Balanced,
}

impl std::fmt::Display for NarrativeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NarrativeType::Valuation => write!(f, "valuation"),
            NarrativeType::Growth => write!(f, "growth"),
            NarrativeType::Risk => write!(f, "risk"),
            NarrativeType::Contrarian => write!(f, "contrarian"),
            NarrativeType::Dividend => write!(f, "dividend"),
            NarrativeType::Balanced => write!(f, "balanced"),
        }
    }
}

impl Default for NarrativeType {
    fn default() -> Self {
        NarrativeType::Balanced
    }
}

/// Context assembled for generating a recommendation explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationContext {
    /// Stock ticker symbol
    pub symbol: String,

    /// Current price
    pub current_price: f64,

    /// Price change over recent period (percentage)
    pub price_change_pct: Option<f64>,

    /// 52-week high
    pub high_52w: Option<f64>,

    /// 52-week low
    pub low_52w: Option<f64>,

    /// Volatility (annualized)
    pub volatility: Option<f64>,

    /// Beta relative to market
    pub beta: Option<f64>,

    /// RSI (14-day)
    pub rsi_14: Option<f64>,

    /// Signal direction from trading signals
    pub signal_direction: Option<String>,

    /// Signal probability
    pub signal_probability: Option<f64>,

    /// Signal confidence level
    pub signal_confidence: Option<String>,

    /// Top contributing factors from signal analysis
    pub signal_factors: Vec<String>,

    /// Max drawdown
    pub max_drawdown: Option<f64>,

    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,

    /// The narrative type to generate
    pub narrative_type: NarrativeType,
}

/// A generated recommendation explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationExplanation {
    /// Stock ticker symbol
    pub symbol: String,

    /// Short headline (1 sentence)
    pub headline: String,

    /// Main analysis narrative (2-4 paragraphs)
    pub narrative: String,

    /// Key factors supporting the analysis (3-5 bullet points)
    pub key_factors: Vec<String>,

    /// Risk considerations (2-3 bullet points)
    pub risk_considerations: Vec<String>,

    /// The narrative type used
    pub narrative_type: NarrativeType,

    /// Overall signal direction
    pub signal_direction: Option<String>,

    /// Confidence level
    pub confidence: Option<String>,

    /// When the explanation was generated
    pub generated_at: DateTime<Utc>,

    /// When the explanation expires (for caching)
    pub expires_at: DateTime<Utc>,

    /// Disclaimer text
    pub disclaimer: String,
}

/// Cached explanation row from the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CachedExplanation {
    pub id: i64,
    pub symbol: String,
    pub explanation: String,
    pub factors_snapshot: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Query parameters for the explanation endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct ExplanationQuery {
    /// Narrative type: valuation, growth, risk, contrarian, dividend, balanced
    pub narrative_type: Option<String>,

    /// Force refresh (ignore cache)
    #[serde(default)]
    pub refresh: bool,
}

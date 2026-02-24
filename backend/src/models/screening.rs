use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// POST body for the screening endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ScreeningRequest {
    /// Tickers to screen (if empty, screen all tickers in the database)
    #[serde(default)]
    pub symbols: Vec<String>,

    /// Factor weights (all optional; missing = use defaults)
    #[serde(default)]
    pub weights: FactorWeights,

    /// Filters to narrow the universe
    #[serde(default)]
    pub filters: ScreeningFilters,

    /// Maximum results to return (default 20)
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Pagination offset (default 0)
    #[serde(default)]
    pub offset: usize,

    /// User risk appetite (conservative, moderate, aggressive) -- adjusts default weights
    pub risk_appetite: Option<RiskAppetite>,

    /// Investment horizon in months (adjusts weight emphasis)
    pub horizon_months: Option<i32>,

    /// Force a fresh calculation (skip cache)
    #[serde(default)]
    pub refresh: bool,
}

fn default_limit() -> usize {
    20
}

/// Risk appetite presets that shift factor weights.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskAppetite {
    Conservative,
    Moderate,
    Aggressive,
}

/// User-supplied factor weights (0.0 - 1.0 each).
/// Missing fields receive sensible defaults.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactorWeights {
    /// Weight for fundamental metrics (P/E, P/B, PEG, D/E, earnings growth)
    pub fundamental: Option<f64>,
    /// Weight for technical indicators (MA crossovers, RSI, relative strength, volume)
    pub technical: Option<f64>,
    /// Weight for sentiment scoring
    pub sentiment: Option<f64>,
    /// Weight for momentum factors (price momentum 1M-12M, volume momentum)
    pub momentum: Option<f64>,
}

impl Default for FactorWeights {
    fn default() -> Self {
        Self {
            fundamental: None,
            technical: None,
            sentiment: None,
            momentum: None,
        }
    }
}

impl FactorWeights {
    /// Resolve actual weights, filling in defaults based on risk appetite and horizon.
    pub fn resolve(&self, risk_appetite: Option<RiskAppetite>, horizon_months: Option<i32>) -> ResolvedWeights {
        let (def_fund, def_tech, def_sent, def_mom) = match risk_appetite {
            Some(RiskAppetite::Conservative) => (0.40, 0.20, 0.15, 0.25),
            Some(RiskAppetite::Aggressive) => (0.15, 0.30, 0.20, 0.35),
            _ => (0.30, 0.25, 0.15, 0.30), // moderate default
        };

        // Horizon adjustment: long horizon boosts fundamentals, short boosts momentum
        let horizon_factor = match horizon_months {
            Some(h) if h <= 3 => (-0.05, 0.0, 0.0, 0.05),
            Some(h) if h >= 12 => (0.05, -0.05, 0.0, 0.0),
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        let raw = [
            self.fundamental.unwrap_or(def_fund) + horizon_factor.0,
            self.technical.unwrap_or(def_tech) + horizon_factor.1,
            self.sentiment.unwrap_or(def_sent) + horizon_factor.2,
            self.momentum.unwrap_or(def_mom) + horizon_factor.3,
        ];

        // Ensure non-negative then normalise to sum = 1.0
        let positive: Vec<f64> = raw.iter().map(|w| w.max(0.0)).collect();
        let sum: f64 = positive.iter().sum();
        let norm = if sum > 0.0 { sum } else { 1.0 };

        ResolvedWeights {
            fundamental: positive[0] / norm,
            technical: positive[1] / norm,
            sentiment: positive[2] / norm,
            momentum: positive[3] / norm,
        }
    }
}

/// Normalised weights that sum to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedWeights {
    pub fundamental: f64,
    pub technical: f64,
    pub sentiment: f64,
    pub momentum: f64,
}

/// Filtering criteria applied before scoring.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScreeningFilters {
    /// Only include these sectors (e.g. "Technology", "Healthcare")
    #[serde(default)]
    pub sectors: Vec<String>,

    /// Market cap range
    pub market_cap: Option<MarketCapRange>,

    /// Minimum average daily volume (liquidity filter)
    pub min_avg_volume: Option<f64>,

    /// Price range
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,

    /// Geographic filter (e.g. "US", "EU")
    #[serde(default)]
    pub geographies: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarketCapRange {
    /// < 2 B
    Small,
    /// 2 B - 10 B
    Mid,
    /// 10 B - 200 B
    Large,
    /// > 200 B
    Mega,
}

// ---------------------------------------------------------------------------
// Factor score types
// ---------------------------------------------------------------------------

/// Scores from the fundamental metrics module.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FundamentalScore {
    pub pe_score: Option<f64>,
    pub pb_score: Option<f64>,
    pub peg_score: Option<f64>,
    pub debt_to_equity_score: Option<f64>,
    pub earnings_growth_score: Option<f64>,
    /// Composite fundamental score 0-100
    pub composite: f64,
    pub details: Vec<ScoreDetail>,
}

/// Scores from the technical indicators module.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechnicalScore {
    pub ma_crossover_score: Option<f64>,
    pub rsi_score: Option<f64>,
    pub relative_strength_score: Option<f64>,
    pub volume_score: Option<f64>,
    /// Composite technical score 0-100
    pub composite: f64,
    pub details: Vec<ScoreDetail>,
}

/// Scores from the sentiment module.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SentimentScore {
    pub news_sentiment_score: Option<f64>,
    pub sentiment_trend_score: Option<f64>,
    /// Composite sentiment score 0-100
    pub composite: f64,
    pub details: Vec<ScoreDetail>,
}

/// Scores from the momentum module.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MomentumScore {
    pub momentum_1m: Option<f64>,
    pub momentum_3m: Option<f64>,
    pub momentum_6m: Option<f64>,
    pub momentum_12m: Option<f64>,
    pub volume_momentum: Option<f64>,
    pub acceleration: Option<f64>,
    /// Composite momentum score 0-100
    pub composite: f64,
    pub details: Vec<ScoreDetail>,
}

/// Human-readable breakdown of a single sub-score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDetail {
    pub metric: String,
    pub raw_value: Option<f64>,
    pub score: f64,
    pub interpretation: String,
}

// ---------------------------------------------------------------------------
// Composite result
// ---------------------------------------------------------------------------

/// The fully scored result for a single ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningResult {
    pub symbol: String,
    pub composite_score: f64,
    pub rank: usize,
    pub fundamental: FundamentalScore,
    pub technical: TechnicalScore,
    pub sentiment: SentimentScore,
    pub momentum: MomentumScore,
    pub weights_used: ResolvedWeights,
    pub explanation: String,
}

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// Response returned by `POST /api/recommendations/screen`.
#[derive(Debug, Clone, Serialize)]
pub struct ScreeningResponse {
    pub results: Vec<ScreeningResult>,
    pub total_screened: usize,
    pub total_passed_filters: usize,
    pub weights_used: ResolvedWeights,
    pub screened_at: DateTime<Utc>,
    pub cache_hit: bool,
    /// Pagination
    pub limit: usize,
    pub offset: usize,
}

// ---------------------------------------------------------------------------
// Cache row (for DB storage)
// ---------------------------------------------------------------------------

/// Flat row stored in the `screening_cache` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningCacheEntry {
    pub id: Uuid,
    pub cache_key: String,
    pub results_json: serde_json::Value,
    pub total_screened: i32,
    pub total_passed_filters: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

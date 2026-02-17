use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sentiment trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SentimentTrend {
    Improving,    // Sentiment getting more positive
    Stable,       // No significant change
    Deteriorating, // Sentiment getting more negative
}

/// Momentum trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MomentumTrend {
    Bullish,   // Price trending up
    Neutral,   // Sideways
    Bearish,   // Price trending down
}

/// Divergence signal (sentiment vs price)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DivergenceType {
    Bullish,   // Sentiment improving while price down (potential reversal)
    Bearish,   // Sentiment deteriorating while price up (warning)
    Confirmed, // Sentiment and price aligned
    None,      // No clear divergence
}

/// Single point in sentiment time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentDataPoint {
    pub date: String,
    pub sentiment_score: f64, // -1.0 to +1.0
    pub news_volume: i32,     // Number of articles that day
    pub price: Option<f64>,   // Stock price (for correlation)
}

/// Sentiment signal for a single ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSignal {
    pub ticker: String,
    pub current_sentiment: f64,        // Latest sentiment score
    pub sentiment_trend: SentimentTrend,
    pub momentum_trend: MomentumTrend,
    pub divergence: DivergenceType,

    // Correlation analysis
    pub sentiment_price_correlation: Option<f64>, // -1 to +1
    pub correlation_lag_days: Option<i32>,        // Lead/lag relationship
    pub correlation_strength: Option<String>,     // "strong", "moderate", "weak"

    // Historical data
    pub historical_sentiment: Vec<SentimentDataPoint>,

    // Metadata
    pub news_articles_analyzed: i32,
    pub calculated_at: DateTime<Utc>,

    // Warnings
    pub warnings: Vec<String>,
}

/// Portfolio-wide sentiment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSentimentAnalysis {
    pub portfolio_id: String,
    pub signals: Vec<SentimentSignal>,
    pub portfolio_avg_sentiment: f64,
    pub bullish_divergences: i32,  // Count of bullish divergence signals
    pub bearish_divergences: i32,  // Count of bearish divergence signals
    pub calculated_at: DateTime<Utc>,
}

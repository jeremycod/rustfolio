use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single news article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub snippet: String,
}

/// Sentiment classification for news
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

impl std::fmt::Display for Sentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sentiment::Positive => write!(f, "positive"),
            Sentiment::Neutral => write!(f, "neutral"),
            Sentiment::Negative => write!(f, "negative"),
        }
    }
}

/// A clustered theme from multiple news articles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsTheme {
    pub theme_name: String,
    pub summary: String,
    pub sentiment: Sentiment,
    pub articles: Vec<NewsArticle>,
    pub relevance_score: f64,
}

/// Portfolio-wide news analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioNewsAnalysis {
    pub portfolio_id: String,
    pub themes: Vec<NewsTheme>,
    pub position_news: HashMap<String, Vec<NewsTheme>>,
    pub overall_sentiment: Sentiment,
    pub fetched_at: DateTime<Utc>,
}

/// Request parameters for fetching news
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsQueryParams {
    /// Number of days to look back (default: 7)
    pub days: Option<i32>,
    /// Filter by ticker (optional)
    pub ticker: Option<String>,
    /// Force refresh, bypassing cache (default: false)
    pub force: Option<bool>,
}

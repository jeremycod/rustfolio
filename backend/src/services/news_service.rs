use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::errors::AppError;
use crate::models::{NewsArticle, NewsTheme, Sentiment};
use crate::services::llm_service::LlmService;

/// Configuration for news service
#[derive(Debug, Clone)]
pub struct NewsConfig {
    pub enabled: bool,
    pub provider: String,
    pub api_key: Option<String>,
}

impl NewsConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("NEWS_ENABLED")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(false),
            provider: std::env::var("NEWS_PROVIDER").unwrap_or_else(|_| "serper".to_string()),
            api_key: std::env::var("NEWS_API_KEY").ok(),
        }
    }
}

/// Trait for news providers
#[async_trait]
pub trait NewsProvider: Send + Sync {
    async fn fetch_news(
        &self,
        query: &str,
        days: i32,
        max_results: usize,
    ) -> Result<Vec<NewsArticle>, AppError>;
}

/// Serper API provider (uses Google's news search)
pub struct SerperProvider {
    api_key: String,
    client: Client,
}

impl SerperProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SerperResponse {
    news: Option<Vec<SerperNewsItem>>,
}

#[derive(Debug, Deserialize)]
struct SerperNewsItem {
    title: String,
    link: String,
    source: String,
    date: String,
    snippet: String,
}

#[async_trait]
impl NewsProvider for SerperProvider {
    async fn fetch_news(
        &self,
        query: &str,
        days: i32,
        max_results: usize,
    ) -> Result<Vec<NewsArticle>, AppError> {
        info!("Fetching news from Serper for query: {}", query);

        let request_body = serde_json::json!({
            "q": query,
            "type": "news",
            "num": max_results.min(100), // Serper max is 100
        });

        let response = self
            .client
            .post("https://google.serper.dev/news")
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!("Serper API request failed: {}", e);
                AppError::External(format!("News API error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Serper API error {}: {}", status, error_text);
            return Err(AppError::External(format!(
                "News API returned error {}: {}",
                status, error_text
            )));
        }

        let serper_response: SerperResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Serper response: {}", e);
            AppError::External(format!("Failed to parse news response: {}", e))
        })?;

        let cutoff_date = Utc::now() - Duration::days(days as i64);

        let articles: Vec<NewsArticle> = serper_response
            .news
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                // Parse date - Serper returns various formats
                let published_at = parse_serper_date(&item.date)?;

                // Filter by date range
                if published_at < cutoff_date {
                    return None;
                }

                Some(NewsArticle {
                    title: item.title,
                    url: item.link,
                    source: item.source,
                    published_at,
                    snippet: item.snippet,
                })
            })
            .collect();

        info!("Fetched {} news articles from Serper", articles.len());
        Ok(articles)
    }
}

/// Parse Serper date format (e.g., "2 hours ago", "1 day ago", "Mar 15, 2024")
fn parse_serper_date(date_str: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    let lower = date_str.to_lowercase();

    // Handle relative dates
    if lower.contains("ago") {
        if let Some(hours) = extract_number(&lower, "hour") {
            return Some(now - Duration::hours(hours as i64));
        }
        if let Some(days) = extract_number(&lower, "day") {
            return Some(now - Duration::days(days as i64));
        }
        if let Some(minutes) = extract_number(&lower, "minute") {
            return Some(now - Duration::minutes(minutes as i64));
        }
    }

    // Try parsing absolute dates
    // Format: "Mar 15, 2024" or similar
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%b %d, %Y") {
        return Some(dt.and_hms_opt(0, 0, 0)?.and_utc());
    }

    // Default to now if parsing fails
    warn!("Could not parse date '{}', using current time", date_str);
    Some(now)
}

fn extract_number(text: &str, _unit: &str) -> Option<u32> {
    text.split_whitespace()
        .find_map(|word| word.parse::<u32>().ok())
}

/// Main news service
pub struct NewsService {
    config: NewsConfig,
    provider: Option<Arc<dyn NewsProvider>>,
    llm_service: Arc<LlmService>,
}

impl NewsService {
    pub fn new(config: NewsConfig, llm_service: Arc<LlmService>) -> Self {
        let provider: Option<Arc<dyn NewsProvider>> = if config.enabled {
            if let Some(api_key) = &config.api_key {
                match config.provider.as_str() {
                    "serper" => {
                        info!("Initializing Serper news provider");
                        Some(Arc::new(SerperProvider::new(api_key.clone())))
                    }
                    _ => {
                        warn!("Unknown news provider: {}", config.provider);
                        None
                    }
                }
            } else {
                warn!("News enabled but no API key provided");
                None
            }
        } else {
            info!("News service disabled");
            None
        };

        Self {
            config,
            provider,
            llm_service,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.provider.is_some()
    }

    /// Fetch news for a ticker symbol
    pub async fn fetch_ticker_news(
        &self,
        ticker: &str,
        days: i32,
    ) -> Result<Vec<NewsArticle>, AppError> {
        if !self.is_enabled() {
            return Err(AppError::External(
                "News service is not enabled".to_string(),
            ));
        }

        let provider = self.provider.as_ref().unwrap();
        let query = format!("{} stock news", ticker);
        provider.fetch_news(&query, days, 20).await
    }

    /// Cluster articles into themes using LLM
    pub async fn cluster_into_themes(
        &self,
        articles: Vec<NewsArticle>,
        user_id: uuid::Uuid,
    ) -> Result<Vec<NewsTheme>, AppError> {
        if articles.is_empty() {
            return Ok(Vec::new());
        }

        if !self.llm_service.is_enabled() {
            // Return a single "unclustered" theme without LLM
            return Ok(vec![NewsTheme {
                theme_name: "Recent News".to_string(),
                summary: format!("{} recent articles", articles.len()),
                sentiment: Sentiment::Neutral,
                relevance_score: 1.0,
                articles,
            }]);
        }

        info!("Clustering {} articles into themes", articles.len());

        // Build prompt for theme extraction
        let prompt = build_theme_clustering_prompt(&articles);

        // Get LLM response
        let response = self
            .llm_service
            .generate_completion_for_user(user_id, prompt)
            .await?;

        // Parse themes from response
        parse_theme_response(&response, articles)
    }
}

/// Build prompt for theme clustering
fn build_theme_clustering_prompt(articles: &[NewsArticle]) -> String {
    let articles_text: String = articles
        .iter()
        .enumerate()
        .map(|(i, article)| {
            format!(
                "[{}] {} - {}\nSource: {} | Date: {}\n",
                i,
                article.title,
                article.snippet,
                article.source,
                article.published_at.format("%Y-%m-%d")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Analyze these news articles and identify 2-4 major themes. For each theme, provide:
1. A concise theme name (3-5 words)
2. A 2-3 sentence summary
3. Sentiment (positive/neutral/negative)
4. Article indices that belong to this theme
5. Relevance score (0.0-1.0)

ARTICLES:
{}

Format your response as valid JSON:
{{
  "themes": [
    {{
      "theme_name": "...",
      "summary": "...",
      "sentiment": "positive|neutral|negative",
      "article_indices": [0, 2, 5],
      "relevance_score": 0.85
    }}
  ]
}}

Guidelines:
- Group related articles together
- Sentiment should reflect the overall tone (positive = good news, negative = concerning)
- Keep summaries factual and educational
- Relevance score reflects how important this theme is to investors"#,
        articles_text
    )
}

#[derive(Debug, Deserialize)]
struct ThemeResponse {
    themes: Vec<ThemeData>,
}

#[derive(Debug, Deserialize)]
struct ThemeData {
    theme_name: String,
    summary: String,
    sentiment: String,
    article_indices: Vec<usize>,
    relevance_score: f64,
}

/// Parse theme clustering response from LLM
fn parse_theme_response(
    response: &str,
    articles: Vec<NewsArticle>,
) -> Result<Vec<NewsTheme>, AppError> {
    // Try to parse JSON response
    match serde_json::from_str::<ThemeResponse>(response) {
        Ok(parsed) => {
            let themes: Vec<NewsTheme> = parsed
                .themes
                .into_iter()
                .map(|theme_data| {
                    let sentiment = match theme_data.sentiment.to_lowercase().as_str() {
                        "positive" => Sentiment::Positive,
                        "negative" => Sentiment::Negative,
                        _ => Sentiment::Neutral,
                    };

                    let theme_articles: Vec<NewsArticle> = theme_data
                        .article_indices
                        .iter()
                        .filter_map(|&idx| articles.get(idx).cloned())
                        .collect();

                    NewsTheme {
                        theme_name: theme_data.theme_name,
                        summary: theme_data.summary,
                        sentiment,
                        relevance_score: theme_data.relevance_score,
                        articles: theme_articles,
                    }
                })
                .collect();

            Ok(themes)
        }
        Err(e) => {
            warn!("Failed to parse theme response: {}", e);
            // Fallback: return all articles as a single theme
            Ok(vec![NewsTheme {
                theme_name: "Recent News".to_string(),
                summary: "Recent news and developments".to_string(),
                sentiment: Sentiment::Neutral,
                relevance_score: 1.0,
                articles,
            }])
        }
    }
}

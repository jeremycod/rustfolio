use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::get;
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;
use std::collections::HashMap;

use crate::errors::AppError;
use crate::models::{NewsQueryParams, PortfolioNewsAnalysis, NewsTheme, Sentiment};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/news", get(get_portfolio_news))
        .route("/positions/:ticker/news", get(get_ticker_news))
}

/// GET /api/news/portfolios/:portfolio_id/news
///
/// Fetch and analyze news for all positions in a portfolio
///
/// Query parameters:
/// - `days`: Number of days to look back (default: 7)
///
/// Returns themed news analysis with sentiment
async fn get_portfolio_news(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<NewsQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioNewsAnalysis>, AppError> {
    let days = params.days.unwrap_or(7);

    info!(
        "GET /api/news/portfolios/{}/news - Fetching portfolio news (days={})",
        portfolio_id, days
    );

    // 1. Get all positions in the portfolio
    let holdings = crate::db::holding_snapshot_queries::fetch_portfolio_latest_holdings(
        &state.pool,
        portfolio_id
    ).await.map_err(|e| {
        error!("Failed to fetch portfolio holdings: {}", e);
        AppError::Db(e)
    })?;

    if holdings.is_empty() {
        return Err(AppError::External(
            "Portfolio has no holdings to fetch news for".to_string()
        ));
    }

    // 2. Get unique tickers
    let mut tickers: Vec<String> = holdings
        .iter()
        .map(|h| h.ticker.clone())
        .collect();
    tickers.sort();
    tickers.dedup();

    info!("Fetching news for {} tickers", tickers.len());

    // 3. Fetch news for each ticker
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Invalid demo user UUID");

    let mut position_news: HashMap<String, Vec<NewsTheme>> = HashMap::new();
    let mut all_themes: Vec<NewsTheme> = Vec::new();

    for ticker in &tickers {
        match state.news_service.fetch_ticker_news(ticker, days).await {
            Ok(articles) if !articles.is_empty() => {
                // Cluster articles into themes
                match state.news_service.cluster_into_themes(articles, demo_user_id).await {
                    Ok(themes) => {
                        info!("Clustered {} themes for {}", themes.len(), ticker);
                        all_themes.extend(themes.clone());
                        position_news.insert(ticker.clone(), themes);
                    }
                    Err(e) => {
                        error!("Failed to cluster themes for {}: {}", ticker, e);
                    }
                }
            }
            Ok(_) => {
                info!("No news articles found for {}", ticker);
            }
            Err(e) => {
                error!("Failed to fetch news for {}: {}", ticker, e);
            }
        }
    }

    // 4. Calculate overall sentiment
    let overall_sentiment = calculate_overall_sentiment(&all_themes);

    info!(
        "Portfolio news analysis complete: {} themes, {} sentiment",
        all_themes.len(),
        overall_sentiment
    );

    Ok(Json(PortfolioNewsAnalysis {
        portfolio_id: portfolio_id.to_string(),
        themes: all_themes,
        position_news,
        overall_sentiment,
        fetched_at: chrono::Utc::now(),
    }))
}

/// GET /api/news/positions/:ticker/news
///
/// Fetch and analyze news for a specific ticker
///
/// Query parameters:
/// - `days`: Number of days to look back (default: 7)
async fn get_ticker_news(
    Path(ticker): Path<String>,
    Query(params): Query<NewsQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<Vec<NewsTheme>>, AppError> {
    let days = params.days.unwrap_or(7);

    info!(
        "GET /api/news/positions/{}/news - Fetching ticker news (days={})",
        ticker, days
    );

    // Fetch news articles
    let articles = state.news_service.fetch_ticker_news(&ticker, days).await?;

    if articles.is_empty() {
        info!("No news articles found for {}", ticker);
        return Ok(Json(Vec::new()));
    }

    // Cluster into themes
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Invalid demo user UUID");

    let themes = state
        .news_service
        .cluster_into_themes(articles, demo_user_id)
        .await?;

    info!("Clustered {} themes for {}", themes.len(), ticker);

    Ok(Json(themes))
}

/// Calculate overall portfolio sentiment from themes
fn calculate_overall_sentiment(themes: &[NewsTheme]) -> Sentiment {
    if themes.is_empty() {
        return Sentiment::Neutral;
    }

    let mut positive_count = 0;
    let mut negative_count = 0;
    let mut neutral_count = 0;

    for theme in themes {
        match theme.sentiment {
            Sentiment::Positive => positive_count += 1,
            Sentiment::Negative => negative_count += 1,
            Sentiment::Neutral => neutral_count += 1,
        }
    }

    // Weighted decision: negative sentiment has more weight
    if negative_count > positive_count {
        Sentiment::Negative
    } else if positive_count > negative_count * 2 {
        Sentiment::Positive
    } else {
        Sentiment::Neutral
    }
}

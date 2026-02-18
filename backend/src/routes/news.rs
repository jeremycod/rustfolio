use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::get;
use tracing::{error, info};
use uuid::Uuid;
use std::collections::HashMap;
use sqlx::PgPool;
use chrono::{Utc, Duration};

use crate::errors::AppError;
use crate::models::{NewsQueryParams, PortfolioNewsAnalysis, NewsTheme, Sentiment};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/news", get(get_portfolio_news))
        .route("/positions/:ticker/news", get(get_ticker_news))
}

/// Check if cached news exists and is still fresh (< 24 hours old)
async fn get_cached_news(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Option<PortfolioNewsAnalysis>, AppError> {
    let result = sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT news_data
        FROM portfolio_news_cache
        WHERE portfolio_id = $1 AND expires_at > NOW()
        "#
    )
    .bind(portfolio_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)?;

    if let Some(news_data) = result {
        info!("Found cached news for portfolio {}", portfolio_id);
        let news_analysis: PortfolioNewsAnalysis = serde_json::from_value(news_data)
            .map_err(|e| AppError::External(format!("Failed to deserialize cached news: {}", e)))?;
        Ok(Some(news_analysis))
    } else {
        info!("No valid cache found for portfolio {}", portfolio_id);
        Ok(None)
    }
}

/// Store news analysis in cache with 24-hour expiration
async fn cache_news(
    pool: &PgPool,
    portfolio_id: Uuid,
    news_analysis: &PortfolioNewsAnalysis,
) -> Result<(), AppError> {
    let news_data = serde_json::to_value(news_analysis)
        .map_err(|e| AppError::External(format!("Failed to serialize news for cache: {}", e)))?;

    let fetched_at = Utc::now();
    let expires_at = fetched_at + Duration::hours(24);

    sqlx::query(
        r#"
        INSERT INTO portfolio_news_cache (portfolio_id, news_data, fetched_at, expires_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (portfolio_id)
        DO UPDATE SET
            news_data = $2,
            fetched_at = $3,
            expires_at = $4,
            updated_at = NOW()
        "#
    )
    .bind(portfolio_id)
    .bind(news_data)
    .bind(fetched_at)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;

    info!("Cached news for portfolio {} (expires at {})", portfolio_id, expires_at);
    Ok(())
}

/// GET /api/news/portfolios/:portfolio_id/news
///
/// Fetch and analyze news for all positions in a portfolio
///
/// Query parameters:
/// - `days`: Number of days to look back (default: 7)
/// - `force`: Force refresh, bypassing cache (default: false)
///
/// Returns themed news analysis with sentiment
async fn get_portfolio_news(
    Path(portfolio_id): Path<Uuid>,
    Query(params): Query<NewsQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioNewsAnalysis>, AppError> {
    let days = params.days.unwrap_or(7);
    let force = params.force.unwrap_or(false);

    info!(
        "GET /api/news/portfolios/{}/news - Fetching portfolio news (days={}, force={})",
        portfolio_id, days, force
    );

    // Check cache first if not forcing refresh
    if !force {
        if let Some(cached_news) = get_cached_news(&state.pool, portfolio_id).await? {
            info!("Returning cached news for portfolio {}", portfolio_id);
            return Ok(Json(cached_news));
        }
    }

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

    let news_analysis = PortfolioNewsAnalysis {
        portfolio_id: portfolio_id.to_string(),
        themes: all_themes,
        position_news,
        overall_sentiment,
        fetched_at: chrono::Utc::now(),
    };

    // Cache the results for future requests
    if let Err(e) = cache_news(&state.pool, portfolio_id, &news_analysis).await {
        error!("Failed to cache news for portfolio {}: {}", portfolio_id, e);
        // Continue even if caching fails - don't fail the request
    }

    Ok(Json(news_analysis))
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
    let mut _neutral_count = 0;

    for theme in themes {
        match theme.sentiment {
            Sentiment::Positive => positive_count += 1,
            Sentiment::Negative => negative_count += 1,
            Sentiment::Neutral => _neutral_count += 1,
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

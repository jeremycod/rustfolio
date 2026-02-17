use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{SentimentSignal, PortfolioSentimentAnalysis, DivergenceType, EnhancedSentimentSignal};
use crate::services::{sentiment_service, price_service, enhanced_sentiment_service, sec_edgar_service};
use crate::state::AppState;

/// Query parameters for sentiment analysis
#[derive(Debug, Deserialize)]
pub struct SentimentQueryParams {
    /// Number of days to analyze (default: 30)
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

/// GET /api/sentiment/positions/:ticker/sentiment
/// Query params: days (default: 30)
pub async fn get_position_sentiment(
    Path(ticker): Path<String>,
    Query(params): Query<SentimentQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<SentimentSignal>, AppError> {
    info!("Fetching sentiment signal for ticker: {}", ticker);

    // Check if news service is enabled
    if !state.news_service.is_enabled() {
        return Err(AppError::External(
            "News service is not enabled. Please configure NEWS_ENABLED=true and NEWS_API_KEY to use sentiment analysis.".to_string()
        ));
    }

    // 1. Fetch news for the ticker
    let articles = state.news_service.fetch_ticker_news(&ticker, params.days).await?;

    if articles.is_empty() {
        return Err(AppError::Validation(
            format!("No news articles found for ticker {} in the last {} days", ticker, params.days)
        ));
    }

    info!("Found {} news articles for {}", articles.len(), ticker);

    // 2. Cluster articles into themes using LLM
    // Use demo user ID (same pattern as news routes)
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Failed to parse demo user UUID");

    let themes = state.news_service.cluster_into_themes(articles, demo_user_id).await?;

    if themes.is_empty() {
        return Err(AppError::Validation(
            format!("Failed to extract themes from news articles for {}", ticker)
        ));
    }

    info!("Extracted {} themes for {}", themes.len(), ticker);

    // 3. Fetch price history for correlation analysis
    let prices = price_service::get_history(&state.pool, &ticker).await?;

    if prices.is_empty() {
        return Err(AppError::Validation(
            format!("No price history available for {}. Please update price data first.", ticker)
        ));
    }

    info!("Found {} price points for {}", prices.len(), ticker);

    // 4. Generate sentiment signal
    let signal = sentiment_service::generate_sentiment_signal(
        &state.pool,
        &ticker,
        themes,
        prices,
    ).await?;

    info!(
        "Generated sentiment signal for {}: score={:.2}, trend={:?}, divergence={:?}",
        ticker, signal.current_sentiment, signal.sentiment_trend, signal.divergence
    );

    Ok(Json(signal))
}

/// GET /api/sentiment/positions/:ticker/enhanced-sentiment
/// Enhanced sentiment combining news + SEC filings + insider trading
/// Query params: days (default: 30)
pub async fn get_enhanced_position_sentiment(
    Path(ticker): Path<String>,
    Query(params): Query<SentimentQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<EnhancedSentimentSignal>, AppError> {
    info!("Fetching enhanced sentiment for ticker: {}", ticker);

    // Check if news service is enabled
    if !state.news_service.is_enabled() {
        return Err(AppError::External(
            "News service is not enabled. Please configure NEWS_ENABLED=true and NEWS_API_KEY to use sentiment analysis.".to_string()
        ));
    }

    // Create SEC Edgar service
    let edgar_service = sec_edgar_service::SecEdgarService::new();

    // Generate enhanced sentiment
    let enhanced_signal = enhanced_sentiment_service::generate_enhanced_sentiment(
        &state.pool,
        &edgar_service,
        &state.llm_service,
        &state.news_service,
        &ticker,
        params.days,
    ).await?;

    info!(
        "Generated enhanced sentiment for {}: combined={:.2}, confidence={:?}",
        ticker, enhanced_signal.combined_sentiment, enhanced_signal.confidence_level
    );

    Ok(Json(enhanced_signal))
}

/// GET /api/sentiment/portfolios/:portfolio_id/sentiment
pub async fn get_portfolio_sentiment(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioSentimentAnalysis>, AppError> {
    info!("Fetching portfolio sentiment for portfolio_id: {}", portfolio_id);

    // Check if news service is enabled
    if !state.news_service.is_enabled() {
        return Err(AppError::External(
            "News service is not enabled. Please configure NEWS_ENABLED=true and NEWS_API_KEY to use sentiment analysis.".to_string()
        ));
    }

    // 1. Get all positions in the portfolio
    let positions = sqlx::query!(
        r#"
        SELECT DISTINCT hs.ticker
        FROM holdings_snapshots hs
        JOIN accounts a ON hs.account_id = a.id
        WHERE a.portfolio_id = $1
          AND hs.quantity > 0
        ORDER BY hs.ticker
        "#,
        portfolio_id
    )
    .fetch_all(&state.pool)
    .await?;

    if positions.is_empty() {
        return Err(AppError::Validation(
            "Portfolio has no positions".to_string()
        ));
    }

    info!("Found {} positions in portfolio", positions.len());

    // 2. Fetch sentiment signal for each position
    let mut signals = Vec::new();
    let mut bullish_divergences = 0;
    let mut bearish_divergences = 0;

    for position in positions {
        let ticker = position.ticker;

        // Try to fetch sentiment for this ticker
        match fetch_ticker_sentiment(&state, &ticker, 30).await {
            Ok(signal) => {
                // Count divergences
                match signal.divergence {
                    DivergenceType::Bullish => bullish_divergences += 1,
                    DivergenceType::Bearish => bearish_divergences += 1,
                    _ => {}
                }
                signals.push(signal);
            }
            Err(e) => {
                error!("Failed to fetch sentiment for {}: {}", ticker, e);
                // Continue with other tickers instead of failing the whole request
            }
        }
    }

    if signals.is_empty() {
        return Err(AppError::Validation(
            "Could not fetch sentiment for any positions in the portfolio".to_string()
        ));
    }

    // 3. Calculate portfolio average sentiment
    let portfolio_avg_sentiment = signals.iter()
        .map(|s| s.current_sentiment)
        .sum::<f64>() / signals.len() as f64;

    let analysis = PortfolioSentimentAnalysis {
        portfolio_id: portfolio_id.to_string(),
        signals,
        portfolio_avg_sentiment,
        bullish_divergences,
        bearish_divergences,
        calculated_at: chrono::Utc::now(),
    };

    info!(
        "Generated portfolio sentiment: avg={:.2}, bullish={}, bearish={}",
        portfolio_avg_sentiment, bullish_divergences, bearish_divergences
    );

    Ok(Json(analysis))
}

/// POST /api/sentiment/cache/clear
/// Clear all sentiment caches (both regular and enhanced)
pub async fn clear_sentiment_cache(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Clearing all sentiment caches");

    // Clear enhanced sentiment cache
    let enhanced_deleted = sqlx::query!("DELETE FROM enhanced_sentiment_cache")
        .execute(&state.pool)
        .await?
        .rows_affected();

    // Clear regular sentiment cache
    let regular_deleted = sqlx::query!("DELETE FROM sentiment_signal_cache")
        .execute(&state.pool)
        .await?
        .rows_affected();

    info!(
        "Cleared {} enhanced sentiment cache entries and {} regular sentiment cache entries",
        enhanced_deleted, regular_deleted
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "enhanced_cache_cleared": enhanced_deleted,
        "regular_cache_cleared": regular_deleted,
        "message": "All sentiment caches cleared successfully"
    })))
}

/// Helper function to fetch sentiment for a single ticker
async fn fetch_ticker_sentiment(
    state: &AppState,
    ticker: &str,
    days: i32,
) -> Result<SentimentSignal, AppError> {
    // Fetch news
    let articles = state.news_service.fetch_ticker_news(ticker, days).await?;

    if articles.is_empty() {
        return Err(AppError::Validation(
            format!("No news for {}", ticker)
        ));
    }

    // Cluster into themes
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Failed to parse demo user UUID");
    let themes = state.news_service.cluster_into_themes(articles, demo_user_id).await?;

    if themes.is_empty() {
        return Err(AppError::Validation(
            format!("No themes for {}", ticker)
        ));
    }

    // Fetch price history
    let prices = price_service::get_history(&state.pool, ticker).await?;

    if prices.is_empty() {
        return Err(AppError::Validation(
            format!("No prices for {}", ticker)
        ));
    }

    // Generate signal
    sentiment_service::generate_sentiment_signal(
        &state.pool,
        ticker,
        themes,
        prices,
    ).await
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions/:ticker/sentiment", get(get_position_sentiment))
        .route("/positions/:ticker/enhanced-sentiment", get(get_enhanced_position_sentiment))
        .route("/portfolios/:portfolio_id/sentiment", get(get_portfolio_sentiment))
        .route("/cache/clear", post(clear_sentiment_cache))
}

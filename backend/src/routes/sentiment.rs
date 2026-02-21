use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{SentimentSignal, PortfolioSentimentAnalysis, DivergenceType, EnhancedSentimentSignal, SentimentTrend, MomentumTrend, SentimentDataPoint};
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
///
/// FAST VERSION: Reads from sentiment_signal_cache instead of fetching fresh data
pub async fn get_portfolio_sentiment(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<PortfolioSentimentAnalysis>, AppError> {
    info!("Fetching portfolio sentiment from cache for portfolio_id: {}", portfolio_id);

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

    // 2. Read cached sentiment for each position from sentiment_signal_cache
    let mut signals = Vec::new();
    let mut bullish_divergences = 0;
    let mut bearish_divergences = 0;

    for position in positions {
        let ticker = &position.ticker;

        // Read from cache instead of fetching fresh data
        let cached = sqlx::query!(
            r#"
            SELECT
                ticker,
                current_sentiment,
                sentiment_trend,
                momentum_trend,
                divergence,
                sentiment_price_correlation,
                correlation_lag_days,
                historical_sentiment,
                news_articles_analyzed,
                warnings,
                calculated_at
            FROM sentiment_signal_cache
            WHERE ticker = $1
              AND expires_at > NOW()
            "#,
            ticker
        )
        .fetch_optional(&state.pool)
        .await?;

        if let Some(cache) = cached {
            // Parse the cached data into SentimentSignal
            let historical: Vec<SentimentDataPoint> = serde_json::from_value(cache.historical_sentiment)
                .unwrap_or_default();

            let warnings: Vec<String> = serde_json::from_value(cache.warnings)
                .unwrap_or_default();

            let divergence = match cache.divergence.as_str() {
                "Bullish" => {
                    bullish_divergences += 1;
                    DivergenceType::Bullish
                },
                "Bearish" => {
                    bearish_divergences += 1;
                    DivergenceType::Bearish
                },
                _ => DivergenceType::None,
            };

            let sentiment_trend = match cache.sentiment_trend.as_str() {
                "Improving" => SentimentTrend::Improving,
                "Deteriorating" => SentimentTrend::Deteriorating,
                _ => SentimentTrend::Stable,
            };

            let momentum_trend = match cache.momentum_trend.as_str() {
                "Bullish" => MomentumTrend::Bullish,
                "Bearish" => MomentumTrend::Bearish,
                _ => MomentumTrend::Neutral,
            };

            // Calculate correlation strength from correlation value
            let correlation_strength = cache.sentiment_price_correlation.map(|corr| {
                let abs_corr = corr.abs();
                if abs_corr >= 0.7 {
                    "strong".to_string()
                } else if abs_corr >= 0.4 {
                    "moderate".to_string()
                } else {
                    "weak".to_string()
                }
            });

            let signal = SentimentSignal {
                ticker: cache.ticker,
                current_sentiment: cache.current_sentiment,
                sentiment_trend,
                momentum_trend,
                divergence,
                sentiment_price_correlation: cache.sentiment_price_correlation,
                correlation_lag_days: cache.correlation_lag_days,
                correlation_strength,
                historical_sentiment: historical,
                news_articles_analyzed: cache.news_articles_analyzed,
                warnings,
                calculated_at: cache.calculated_at.and_utc(),
            };

            signals.push(signal);
        } else {
            // No cached data for this ticker - skip it
            info!("No cached sentiment data for ticker: {}", ticker);
        }
    }

    if signals.is_empty() {
        return Err(AppError::Validation(
            "No cached sentiment data available for portfolio positions. Please wait for the sentiment cache to be populated by background jobs.".to_string()
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
        "Generated portfolio sentiment from cache: avg={:.2}, bullish={}, bearish={}, positions with data: {}",
        portfolio_avg_sentiment, bullish_divergences, bearish_divergences, analysis.signals.len()
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

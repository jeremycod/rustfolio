use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{SentimentSignal, PortfolioSentimentAnalysis, DivergenceType, EnhancedSentimentSignal, SentimentTrend, MomentumTrend, SentimentDataPoint, SentimentAwareForecast};
use crate::services::{sentiment_service, price_service, enhanced_sentiment_service, sec_edgar_service, sentiment_forecasting_service};
use crate::state::AppState;

/// Query parameters for sentiment analysis
#[derive(Debug, Deserialize)]
pub struct SentimentQueryParams {
    /// Number of days to analyze (default: 30)
    #[serde(default = "default_days")]
    pub days: i32,
}

/// Query parameters for sentiment forecast
#[derive(Debug, Deserialize)]
pub struct SentimentForecastParams {
    /// Number of days to forecast (default: 30)
    #[serde(default = "default_forecast_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

fn default_forecast_days() -> i32 {
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
/// FAST VERSION: Reads from sentiment_signal_cache instead of fetching fresh data.
/// Returns cached data even if expired, with warnings about staleness.
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
    // Accept even expired cache, but warn the user
    let mut signals = Vec::new();
    let mut bullish_divergences = 0;
    let mut bearish_divergences = 0;
    let mut missing_tickers = Vec::new();

    for position in positions {
        let ticker = &position.ticker;

        // Read from cache - accept even expired data
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
                calculated_at,
                expires_at
            FROM sentiment_signal_cache
            WHERE ticker = $1
            ORDER BY calculated_at DESC
            LIMIT 1
            "#,
            ticker
        )
        .fetch_optional(&state.pool)
        .await?;

        if let Some(cache) = cached {
            // Check if data is stale
            let is_expired = cache.expires_at < chrono::Utc::now().naive_utc();
            let age_hours = (chrono::Utc::now().naive_utc() - cache.calculated_at).num_hours();
            // Parse the cached data into SentimentSignal
            let historical: Vec<SentimentDataPoint> = serde_json::from_value(cache.historical_sentiment)
                .unwrap_or_default();

            let mut warnings: Vec<String> = serde_json::from_value(cache.warnings)
                .unwrap_or_default();

            // Add staleness warning if expired
            if is_expired {
                warnings.insert(0, format!(
                    "⚠️ Data is {} hours old and may be stale. Click Refresh Sentiment to update.",
                    age_hours
                ));
            }

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
            // No cached data for this ticker - track it
            info!("No cached sentiment data for ticker: {}", ticker);
            missing_tickers.push(ticker.clone());
        }
    }

    // If no data at all, return helpful error
    if signals.is_empty() {
        return Err(AppError::Validation(
            format!(
                "No sentiment data available yet. Sentiment analysis requires news data which is fetched on-demand. \
                Missing data for: {}. \
                Sentiment data will be available after the first analysis is triggered.",
                missing_tickers.join(", ")
            )
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

/// GET /api/sentiment/portfolios/:portfolio_id/cache-status
/// Get cache status for portfolio sentiment data
pub async fn get_portfolio_sentiment_cache_status(
    Path(portfolio_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Checking sentiment cache status for portfolio_id: {}", portfolio_id);

    // Get all tickers in the portfolio
    let tickers = sqlx::query!(
        r#"
        SELECT DISTINCT hs.ticker
        FROM holdings_snapshots hs
        JOIN accounts a ON hs.account_id = a.id
        WHERE a.portfolio_id = $1
          AND hs.quantity > 0
        "#,
        portfolio_id
    )
    .fetch_all(&state.pool)
    .await?;

    if tickers.is_empty() {
        return Ok(Json(serde_json::json!({
            "portfolio_id": portfolio_id.to_string(),
            "total_positions": 0,
            "cached_positions": 0,
            "missing_positions": 0,
            "expired_positions": 0,
            "is_complete": true,
            "is_fresh": true,
            "message": "Portfolio has no positions"
        })));
    }

    let mut cached = 0;
    let mut expired = 0;
    let mut missing = 0;
    let mut oldest_update: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut missing_list = Vec::new();

    for ticker_row in &tickers {
        let ticker = &ticker_row.ticker;

        let cache_row = sqlx::query!(
            r#"
            SELECT calculated_at, expires_at
            FROM sentiment_signal_cache
            WHERE ticker = $1
            ORDER BY calculated_at DESC
            LIMIT 1
            "#,
            ticker
        )
        .fetch_optional(&state.pool)
        .await?;

        if let Some(cache) = cache_row {
            let calc_at = cache.calculated_at.and_utc();
            cached += 1;

            if cache.expires_at < chrono::Utc::now().naive_utc() {
                expired += 1;
            }

            // Track oldest update
            if oldest_update.is_none() || calc_at < oldest_update.unwrap() {
                oldest_update = Some(calc_at);
            }
        } else {
            missing += 1;
            missing_list.push(ticker.clone());
        }
    }

    let total = tickers.len();
    let is_complete = missing == 0;
    let is_fresh = expired == 0;
    let cache_age_hours = oldest_update
        .map(|dt| (chrono::Utc::now() - dt).num_hours())
        .unwrap_or(0);

    let message = if missing > 0 {
        format!(
            "Sentiment data is missing for {} position(s). News data is fetched on-demand.",
            missing
        )
    } else if expired > 0 {
        format!(
            "Sentiment data for {} position(s) is stale (oldest: {} hours old). Refresh recommended.",
            expired, cache_age_hours
        )
    } else {
        "All sentiment data is up to date".to_string()
    };

    Ok(Json(serde_json::json!({
        "portfolio_id": portfolio_id.to_string(),
        "total_positions": total,
        "cached_positions": cached,
        "missing_positions": missing,
        "expired_positions": expired,
        "missing_tickers": missing_list,
        "is_complete": is_complete,
        "is_fresh": is_fresh,
        "oldest_update": oldest_update,
        "cache_age_hours": cache_age_hours,
        "message": message,
        "recommendation": if !is_complete || !is_fresh {
            "Sentiment analysis requires news data. Use the Refresh button to fetch the latest news and generate sentiment analysis."
        } else {
            "Cache is healthy"
        }
    })))
}

/// Helper function to fetch sentiment for a single ticker
#[allow(dead_code)]
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

/// GET /api/sentiment/positions/:ticker/sentiment-forecast
/// Generate sentiment-aware price forecast for a stock
/// Query params: days (default: 30)
pub async fn get_sentiment_aware_forecast(
    Path(ticker): Path<String>,
    Query(params): Query<SentimentForecastParams>,
    State(state): State<AppState>,
) -> Result<Json<SentimentAwareForecast>, AppError> {
    info!("Generating sentiment-aware forecast for {}, days: {}", ticker, params.days);

    // Check if news service is enabled
    if !state.news_service.is_enabled() {
        return Err(AppError::External(
            "News service is not enabled. Please configure NEWS_ENABLED=true and NEWS_API_KEY to use sentiment-aware forecasts.".to_string()
        ));
    }

    // 1. Get enhanced sentiment (includes news, SEC, insider)
    let edgar_service = sec_edgar_service::SecEdgarService::new();
    let enhanced_sentiment = enhanced_sentiment_service::generate_enhanced_sentiment(
        &state.pool,
        &edgar_service,
        &state.llm_service,
        &state.news_service,
        &ticker,
        30, // Use 30 days of sentiment data
    ).await?;

    info!(
        "Enhanced sentiment for {}: combined={:.2}, confidence={:?}",
        ticker, enhanced_sentiment.combined_sentiment, enhanced_sentiment.confidence_level
    );

    // 2. Generate sentiment-aware forecast
    let forecast = sentiment_forecasting_service::generate_sentiment_aware_stock_forecast(
        &state.pool,
        &ticker,
        &enhanced_sentiment,
        params.days,
    ).await?;

    info!(
        "Generated sentiment-aware forecast for {}: reversal_prob={:.2}, divergence={}",
        ticker, forecast.reversal_probability, forecast.sentiment_factors.divergence_detected
    );

    Ok(Json(forecast))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions/:ticker/sentiment", get(get_position_sentiment))
        .route("/positions/:ticker/enhanced-sentiment", get(get_enhanced_position_sentiment))
        .route("/positions/:ticker/sentiment-forecast", get(get_sentiment_aware_forecast))
        .route("/portfolios/:portfolio_id/sentiment", get(get_portfolio_sentiment))
        .route("/portfolios/:portfolio_id/cache-status", get(get_portfolio_sentiment_cache_status))
        .route("/cache/clear", post(clear_sentiment_cache))
}

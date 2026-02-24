use axum::extract::{Path, Query, State};
use axum::{Json, Router};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::errors::AppError;
use crate::models::{SignalType, SignalGenerationParams, SignalResponse, TradingSignal};
use crate::services::{price_service, signal_service::SignalService};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:symbol/signals", get(get_stock_signals))
        .route("/:symbol/signals/history", get(get_signals_history))
}

#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    /// Time horizon in months (default: 3)
    horizon: Option<i32>,

    /// Filter by signal types (comma-separated: momentum,trend,mean_reversion,combined)
    signal_types: Option<String>,

    /// Minimum probability threshold (0.0 to 1.0, default: 0.5)
    min_probability: Option<f64>,

    /// Force refresh (ignore cache)
    #[serde(default)]
    refresh: bool,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ErrorResponse {
    error: String,
}

/// Get trading signals for a stock symbol
///
/// Generates probability-based trading signals using technical indicators:
/// - Momentum signals (RSI, MACD, price momentum)
/// - Mean reversion signals (Bollinger bands, oversold/overbought)
/// - Trend signals (SMA crossovers, EMA alignment)
/// - Combined multi-factor signals
///
/// Signals are cached for 4 hours unless refresh=true
///
/// # Example
/// ```
/// GET /api/stocks/AAPL/signals?horizon=3&signal_types=combined&min_probability=0.6
/// ```
#[axum::debug_handler]
pub async fn get_stock_signals(
    Path(symbol): Path<String>,
    Query(query): Query<SignalQuery>,
    State(state): State<AppState>,
) -> Result<Json<SignalResponse>, AppError> {
    let symbol = symbol.to_uppercase();
    let horizon = query.horizon.unwrap_or(3);

    info!("GET /stocks/{}/signals - horizon={}, refresh={}", symbol, horizon, query.refresh);

    // Validate horizon
    if !(1..=12).contains(&horizon) {
        error!("Invalid horizon: {}. Must be between 1 and 12 months.", horizon);
        return Err(AppError::Validation(
            "Invalid horizon. Must be between 1 and 12 months.".to_string(),
        ));
    }

    // Validate min_probability
    let min_probability = query.min_probability.unwrap_or(0.5);
    if !(0.0..=1.0).contains(&min_probability) {
        error!("Invalid min_probability: {}. Must be between 0.0 and 1.0.", min_probability);
        return Err(AppError::Validation(
            "Invalid min_probability. Must be between 0.0 and 1.0.".to_string(),
        ));
    }

    // Parse signal types filter
    let signal_types_filter: Option<Vec<SignalType>> = query.signal_types.as_ref().map(|types| {
        types
            .split(',')
            .filter_map(|t| match t.trim() {
                "momentum" => Some(SignalType::Momentum),
                "mean_reversion" => Some(SignalType::MeanReversion),
                "trend" => Some(SignalType::Trend),
                "combined" => Some(SignalType::Combined),
                _ => None,
            })
            .collect()
    });

    let signal_service = SignalService::new(state.pool.clone());

    // Check cache first (unless refresh requested)
    if !query.refresh {
        match signal_service
            .get_cached_signals(&symbol, Some(horizon), None)
            .await
        {
            Ok(cached_signals) if !cached_signals.is_empty() => {
                info!("Found {} cached signals for {}", cached_signals.len(), symbol);

                // Filter by horizon, signal types, and min probability
                let filtered_signals: Vec<_> = if let Some(ref types) = signal_types_filter {
                    cached_signals
                        .into_iter()
                        .filter(|s| s.horizon_months == horizon)
                        .filter(|s| types.contains(&s.signal_type))
                        .filter(|s| s.probability >= min_probability)
                        .collect()
                } else {
                    cached_signals
                        .into_iter()
                        .filter(|s| s.horizon_months == horizon)
                        .filter(|s| s.probability >= min_probability)
                        .collect()
                };

                // Deduplicate: keep only the most recent signal for each signal_type
                // Group by signal_type and keep the one with the latest generated_at
                use std::collections::HashMap;
                let mut latest_signals: HashMap<SignalType, TradingSignal> = HashMap::new();
                for signal in filtered_signals {
                    latest_signals
                        .entry(signal.signal_type)
                        .and_modify(|existing| {
                            if signal.generated_at > existing.generated_at {
                                *existing = signal.clone();
                            }
                        })
                        .or_insert(signal);
                }

                let deduplicated_signals: Vec<_> = latest_signals.into_values().collect();

                if !deduplicated_signals.is_empty() {
                    info!("Returning {} deduplicated cached signals for {}", deduplicated_signals.len(), symbol);

                    // Build response from cached signals
                    let (recommendation, confidence) =
                        signal_service.calculate_overall_recommendation(&deduplicated_signals);

                    let response = SignalResponse {
                        ticker: symbol.clone(),
                        current_price: None, // Not available from cache
                        signals: deduplicated_signals,
                        recommendation: Some(recommendation),
                        confidence: Some(confidence),
                        analyzed_at: chrono::Utc::now(),
                    };

                    return Ok(Json(response));
                }
            }
            Ok(_) => {
                info!("No cached signals found for {}, generating fresh signals", symbol);
            }
            Err(e) => {
                error!("Cache lookup error for {}: {}", symbol, e);
                // Continue to generate fresh signals
            }
        }
    }

    // Fetch price data
    let price_data = price_service::get_history(&state.pool, &symbol)
        .await
        .map_err(|e| {
            error!("Failed to fetch price data for {}: {}", symbol, e);
            e
        })?;

    if price_data.is_empty() {
        error!("No price data found for symbol: {}", symbol);
        return Err(AppError::NotFound(format!(
            "No price data found for symbol: {}",
            symbol
        )));
    }

    // Ensure we have enough data
    if price_data.len() < 30 {
        error!(
            "Insufficient price data for {}. Need at least 30 days, got {}",
            symbol,
            price_data.len()
        );
        return Err(AppError::Validation(format!(
            "Insufficient price data for {}. Need at least 30 days of history.",
            symbol
        )));
    }

    info!("Fetched {} days of price data for {}", price_data.len(), symbol);

    // Extract prices (most recent first)
    // Note: PricePoint only has close_price, no volume data
    let mut prices: Vec<f64> = price_data
        .iter()
        .filter_map(|p| p.close_price.to_string().parse::<f64>().ok())
        .collect();

    // Reverse to get chronological order (oldest first) as expected by indicators
    prices.reverse();

    let current_price = *prices.last().unwrap_or(&0.0);

    // Generate dummy volumes (all equal since we don't have volume data in PricePoint)
    let volumes: Vec<f64> = vec![1000000.0; prices.len()];

    // Only generate signals for the requested horizon
    let horizons = vec![horizon];

    let params = SignalGenerationParams {
        ticker: symbol.clone(),
        prices,
        volumes,
        horizons,
        current_price,
    };

    info!("Generating signals for {} with horizon: {}", symbol, horizon);

    // Generate fresh signals
    let mut response = signal_service
        .generate_signals(params)
        .await
        .map_err(|e| {
            error!("Failed to generate signals for {}: {}", symbol, e);
            AppError::External(format!("Failed to generate signals: {}", e))
        })?;

    info!("Generated {} signals for {}", response.signals.len(), symbol);

    // Filter by signal types, min probability, and requested horizon
    response.signals.retain(|s| s.horizon_months == horizon);
    if let Some(ref types) = signal_types_filter {
        response.signals.retain(|s| types.contains(&s.signal_type));
    }
    response.signals.retain(|s| s.probability >= min_probability);

    // Recalculate recommendation after filtering
    let (recommendation, confidence) =
        signal_service.calculate_overall_recommendation(&response.signals);
    response.recommendation = Some(recommendation);
    response.confidence = Some(confidence);

    info!(
        "Returning {} filtered signals for {} (recommendation: {:?})",
        response.signals.len(), symbol, response.recommendation
    );

    Ok(Json(response))
}

/// Get cached signals history for a stock
///
/// Returns historical signals for backtesting and accuracy tracking
///
/// # Example
/// ```
/// GET /api/stocks/AAPL/signals/history?days=30
/// ```
#[axum::debug_handler]
pub async fn get_signals_history(
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<HistoryResponse>, AppError> {
    let symbol = symbol.to_uppercase();
    let days = query.days.unwrap_or(30);

    info!("GET /stocks/{}/signals/history - days={}", symbol, days);

    if days > 365 {
        error!("Requested history too long: {} days (max 365)", days);
        return Err(AppError::Validation(
            "Maximum history is 365 days".to_string(),
        ));
    }

    let signal_service = SignalService::new(state.pool.clone());

    let signals = signal_service
        .get_cached_signals(&symbol, None, None)
        .await
        .map_err(|e| {
            error!("Failed to fetch signal history for {}: {}", symbol, e);
            AppError::External(format!("Failed to fetch signal history: {}", e))
        })?;

    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let filtered: Vec<_> = signals
        .into_iter()
        .filter(|s| s.generated_at >= cutoff)
        .collect();

    info!("Returning {} historical signals for {}", filtered.len(), symbol);

    Ok(Json(HistoryResponse {
        ticker: symbol,
        signals: filtered.clone(),
        count: filtered.len(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    /// Number of days of history (default: 30, max: 365)
    days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    ticker: String,
    signals: Vec<crate::models::TradingSignal>,
    count: usize,
}

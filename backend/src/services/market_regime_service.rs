use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use tracing::info;

use crate::db::market_regime_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::{
    AdjustedThresholds, CreateMarketRegime, CurrentRegimeWithThresholds, MarketRegime,
    RegimeDetectionParams, RegimeType,
};
use crate::models::risk::RiskThresholdSettings;

/// Detect the current market regime based on recent market data
///
/// This function analyzes recent price data for a benchmark ticker (typically SPY)
/// to classify the current market regime into one of four categories:
/// - Bull: Positive returns with low volatility
/// - Bear: Negative returns with elevated volatility
/// - High Volatility: Extreme volatility regardless of returns
/// - Normal: Standard market conditions
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `params` - Regime detection parameters (benchmark, lookback period, thresholds)
/// * `price_provider` - External price data provider
///
/// # Returns
///
/// Returns the detected regime type, volatility level, market return, and confidence score
pub async fn detect_current_regime(
    _pool: &PgPool,
    params: &RegimeDetectionParams,
    price_provider: &dyn PriceProvider,
) -> Result<(RegimeType, f64, f64, f64), AppError> {
    info!(
        "Detecting market regime using {} with {}-day lookback",
        params.benchmark_ticker, params.lookback_days
    );

    // Fetch recent price data from provider
    let days = (params.lookback_days + 10) as u32; // Extra buffer for calculations
    let prices = price_provider
        .fetch_daily_history(&params.benchmark_ticker, days)
        .await
        .map_err(|e| AppError::External(format!("Failed to fetch prices: {}", e)))?;

    if prices.is_empty() {
        return Err(AppError::External(format!(
            "No price data available for {}",
            params.benchmark_ticker
        )));
    }

    // Convert to simple format for calculation
    let price_points: Vec<PricePoint> = prices
        .iter()
        .map(|p| PricePoint {
            date: p.date,
            close: p.close.to_f64().unwrap_or(0.0),
        })
        .collect();

    calculate_regime_from_prices(&price_points, params)
}

/// Simple price point for calculations
struct PricePoint {
    date: NaiveDate,
    close: f64,
}

/// Calculate regime from price data
fn calculate_regime_from_prices(
    prices: &[PricePoint],
    params: &RegimeDetectionParams,
) -> Result<(RegimeType, f64, f64, f64), AppError> {
    if prices.len() < params.lookback_days as usize {
        return Err(AppError::External(format!(
            "Insufficient price data: need {} days, have {}",
            params.lookback_days,
            prices.len()
        )));
    }

    // Take only the most recent lookback_days
    let recent_prices = &prices[prices.len().saturating_sub(params.lookback_days as usize)..];

    // Calculate daily returns
    let mut returns = Vec::new();
    for i in 1..recent_prices.len() {
        let prev_close = recent_prices[i - 1].close;
        let curr_close = recent_prices[i].close;
        if prev_close > 0.0 {
            let daily_return = (curr_close - prev_close) / prev_close;
            returns.push(daily_return);
        }
    }

    if returns.is_empty() {
        return Err(AppError::External(
            "Could not calculate returns from price data".to_string(),
        ));
    }

    // Calculate volatility (annualized standard deviation)
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|r| {
            let diff = r - mean_return;
            diff * diff
        })
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();
    let annualized_volatility = std_dev * (252.0_f64).sqrt() * 100.0; // Convert to percentage

    // Calculate total return over period
    let start_price = recent_prices[0].close;
    let end_price = recent_prices[recent_prices.len() - 1].close;
    let total_return = ((end_price - start_price) / start_price) * 100.0; // As percentage

    // Detect regime based on volatility and returns
    let (regime_type, confidence) = classify_regime(
        annualized_volatility,
        total_return,
        params.bull_volatility_threshold,
        params.bear_volatility_threshold,
        params.high_volatility_threshold,
    );

    info!(
        "Regime detected: {:?}, volatility: {:.2}%, return: {:.2}%, confidence: {:.1}%",
        regime_type, annualized_volatility, total_return, confidence
    );

    Ok((regime_type, annualized_volatility, total_return, confidence))
}

/// Classify market regime based on volatility and returns
///
/// Rules:
/// - High Volatility: volatility > high_volatility_threshold (takes precedence)
/// - Bull: positive returns AND volatility < bull_volatility_threshold
/// - Bear: negative returns AND volatility > bear_volatility_threshold
/// - Normal: everything else
fn classify_regime(
    volatility: f64,
    returns: f64,
    bull_vol_threshold: f64,
    bear_vol_threshold: f64,
    high_vol_threshold: f64,
) -> (RegimeType, f64) {
    // High volatility regime (takes precedence)
    if volatility > high_vol_threshold {
        let confidence = ((volatility - high_vol_threshold) / high_vol_threshold * 100.0)
            .min(100.0)
            .max(75.0);
        return (RegimeType::HighVolatility, confidence);
    }

    // Bull market: positive returns + low volatility
    if returns > 0.0 && volatility < bull_vol_threshold {
        let vol_margin = (bull_vol_threshold - volatility) / bull_vol_threshold;
        let return_strength = (returns / 10.0).min(1.0); // Normalize returns
        let confidence = ((vol_margin * 0.6 + return_strength * 0.4) * 100.0)
            .min(100.0)
            .max(60.0);
        return (RegimeType::Bull, confidence);
    }

    // Bear market: negative returns + elevated volatility
    if returns < 0.0 && volatility > bear_vol_threshold {
        let vol_excess = (volatility - bear_vol_threshold) / bear_vol_threshold;
        let return_decline = (returns.abs() / 10.0).min(1.0); // Normalize returns
        let confidence = ((vol_excess * 0.6 + return_decline * 0.4) * 100.0)
            .min(100.0)
            .max(60.0);
        return (RegimeType::Bear, confidence);
    }

    // Normal market (default)
    // Confidence is lower for normal regime as it's the catch-all
    let confidence = 70.0;
    (RegimeType::Normal, confidence)
}

/// Update the market regime in the database for a specific date
///
/// This function should be called daily (typically after market close) to update
/// the regime classification and store it for historical tracking.
pub async fn update_regime_for_date(
    pool: &PgPool,
    date: NaiveDate,
    params: &RegimeDetectionParams,
    price_provider: &dyn PriceProvider,
) -> Result<MarketRegime, AppError> {
    let (regime_type, volatility, market_return, confidence) =
        detect_current_regime(pool, params, price_provider).await?;

    let threshold_multiplier = regime_type.threshold_multiplier();

    let create_regime = CreateMarketRegime {
        date,
        regime_type: regime_type.to_string(),
        volatility_level: BigDecimal::from_f64(volatility)
            .ok_or_else(|| AppError::External("Failed to convert volatility".to_string()))?,
        market_return: BigDecimal::from_f64(market_return),
        confidence: BigDecimal::from_f64(confidence)
            .ok_or_else(|| AppError::External("Failed to convert confidence".to_string()))?,
        benchmark_ticker: params.benchmark_ticker.clone(),
        lookback_days: params.lookback_days as i32,
        threshold_multiplier: BigDecimal::from_f64(threshold_multiplier)
            .ok_or_else(|| AppError::External("Failed to convert multiplier".to_string()))?,
    };

    let regime = market_regime_queries::upsert_regime(pool, create_regime)
        .await
        .map_err(AppError::Db)?;

    Ok(regime)
}

/// Get the current market regime with adjusted thresholds
pub async fn get_current_regime_with_thresholds(
    pool: &PgPool,
) -> Result<CurrentRegimeWithThresholds, AppError> {
    let regime = market_regime_queries::get_current_regime(pool)
        .await
        .map_err(AppError::Db)?;

    let regime_type = RegimeType::from_string(&regime.regime_type);
    let adjusted_thresholds = AdjustedThresholds::from_regime_type(&regime_type);

    Ok(CurrentRegimeWithThresholds {
        regime,
        adjusted_thresholds,
    })
}

/// Get historical market regimes
pub async fn get_regime_history(
    pool: &PgPool,
    days: i64,
) -> Result<Vec<MarketRegime>, AppError> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(days);

    market_regime_queries::get_regime_history(pool, start_date, end_date)
        .await
        .map_err(AppError::Db)
}

/// Calculate adaptive thresholds based on current regime
///
/// This function takes base thresholds and adjusts them according to the current
/// market regime, making risk detection more context-aware.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `base_thresholds` - The unadjusted risk thresholds
///
/// # Returns
///
/// Returns a new RiskThresholdSettings struct with adjusted values
pub async fn calculate_adaptive_thresholds(
    pool: &PgPool,
    base_thresholds: &RiskThresholdSettings,
) -> Result<RiskThresholdSettings, AppError> {
    let regime = market_regime_queries::get_current_regime(pool)
        .await
        .map_err(AppError::Db)?;

    let multiplier = regime
        .threshold_multiplier
        .to_f64()
        .unwrap_or(1.0);

    // Apply multiplier to all thresholds
    let mut adjusted = base_thresholds.clone();
    adjusted.volatility_warning_threshold *= multiplier;
    adjusted.volatility_critical_threshold *= multiplier;
    adjusted.drawdown_warning_threshold *= multiplier;
    adjusted.drawdown_critical_threshold *= multiplier;
    adjusted.beta_warning_threshold *= multiplier;
    adjusted.beta_critical_threshold *= multiplier;
    adjusted.risk_score_warning_threshold *= multiplier;
    adjusted.risk_score_critical_threshold *= multiplier;
    adjusted.var_warning_threshold *= multiplier;
    adjusted.var_critical_threshold *= multiplier;

    Ok(adjusted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_regime_high_volatility() {
        let (regime, confidence) = classify_regime(40.0, 5.0, 20.0, 25.0, 35.0);
        assert_eq!(regime, RegimeType::HighVolatility);
        assert!(confidence >= 75.0);
    }

    #[test]
    fn test_classify_regime_bull() {
        let (regime, confidence) = classify_regime(15.0, 8.0, 20.0, 25.0, 35.0);
        assert_eq!(regime, RegimeType::Bull);
        assert!(confidence >= 60.0);
    }

    #[test]
    fn test_classify_regime_bear() {
        let (regime, confidence) = classify_regime(30.0, -5.0, 20.0, 25.0, 35.0);
        assert_eq!(regime, RegimeType::Bear);
        assert!(confidence >= 60.0);
    }

    #[test]
    fn test_classify_regime_normal() {
        let (regime, _) = classify_regime(18.0, 2.0, 20.0, 25.0, 35.0);
        assert_eq!(regime, RegimeType::Normal);
    }

    #[test]
    fn test_classify_regime_normal_flat() {
        let (regime, _) = classify_regime(22.0, 0.5, 20.0, 25.0, 35.0);
        assert_eq!(regime, RegimeType::Normal);
    }

    #[test]
    fn test_threshold_multiplier_application() {
        // Bull market should tighten thresholds
        let base_volatility = 30.0;
        let bull_multiplier = RegimeType::Bull.threshold_multiplier();
        assert_eq!(base_volatility * bull_multiplier, 24.0); // 30 * 0.8

        // Bear market should loosen thresholds
        let bear_multiplier = RegimeType::Bear.threshold_multiplier();
        assert_eq!(base_volatility * bear_multiplier, 39.0); // 30 * 1.3

        // High volatility should significantly loosen thresholds
        let high_vol_multiplier = RegimeType::HighVolatility.threshold_multiplier();
        assert_eq!(base_volatility * high_vol_multiplier, 45.0); // 30 * 1.5
    }
}

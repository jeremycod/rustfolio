use crate::db::{price_queries, watchlist_queries};
use crate::models::watchlist::*;
use crate::services::indicators;
use serde_json::json;
use sqlx::PgPool;

const ALERT_COOLDOWN_HOURS: i32 = 4;
const RSI_PERIOD: usize = 14;
#[allow(dead_code)]
const VOLUME_PERIOD: usize = 20;

// ==============================================================================
// Threshold Breach Detection
// ==============================================================================

/// Check all enabled thresholds for a given ticker and its current data.
/// Returns a list of monitoring results (alerts to generate).
pub async fn check_thresholds(
    pool: &PgPool,
    item: &WatchlistItem,
    user_id: uuid::Uuid,
    current_price: f64,
    rsi_value: Option<f64>,
    volume_ratio: Option<f64>,
    volatility: Option<f64>,
) -> Result<Vec<MonitoringResult>, sqlx::Error> {
    let thresholds = watchlist_queries::get_thresholds_for_item(pool, item.id).await?;
    let mut results = Vec::new();

    for threshold in thresholds {
        if !threshold.enabled {
            continue;
        }

        // Check cooldown
        let has_recent = watchlist_queries::has_recent_alert(
            pool,
            item.id,
            &threshold.threshold_type,
            ALERT_COOLDOWN_HOURS,
        )
        .await?;

        if has_recent {
            continue;
        }

        let comparison = ThresholdComparison::from_str(&threshold.comparison);
        let comparison = match comparison {
            Some(c) => c,
            None => continue,
        };

        let triggered = match threshold.threshold_type.as_str() {
            "price_above" | "price_below" => {
                comparison.evaluate(current_price, threshold.value)
            }
            "price_change_pct" => {
                if let Some(added_price) = item.added_price.as_ref().and_then(|p| p.to_string().parse::<f64>().ok()) {
                    if added_price > 0.0 {
                        let change_pct = ((current_price - added_price) / added_price) * 100.0;
                        comparison.evaluate(change_pct.abs(), threshold.value)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "volatility" => {
                if let Some(vol) = volatility {
                    comparison.evaluate(vol, threshold.value)
                } else {
                    false
                }
            }
            "volume_spike" => {
                if let Some(ratio) = volume_ratio {
                    comparison.evaluate(ratio, threshold.value)
                } else {
                    false
                }
            }
            "rsi_overbought" => {
                if let Some(rsi) = rsi_value {
                    comparison.evaluate(rsi, threshold.value)
                } else {
                    false
                }
            }
            "rsi_oversold" => {
                if let Some(rsi) = rsi_value {
                    // For oversold, we typically check rsi < threshold
                    comparison.evaluate(rsi, threshold.value)
                } else {
                    false
                }
            }
            _ => false,
        };

        if triggered {
            let actual = match threshold.threshold_type.as_str() {
                "price_above" | "price_below" => current_price,
                "price_change_pct" => {
                    if let Some(added) = item.added_price.as_ref().and_then(|p| p.to_string().parse::<f64>().ok()) {
                        ((current_price - added) / added) * 100.0
                    } else {
                        0.0
                    }
                }
                "volatility" => volatility.unwrap_or(0.0),
                "volume_spike" => volume_ratio.unwrap_or(0.0),
                "rsi_overbought" | "rsi_oversold" => rsi_value.unwrap_or(0.0),
                _ => 0.0,
            };

            let severity = determine_severity(&threshold.threshold_type, actual, threshold.value);
            let message = format_alert_message(
                &item.ticker,
                &threshold.threshold_type,
                actual,
                threshold.value,
            );

            results.push(MonitoringResult {
                ticker: item.ticker.clone(),
                watchlist_item_id: item.id,
                user_id,
                alert_type: threshold.threshold_type.clone(),
                severity,
                message,
                actual_value: actual,
                threshold_value: Some(threshold.value),
                metadata: json!({
                    "current_price": current_price,
                    "rsi": rsi_value,
                    "volume_ratio": volume_ratio,
                    "volatility": volatility,
                }),
            });
        }
    }

    Ok(results)
}

// ==============================================================================
// Technical Pattern Recognition (Basic)
// ==============================================================================

/// Detect basic technical patterns from price data.
pub fn detect_patterns(
    prices: &[f64],
    ticker: &str,
    watchlist_item_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Vec<MonitoringResult> {
    let mut results = Vec::new();

    if prices.len() < 30 {
        return results;
    }

    // RSI-based signals
    let rsi_values = indicators::rsi(prices, RSI_PERIOD);
    if let Some(Some(rsi)) = rsi_values.last() {
        if *rsi > 80.0 {
            results.push(MonitoringResult {
                ticker: ticker.to_string(),
                watchlist_item_id,
                user_id,
                alert_type: "pattern_rsi_extreme_overbought".to_string(),
                severity: "high".to_string(),
                message: format!("{}: RSI at {:.1} indicates extreme overbought conditions", ticker, rsi),
                actual_value: *rsi,
                threshold_value: Some(80.0),
                metadata: json!({"pattern": "rsi_extreme_overbought", "rsi": rsi}),
            });
        } else if *rsi < 20.0 {
            results.push(MonitoringResult {
                ticker: ticker.to_string(),
                watchlist_item_id,
                user_id,
                alert_type: "pattern_rsi_extreme_oversold".to_string(),
                severity: "high".to_string(),
                message: format!("{}: RSI at {:.1} indicates extreme oversold conditions", ticker, rsi),
                actual_value: *rsi,
                threshold_value: Some(20.0),
                metadata: json!({"pattern": "rsi_extreme_oversold", "rsi": rsi}),
            });
        }
    }

    // MACD crossover detection
    let (macd_line, signal_line, _) = indicators::macd(prices, 12, 26, 9);
    let len = macd_line.len();
    if len >= 2 {
        if let (Some(curr_macd), Some(curr_signal), Some(prev_macd), Some(prev_signal)) = (
            macd_line[len - 1],
            signal_line[len - 1],
            macd_line[len - 2],
            signal_line[len - 2],
        ) {
            // Bullish crossover: MACD crosses above signal
            if prev_macd <= prev_signal && curr_macd > curr_signal {
                results.push(MonitoringResult {
                    ticker: ticker.to_string(),
                    watchlist_item_id,
                    user_id,
                    alert_type: "pattern_macd_bullish_crossover".to_string(),
                    severity: "medium".to_string(),
                    message: format!("{}: MACD bullish crossover detected (MACD: {:.4}, Signal: {:.4})", ticker, curr_macd, curr_signal),
                    actual_value: curr_macd,
                    threshold_value: Some(curr_signal),
                    metadata: json!({"pattern": "macd_bullish_crossover", "macd": curr_macd, "signal": curr_signal}),
                });
            }
            // Bearish crossover: MACD crosses below signal
            if prev_macd >= prev_signal && curr_macd < curr_signal {
                results.push(MonitoringResult {
                    ticker: ticker.to_string(),
                    watchlist_item_id,
                    user_id,
                    alert_type: "pattern_macd_bearish_crossover".to_string(),
                    severity: "medium".to_string(),
                    message: format!("{}: MACD bearish crossover detected (MACD: {:.4}, Signal: {:.4})", ticker, curr_macd, curr_signal),
                    actual_value: curr_macd,
                    threshold_value: Some(curr_signal),
                    metadata: json!({"pattern": "macd_bearish_crossover", "macd": curr_macd, "signal": curr_signal}),
                });
            }
        }
    }

    // Bollinger Band touch detection
    let (_, upper_band, lower_band) = indicators::bollinger_bands(prices, 20, 2.0);
    if let (Some(Some(upper)), Some(Some(lower))) = (upper_band.last(), lower_band.last()) {
        let last_price = prices[prices.len() - 1];
        if last_price >= *upper {
            results.push(MonitoringResult {
                ticker: ticker.to_string(),
                watchlist_item_id,
                user_id,
                alert_type: "pattern_bollinger_upper_touch".to_string(),
                severity: "low".to_string(),
                message: format!("{}: Price ({:.2}) touched upper Bollinger Band ({:.2})", ticker, last_price, upper),
                actual_value: last_price,
                threshold_value: Some(*upper),
                metadata: json!({"pattern": "bollinger_upper_touch", "price": last_price, "upper_band": upper}),
            });
        } else if last_price <= *lower {
            results.push(MonitoringResult {
                ticker: ticker.to_string(),
                watchlist_item_id,
                user_id,
                alert_type: "pattern_bollinger_lower_touch".to_string(),
                severity: "low".to_string(),
                message: format!("{}: Price ({:.2}) touched lower Bollinger Band ({:.2})", ticker, last_price, lower),
                actual_value: last_price,
                threshold_value: Some(*lower),
                metadata: json!({"pattern": "bollinger_lower_touch", "price": last_price, "lower_band": lower}),
            });
        }
    }

    results
}

// ==============================================================================
// Sentiment Shift Detection
// ==============================================================================

/// Detect significant sentiment shifts by comparing current to previous score.
pub fn detect_sentiment_shift(
    ticker: &str,
    watchlist_item_id: uuid::Uuid,
    user_id: uuid::Uuid,
    current_sentiment: f64,
    previous_sentiment: Option<f64>,
) -> Option<MonitoringResult> {
    let prev = match previous_sentiment {
        Some(p) => p,
        None => return None,
    };

    let shift = current_sentiment - prev;
    let abs_shift = shift.abs();

    // Significant sentiment shift threshold: 0.3 on a -1 to 1 scale
    if abs_shift < 0.3 {
        return None;
    }

    let direction = if shift > 0.0 { "positive" } else { "negative" };
    let severity = if abs_shift >= 0.6 { "high" } else { "medium" };

    Some(MonitoringResult {
        ticker: ticker.to_string(),
        watchlist_item_id,
        user_id,
        alert_type: format!("sentiment_shift_{}", direction),
        severity: severity.to_string(),
        message: format!(
            "{}: Significant {} sentiment shift ({:+.2}, from {:.2} to {:.2})",
            ticker, direction, shift, prev, current_sentiment
        ),
        actual_value: current_sentiment,
        threshold_value: Some(prev),
        metadata: json!({
            "sentiment_shift": shift,
            "previous_sentiment": prev,
            "current_sentiment": current_sentiment,
            "direction": direction,
        }),
    })
}

// ==============================================================================
// Full Monitoring Run for a Ticker
// ==============================================================================

/// Run full monitoring for a single ticker across all watchlist items that contain it.
pub async fn monitor_ticker(
    pool: &PgPool,
    ticker: &str,
) -> Result<Vec<MonitoringResult>, Box<dyn std::error::Error + Send + Sync>> {
    let mut all_results = Vec::new();

    // Fetch price data
    let prices = price_queries::fetch_all(pool, ticker).await?;
    if prices.is_empty() {
        return Ok(all_results);
    }

    let price_values: Vec<f64> = prices
        .iter()
        .map(|p| p.close_price.to_string().parse::<f64>().unwrap_or(0.0))
        .collect();

    let current_price = *price_values.last().unwrap_or(&0.0);

    if current_price == 0.0 {
        return Ok(all_results);
    }

    // Calculate RSI
    let rsi_values = indicators::rsi(&price_values, RSI_PERIOD);
    let current_rsi = rsi_values.last().and_then(|v| *v);

    // Calculate volatility (annualized std dev of returns)
    let volatility = if price_values.len() >= 20 {
        let returns: Vec<f64> = price_values
            .windows(2)
            .map(|w| if w[0] > 0.0 { (w[1] / w[0]).ln() } else { 0.0 })
            .collect();
        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
        Some(variance.sqrt() * (252.0_f64).sqrt() * 100.0) // Annualized percentage
    } else {
        None
    };

    // Volume ratio (placeholder - we don't have volume data in price_points)
    let volume_ratio: Option<f64> = None;

    // Get all watchlist items for this ticker
    let items_with_users = watchlist_queries::get_all_items_for_ticker(pool, ticker).await?;

    for (item, user_id) in &items_with_users {
        // 1. Check custom thresholds
        let threshold_results = check_thresholds(
            pool,
            item,
            *user_id,
            current_price,
            current_rsi,
            volume_ratio,
            volatility,
        )
        .await?;
        all_results.extend(threshold_results);

        // 2. Detect technical patterns (with cooldown check)
        let has_recent_pattern = watchlist_queries::has_recent_alert(
            pool,
            item.id,
            "pattern_",
            ALERT_COOLDOWN_HOURS,
        )
        .await
        .unwrap_or(true);

        if !has_recent_pattern {
            let pattern_results = detect_patterns(&price_values, ticker, item.id, *user_id);
            all_results.extend(pattern_results);
        }

        // 3. Detect sentiment shifts
        let prev_state = watchlist_queries::get_monitoring_state(pool, item.id).await?;
        let prev_sentiment = prev_state.as_ref().and_then(|s| s.last_sentiment_score);

        // We don't have a direct sentiment score here, but we can use the monitoring state
        // In a full implementation, this would query the sentiment service
        if let Some(prev_s) = prev_sentiment {
            // Use a placeholder current sentiment (would come from sentiment service in production)
            // For now, skip actual sentiment shift detection unless we have data
            let _ = prev_s; // acknowledge we have the data
        }

        // 4. Update monitoring state
        watchlist_queries::upsert_monitoring_state(
            pool,
            item.id,
            Some(current_price),
            current_rsi,
            volume_ratio,
            volatility,
            None, // sentiment not computed here
        )
        .await?;
    }

    Ok(all_results)
}

// ==============================================================================
// Helper Functions
// ==============================================================================

fn determine_severity(threshold_type: &str, actual: f64, threshold: f64) -> String {
    let ratio = if threshold != 0.0 {
        (actual / threshold).abs()
    } else {
        1.0
    };

    match threshold_type {
        "price_above" | "price_below" => {
            if ratio >= 1.1 { "high" } else { "medium" }
        }
        "price_change_pct" => {
            if actual.abs() >= 10.0 {
                "critical"
            } else if actual.abs() >= 5.0 {
                "high"
            } else {
                "medium"
            }
        }
        "volatility" => {
            if ratio >= 1.5 { "high" } else { "medium" }
        }
        "volume_spike" => {
            if actual >= 3.0 { "high" } else { "medium" }
        }
        "rsi_overbought" | "rsi_oversold" => {
            if actual >= 85.0 || actual <= 15.0 {
                "high"
            } else {
                "medium"
            }
        }
        _ => "medium",
    }
    .to_string()
}

fn format_alert_message(
    ticker: &str,
    threshold_type: &str,
    actual: f64,
    threshold: f64,
) -> String {
    match threshold_type {
        "price_above" => format!(
            "{}: Price ${:.2} exceeded upper threshold ${:.2}",
            ticker, actual, threshold
        ),
        "price_below" => format!(
            "{}: Price ${:.2} dropped below threshold ${:.2}",
            ticker, actual, threshold
        ),
        "price_change_pct" => format!(
            "{}: Price changed {:.2}% (threshold: {:.2}%)",
            ticker, actual, threshold
        ),
        "volatility" => format!(
            "{}: Volatility at {:.2}% exceeds threshold {:.2}%",
            ticker, actual, threshold
        ),
        "volume_spike" => format!(
            "{}: Volume ratio at {:.2}x exceeds threshold {:.2}x",
            ticker, actual, threshold
        ),
        "rsi_overbought" => format!(
            "{}: RSI at {:.1} exceeds overbought threshold {:.1}",
            ticker, actual, threshold
        ),
        "rsi_oversold" => format!(
            "{}: RSI at {:.1} below oversold threshold {:.1}",
            ticker, actual, threshold
        ),
        _ => format!(
            "{}: {} alert - value {:.2} (threshold: {:.2})",
            ticker, threshold_type, actual, threshold
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_severity_price_change() {
        assert_eq!(determine_severity("price_change_pct", 12.0, 5.0), "critical");
        assert_eq!(determine_severity("price_change_pct", 7.0, 5.0), "high");
        assert_eq!(determine_severity("price_change_pct", 3.0, 2.0), "medium");
    }

    #[test]
    fn test_determine_severity_rsi() {
        assert_eq!(determine_severity("rsi_overbought", 90.0, 70.0), "high");
        assert_eq!(determine_severity("rsi_oversold", 12.0, 30.0), "high");
        assert_eq!(determine_severity("rsi_overbought", 75.0, 70.0), "medium");
    }

    #[test]
    fn test_format_alert_message() {
        let msg = format_alert_message("AAPL", "price_above", 155.0, 150.0);
        assert!(msg.contains("AAPL"));
        assert!(msg.contains("155.00"));
        assert!(msg.contains("150.00"));
    }

    #[test]
    fn test_detect_patterns_insufficient_data() {
        let prices = vec![100.0; 10]; // Only 10 prices, need 30
        let results = detect_patterns(
            &prices,
            "AAPL",
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_detect_sentiment_shift() {
        let item_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();

        // No previous sentiment
        assert!(detect_sentiment_shift("AAPL", item_id, user_id, 0.5, None).is_none());

        // Small shift (below threshold)
        assert!(detect_sentiment_shift("AAPL", item_id, user_id, 0.5, Some(0.4)).is_none());

        // Significant positive shift
        let result = detect_sentiment_shift("AAPL", item_id, user_id, 0.5, Some(-0.1));
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.alert_type.contains("positive"));

        // Significant negative shift
        let result = detect_sentiment_shift("AAPL", item_id, user_id, -0.3, Some(0.4));
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.alert_type.contains("negative"));
        assert_eq!(r.severity, "high"); // shift of 0.7 >= 0.6
    }
}

// Alert service with real price change detection
// Full implementation would integrate deeply with existing risk and sentiment services

use crate::db::alert_queries::*;
use crate::db::price_queries;
use crate::models::alert::*;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// ==============================================================================
// Alert Evaluation (Simplified)
// ==============================================================================

/// Evaluate all active alerts for a user
pub async fn evaluate_all_alerts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AlertEvaluationResult>, sqlx::Error> {
    let rules = get_active_alert_rules_for_user(pool, user_id).await?;

    let mut results = Vec::new();
    for rule in rules {
        if let Some(result) = evaluate_alert_rule_simple(pool, &rule).await? {
            results.push(result);
        }
    }

    Ok(results)
}

/// Simplified alert evaluation (placeholder logic)
pub async fn evaluate_alert_rule_simple(
    pool: &PgPool,
    rule: &AlertRule,
) -> Result<Option<AlertEvaluationResult>, sqlx::Error> {
    // Check cooldown first
    if is_in_cooldown(rule.last_triggered_at, rule.cooldown_hours) {
        return Ok(None);
    }

    // Parse rule_type JSON to get the AlertType enum with config
    let alert_type: AlertType = serde_json::from_str(&rule.rule_type)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to parse rule_type: {}", e)))?;

    // Parse comparison
    let comparison = Comparison::from_str(&rule.comparison)
        .ok_or_else(|| sqlx::Error::Protocol(format!("Invalid comparison: {}", rule.comparison)))?;

    // Evaluate based on alert type
    let (triggered, actual_value, message, threshold) = match alert_type {
        AlertType::PriceChange { percentage, direction: _, timeframe: _ } => {
            // Real price change detection using configured percentage
            if let Some(ticker) = &rule.ticker {
                match calculate_price_change(pool, ticker).await {
                    Ok(Some(change_pct)) => {
                        let abs_change = change_pct.abs();
                        // Use the configured percentage as threshold
                        let triggered = comparison.evaluate(abs_change, percentage);
                        let direction_str = if change_pct > 0.0 { "increased" } else { "decreased" };
                        let message = format!(
                            "{} price {} by {:.2}% (threshold: {:.2}%)",
                            ticker, direction_str, abs_change, percentage
                        );
                        (triggered, abs_change, message, percentage)
                    }
                    Ok(None) => {
                        // No price data available
                        (false, 0.0, format!("{}: No recent price data available", ticker), percentage)
                    }
                    Err(e) => {
                        eprintln!("Error calculating price change for {}: {:?}", ticker, e);
                        (false, 0.0, format!("{}: Error fetching price data", ticker), percentage)
                    }
                }
            } else {
                (false, 0.0, "No ticker specified for price change alert".to_string(), percentage)
            }
        }
        AlertType::VolatilitySpike { threshold } => {
            let simulated_volatility = 45.0; // Would get from risk_service
            let triggered = comparison.evaluate(simulated_volatility, threshold);
            let message = format!(
                "Volatility: {:.2}% (threshold: {:.2}%)",
                simulated_volatility, threshold
            );
            (triggered, simulated_volatility, message, threshold)
        }
        AlertType::DrawdownExceeded { percentage } => {
            let simulated_drawdown = 12.0; // Would calculate from price history
            let triggered = comparison.evaluate(simulated_drawdown, percentage);
            let message = format!(
                "Drawdown: {:.2}% (threshold: {:.2}%)",
                simulated_drawdown, percentage
            );
            (triggered, simulated_drawdown, message, percentage)
        }
        AlertType::RiskThreshold { metric: _, threshold } => {
            let simulated_risk = 75.0; // Would get from risk_service
            let triggered = comparison.evaluate(simulated_risk, threshold);
            let message = format!(
                "Risk score: {:.2} (threshold: {:.2})",
                simulated_risk, threshold
            );
            (triggered, simulated_risk, message, threshold)
        }
        AlertType::SentimentChange { sentiment_threshold, trend: _ } => {
            let simulated_sentiment = -0.4; // Would get from sentiment_service
            let triggered = comparison.evaluate(simulated_sentiment, sentiment_threshold);
            let message = format!(
                "Sentiment: {:.2} (threshold: {:.2})",
                simulated_sentiment, sentiment_threshold
            );
            (triggered, simulated_sentiment, message, sentiment_threshold)
        }
        AlertType::Divergence { divergence_type: _ } => {
            let triggered = false; // Would check for divergence
            let message = "No divergence detected".to_string();
            (triggered, 0.0, message, 0.0)
        }
    };

    if triggered {
        let severity = calculate_severity(&rule.rule_type, threshold, actual_value);

        Ok(Some(AlertEvaluationResult {
            rule_id: rule.id,
            triggered: true,
            actual_value,
            threshold,  // Use the threshold from rule config
            message,
            severity,
            metadata: json!({
                "rule_type": rule.rule_type,
                "ticker": rule.ticker,
                "portfolio_id": rule.portfolio_id,
            }),
        }))
    } else {
        Ok(None)
    }
}

// ==============================================================================
// Helper Functions
// ==============================================================================

/// Calculate price change percentage for a ticker
async fn calculate_price_change(pool: &PgPool, ticker: &str) -> Result<Option<f64>, sqlx::Error> {
    // Get the last 2 days of prices to calculate daily change
    let prices = price_queries::fetch_window(pool, ticker, 2).await?;

    if prices.len() < 2 {
        // Need at least 2 data points
        return Ok(None);
    }

    // fetch_window returns DESC order (most recent first)
    // Convert BigDecimal to f64 for calculations
    let latest_price: f64 = prices[0].close_price.to_string().parse().unwrap_or(0.0);
    let previous_price: f64 = prices[1].close_price.to_string().parse().unwrap_or(0.0);

    if previous_price == 0.0 {
        return Ok(None);
    }

    // Calculate percentage change: ((new - old) / old) * 100
    let change_pct = ((latest_price - previous_price) / previous_price) * 100.0;

    Ok(Some(change_pct))
}

/// Check if alert is in cooldown period
pub fn is_in_cooldown(last_triggered: Option<DateTime<Utc>>, cooldown_hours: i32) -> bool {
    if let Some(last) = last_triggered {
        let cooldown_duration = Duration::hours(cooldown_hours as i64);
        let next_allowed = last + cooldown_duration;
        Utc::now() < next_allowed
    } else {
        false
    }
}

/// Calculate alert severity based on how far actual exceeds threshold
pub fn calculate_severity(rule_type: &str, threshold: f64, actual_value: f64) -> AlertSeverity {
    let ratio = if threshold > 0.0 {
        actual_value / threshold
    } else {
        1.0
    };

    match rule_type {
        "price_change" => {
            if ratio >= 2.0 {
                AlertSeverity::Critical
            } else if ratio >= 1.5 {
                AlertSeverity::High
            } else if ratio >= 1.2 {
                AlertSeverity::Medium
            } else {
                AlertSeverity::Low
            }
        }
        "volatility_spike" => {
            if ratio >= 2.0 {
                AlertSeverity::Critical
            } else if ratio >= 1.5 {
                AlertSeverity::High
            } else {
                AlertSeverity::Medium
            }
        }
        "drawdown_exceeded" => {
            if ratio >= 1.5 {
                AlertSeverity::Critical
            } else if ratio >= 1.2 {
                AlertSeverity::High
            } else {
                AlertSeverity::Medium
            }
        }
        "risk_threshold" => {
            if ratio >= 1.5 {
                AlertSeverity::High
            } else if ratio >= 1.2 {
                AlertSeverity::Medium
            } else {
                AlertSeverity::Low
            }
        }
        "sentiment_change" => AlertSeverity::Medium,
        "divergence" => AlertSeverity::High,
        _ => AlertSeverity::Medium,
    }
}

/// Process triggered alert - create history and prepare for notification
pub async fn process_triggered_alert(
    pool: &PgPool,
    rule: &AlertRule,
    result: &AlertEvaluationResult,
) -> Result<AlertHistory, sqlx::Error> {
    // Create alert history
    let alert_history = create_alert_history(
        pool,
        rule.id,
        rule.user_id,
        rule.portfolio_id,
        rule.ticker.as_deref(),
        &rule.rule_type,
        result.threshold,
        result.actual_value,
        &result.message,
        &result.severity.to_string(),
        result.metadata.clone(),
    )
    .await?;

    // Update last triggered timestamp
    update_rule_last_triggered(pool, rule.id).await?;

    Ok(alert_history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_cooldown() {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        assert!(is_in_cooldown(Some(one_hour_ago), 24));

        let two_days_ago = now - Duration::days(2);
        assert!(!is_in_cooldown(Some(two_days_ago), 24));

        assert!(!is_in_cooldown(None, 24));
    }

    #[test]
    fn test_calculate_severity() {
        assert_eq!(
            calculate_severity("price_change", 10.0, 25.0),
            AlertSeverity::Critical
        );
        assert_eq!(
            calculate_severity("price_change", 10.0, 16.0),
            AlertSeverity::High
        );
    }
}

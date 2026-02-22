// Simplified alert service - placeholder implementations for demonstration
// Full implementation would integrate deeply with existing risk and sentiment services

use crate::db::alert_queries::*;
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
    _pool: &PgPool,
    rule: &AlertRule,
) -> Result<Option<AlertEvaluationResult>, sqlx::Error> {
    // Check cooldown first
    if is_in_cooldown(rule.last_triggered_at, rule.cooldown_hours) {
        return Ok(None);
    }

    // Parse comparison
    let comparison = Comparison::from_str(&rule.comparison)
        .ok_or_else(|| sqlx::Error::Protocol(format!("Invalid comparison: {}", rule.comparison)))?;

    // Simplified evaluation - would integrate with actual services in production
    let (triggered, actual_value, message) = match rule.rule_type.as_str() {
        "price_change" => {
            // Placeholder: simulate price change detection
            let simulated_change = 5.5; // Would get from price_queries
            let triggered = comparison.evaluate(simulated_change, rule.threshold);
            let message = format!(
                "Price changed by {:.2}% (threshold: {:.2}%)",
                simulated_change, rule.threshold
            );
            (triggered, simulated_change, message)
        }
        "volatility_spike" => {
            let simulated_volatility = 45.0; // Would get from risk_service
            let triggered = comparison.evaluate(simulated_volatility, rule.threshold);
            let message = format!(
                "Volatility: {:.2}% (threshold: {:.2}%)",
                simulated_volatility, rule.threshold
            );
            (triggered, simulated_volatility, message)
        }
        "drawdown_exceeded" => {
            let simulated_drawdown = 12.0; // Would calculate from price history
            let triggered = comparison.evaluate(simulated_drawdown, rule.threshold);
            let message = format!(
                "Drawdown: {:.2}% (threshold: {:.2}%)",
                simulated_drawdown, rule.threshold
            );
            (triggered, simulated_drawdown, message)
        }
        "risk_threshold" => {
            let simulated_risk = 75.0; // Would get from risk_service
            let triggered = comparison.evaluate(simulated_risk, rule.threshold);
            let message = format!(
                "Risk score: {:.2} (threshold: {:.2})",
                simulated_risk, rule.threshold
            );
            (triggered, simulated_risk, message)
        }
        "sentiment_change" => {
            let simulated_sentiment = -0.4; // Would get from sentiment_service
            let triggered = comparison.evaluate(simulated_sentiment, rule.threshold);
            let message = format!(
                "Sentiment: {:.2} (threshold: {:.2})",
                simulated_sentiment, rule.threshold
            );
            (triggered, simulated_sentiment, message)
        }
        "divergence" => {
            let triggered = false; // Would check for divergence
            let message = "No divergence detected".to_string();
            (triggered, 0.0, message)
        }
        _ => (false, 0.0, "Unknown alert type".to_string()),
    };

    if triggered {
        let severity = calculate_severity(&rule.rule_type, rule.threshold, actual_value);

        Ok(Some(AlertEvaluationResult {
            rule_id: rule.id,
            triggered: true,
            actual_value,
            threshold: rule.threshold,
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

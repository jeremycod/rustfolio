use crate::models::alert::*;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

// ==============================================================================
// Alert Rules CRUD Operations
// ==============================================================================

pub async fn create_alert_rule(
    pool: &PgPool,
    user_id: Uuid,
    portfolio_id: Option<Uuid>,
    ticker: Option<String>,
    rule_type: &str,
    threshold: f64,
    comparison: &str,
    name: &str,
    description: Option<&str>,
    notification_channels: Vec<String>,
    cooldown_hours: i32,
) -> Result<AlertRule, sqlx::Error> {
    let rule = sqlx::query_as::<_, AlertRule>(
        r#"
        INSERT INTO alert_rules (
            user_id, portfolio_id, ticker, rule_type, threshold, comparison,
            name, description, notification_channels, cooldown_hours
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(portfolio_id)
    .bind(ticker)
    .bind(rule_type)
    .bind(threshold)
    .bind(comparison)
    .bind(name)
    .bind(description)
    .bind(&notification_channels)
    .bind(cooldown_hours)
    .fetch_one(pool)
    .await?;

    Ok(rule)
}

pub async fn get_alert_rule(pool: &PgPool, rule_id: Uuid) -> Result<AlertRule, sqlx::Error> {
    let rule = sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT * FROM alert_rules WHERE id = $1
        "#,
    )
    .bind(rule_id)
    .fetch_one(pool)
    .await?;

    Ok(rule)
}

pub async fn get_alert_rules_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AlertRule>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT * FROM alert_rules
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

pub async fn get_active_alert_rules_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AlertRule>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT * FROM alert_rules
        WHERE user_id = $1 AND enabled = TRUE
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

#[allow(dead_code)]
pub async fn get_all_active_alert_rules(pool: &PgPool) -> Result<Vec<AlertRule>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT * FROM alert_rules
        WHERE enabled = TRUE
        ORDER BY user_id, created_at
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

#[allow(dead_code)]
pub async fn get_alert_rules_for_portfolio(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<AlertRule>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT * FROM alert_rules
        WHERE portfolio_id = $1 AND enabled = TRUE
        ORDER BY created_at DESC
        "#,
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

pub async fn update_alert_rule(
    pool: &PgPool,
    rule_id: Uuid,
    threshold: Option<f64>,
    comparison: Option<&str>,
    enabled: Option<bool>,
    name: Option<&str>,
    description: Option<&str>,
    notification_channels: Option<Vec<String>>,
    cooldown_hours: Option<i32>,
) -> Result<AlertRule, sqlx::Error> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE alert_rules SET ");

    let mut separated = query_builder.separated(", ");
    let mut has_updates = false;

    if let Some(threshold) = threshold {
        separated.push("threshold = ");
        separated.push_bind_unseparated(threshold);
        has_updates = true;
    }

    if let Some(comparison) = comparison {
        separated.push("comparison = ");
        separated.push_bind_unseparated(comparison);
        has_updates = true;
    }

    if let Some(enabled) = enabled {
        separated.push("enabled = ");
        separated.push_bind_unseparated(enabled);
        has_updates = true;
    }

    if let Some(name) = name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
        has_updates = true;
    }

    if let Some(description) = description {
        separated.push("description = ");
        separated.push_bind_unseparated(description);
        has_updates = true;
    }

    if let Some(channels) = notification_channels {
        separated.push("notification_channels = ");
        separated.push_bind_unseparated(channels);
        has_updates = true;
    }

    if let Some(cooldown) = cooldown_hours {
        separated.push("cooldown_hours = ");
        separated.push_bind_unseparated(cooldown);
        has_updates = true;
    }

    if !has_updates {
        return get_alert_rule(pool, rule_id).await;
    }

    query_builder.push(", updated_at = NOW() WHERE id = ");
    query_builder.push_bind(rule_id);
    query_builder.push(" RETURNING *");

    let rule = query_builder
        .build_query_as::<AlertRule>()
        .fetch_one(pool)
        .await?;

    Ok(rule)
}

pub async fn delete_alert_rule(pool: &PgPool, rule_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM alert_rules WHERE id = $1")
        .bind(rule_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_rule_last_triggered(
    pool: &PgPool,
    rule_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE alert_rules
        SET last_triggered_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(rule_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ==============================================================================
// Alert History Operations
// ==============================================================================

pub async fn create_alert_history(
    pool: &PgPool,
    alert_rule_id: Uuid,
    user_id: Uuid,
    portfolio_id: Option<Uuid>,
    ticker: Option<&str>,
    rule_type: &str,
    threshold: f64,
    actual_value: f64,
    message: &str,
    severity: &str,
    metadata: serde_json::Value,
) -> Result<AlertHistory, sqlx::Error> {
    let alert = sqlx::query_as::<_, AlertHistory>(
        r#"
        INSERT INTO alert_history (
            alert_rule_id, user_id, portfolio_id, ticker, rule_type,
            threshold, actual_value, message, severity, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(alert_rule_id)
    .bind(user_id)
    .bind(portfolio_id)
    .bind(ticker)
    .bind(rule_type)
    .bind(threshold)
    .bind(actual_value)
    .bind(message)
    .bind(severity)
    .bind(metadata)
    .fetch_one(pool)
    .await?;

    Ok(alert)
}

pub async fn get_alert_history_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<AlertHistory>, sqlx::Error> {
    let limit = limit.unwrap_or(100).min(1000);
    let offset = offset.unwrap_or(0);

    let alerts = sqlx::query_as::<_, AlertHistory>(
        r#"
        SELECT * FROM alert_history
        WHERE user_id = $1
        ORDER BY triggered_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

pub async fn get_alert_history_for_rule(
    pool: &PgPool,
    rule_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<AlertHistory>, sqlx::Error> {
    let limit = limit.unwrap_or(100).min(1000);

    let alerts = sqlx::query_as::<_, AlertHistory>(
        r#"
        SELECT * FROM alert_history
        WHERE alert_rule_id = $1
        ORDER BY triggered_at DESC
        LIMIT $2
        "#,
    )
    .bind(rule_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

#[allow(dead_code)]
pub async fn get_recent_alert_history(
    pool: &PgPool,
    user_id: Uuid,
    days: i32,
) -> Result<Vec<AlertHistory>, sqlx::Error> {
    let alerts = sqlx::query_as::<_, AlertHistory>(
        r#"
        SELECT * FROM alert_history
        WHERE user_id = $1
          AND triggered_at >= NOW() - ($2 || ' days')::INTERVAL
        ORDER BY triggered_at DESC
        "#,
    )
    .bind(user_id)
    .bind(days)
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

#[allow(dead_code)]
pub async fn count_user_alerts(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM alert_history
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

// ==============================================================================
// Notification Operations
// ==============================================================================

pub async fn create_notification(
    pool: &PgPool,
    user_id: Uuid,
    alert_history_id: Option<Uuid>,
    title: &str,
    message: &str,
    notification_type: &str,
    link: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Notification, sqlx::Error> {
    let notification = sqlx::query_as::<_, Notification>(
        r#"
        INSERT INTO notifications (
            user_id, alert_history_id, title, message, notification_type, link, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(alert_history_id)
    .bind(title)
    .bind(message)
    .bind(notification_type)
    .bind(link)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(notification)
}

pub async fn get_user_notifications(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Notification>, sqlx::Error> {
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    let notifications = sqlx::query_as::<_, Notification>(
        r#"
        SELECT * FROM notifications
        WHERE user_id = $1
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

#[allow(dead_code)]
pub async fn get_unread_notifications(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Notification>, sqlx::Error> {
    let notifications = sqlx::query_as::<_, Notification>(
        r#"
        SELECT * FROM notifications
        WHERE user_id = $1
          AND read = FALSE
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

pub async fn mark_notification_read(
    pool: &PgPool,
    notification_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE notifications
        SET read = TRUE, read_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(notification_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_all_notifications_read(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE notifications
        SET read = TRUE, read_at = NOW()
        WHERE user_id = $1 AND read = FALSE
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_notification(pool: &PgPool, notification_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notifications WHERE id = $1")
        .bind(notification_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn count_user_notifications(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(i64, i64), sqlx::Error> {
    let counts: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS total,
            COUNT(*) FILTER (WHERE read = FALSE) AS unread
        FROM notifications
        WHERE user_id = $1
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(counts)
}

// ==============================================================================
// Notification Preferences Operations
// ==============================================================================

pub async fn get_notification_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<NotificationPreferences>, sqlx::Error> {
    let prefs = sqlx::query_as::<_, NotificationPreferences>(
        r#"
        SELECT * FROM notification_preferences WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(prefs)
}

pub async fn get_or_create_notification_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<NotificationPreferences, sqlx::Error> {
    // Try to get existing preferences
    if let Some(prefs) = get_notification_preferences(pool, user_id).await? {
        return Ok(prefs);
    }

    // Create default preferences
    let prefs = sqlx::query_as::<_, NotificationPreferences>(
        r#"
        INSERT INTO notification_preferences (user_id)
        VALUES ($1)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(prefs)
}

pub async fn update_notification_preferences(
    pool: &PgPool,
    user_id: Uuid,
    email_enabled: Option<bool>,
    in_app_enabled: Option<bool>,
    webhook_enabled: Option<bool>,
    webhook_url: Option<&str>,
    quiet_hours_start: Option<NaiveTime>,
    quiet_hours_end: Option<NaiveTime>,
    timezone: Option<&str>,
    max_daily_emails: Option<i32>,
) -> Result<NotificationPreferences, sqlx::Error> {
    // Ensure preferences exist
    get_or_create_notification_preferences(pool, user_id).await?;

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("UPDATE notification_preferences SET ");

    let mut separated = query_builder.separated(", ");
    let mut has_updates = false;

    if let Some(enabled) = email_enabled {
        separated.push("email_enabled = ");
        separated.push_bind_unseparated(enabled);
        has_updates = true;
    }

    if let Some(enabled) = in_app_enabled {
        separated.push("in_app_enabled = ");
        separated.push_bind_unseparated(enabled);
        has_updates = true;
    }

    if let Some(enabled) = webhook_enabled {
        separated.push("webhook_enabled = ");
        separated.push_bind_unseparated(enabled);
        has_updates = true;
    }

    if let Some(url) = webhook_url {
        separated.push("webhook_url = ");
        separated.push_bind_unseparated(url);
        has_updates = true;
    }

    if let Some(start) = quiet_hours_start {
        separated.push("quiet_hours_start = ");
        separated.push_bind_unseparated(start);
        has_updates = true;
    }

    if let Some(end) = quiet_hours_end {
        separated.push("quiet_hours_end = ");
        separated.push_bind_unseparated(end);
        has_updates = true;
    }

    if let Some(tz) = timezone {
        separated.push("timezone = ");
        separated.push_bind_unseparated(tz);
        has_updates = true;
    }

    if let Some(max) = max_daily_emails {
        separated.push("max_daily_emails = ");
        separated.push_bind_unseparated(max);
        has_updates = true;
    }

    if !has_updates {
        return get_or_create_notification_preferences(pool, user_id).await;
    }

    query_builder.push(", updated_at = NOW() WHERE user_id = ");
    query_builder.push_bind(user_id);
    query_builder.push(" RETURNING *");

    let prefs = query_builder
        .build_query_as::<NotificationPreferences>()
        .fetch_one(pool)
        .await?;

    Ok(prefs)
}

// ==============================================================================
// Email Rate Limiting Operations
// ==============================================================================

pub async fn get_daily_email_count(pool: &PgPool, user_id: Uuid) -> Result<i32, sqlx::Error> {
    let count: (i32,) = sqlx::query_as(
        r#"
        SELECT get_daily_email_count($1)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

pub async fn increment_daily_email_count(pool: &PgPool, user_id: Uuid) -> Result<i32, sqlx::Error> {
    let count: (i32,) = sqlx::query_as(
        r#"
        SELECT increment_daily_email_count($1)
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

// ==============================================================================
// User Operations (minimal)
// ==============================================================================

pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

#[allow(dead_code)]
pub async fn create_user(pool: &PgPool, email: &str, name: Option<&str>) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(name)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

use crate::db::alert_queries::*;
use crate::models::alert::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use lettre::{
    message::{header::ContentType, MultiPart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use std::env;

// ==============================================================================
// Notification Service
// ==============================================================================

/// Send notification through all enabled channels
pub async fn send_notification(
    pool: &PgPool,
    user_id: Uuid,
    alert: &AlertHistory,
) -> Result<(), sqlx::Error> {
    // Get user preferences
    let prefs = get_or_create_notification_preferences(pool, user_id).await?;

    // Get user info for email
    let user = get_user(pool, user_id).await?;

    // Check each channel
    if prefs.in_app_enabled {
        if should_send_in_app_notification(pool, user_id, &prefs).await? {
            create_in_app_notification(pool, user_id, alert).await?;
        }
    }

    if prefs.email_enabled {
        if should_send_email_notification(pool, user_id, &prefs).await? {
            send_email_notification(pool, &user.email, alert, &prefs).await?;
        }
    }

    if prefs.webhook_enabled {
        if let Some(webhook_url) = &prefs.webhook_url {
            send_webhook_notification(webhook_url, alert).await?;
        }
    }

    Ok(())
}

// ==============================================================================
// In-App Notifications
// ==============================================================================

/// Create in-app notification
pub async fn create_in_app_notification(
    pool: &PgPool,
    user_id: Uuid,
    alert: &AlertHistory,
) -> Result<Notification, sqlx::Error> {
    let title = if let Some(ticker) = &alert.ticker {
        format!("🚨 {} Alert: {}", format_rule_type(&alert.rule_type), ticker)
    } else {
        format!("🚨 {} Alert", format_rule_type(&alert.rule_type))
    };

    let link = if let Some(portfolio_id) = alert.portfolio_id {
        Some(format!("/portfolios/{}", portfolio_id))
    } else if let Some(ticker) = &alert.ticker {
        Some(format!("/positions/{}", ticker))
    } else {
        None
    };

    let notification = create_notification(
        pool,
        user_id,
        Some(alert.id),
        &title,
        &alert.message,
        "alert",
        link.as_deref(),
        None,
    )
    .await?;

    Ok(notification)
}

/// Check if in-app notification should be sent
async fn should_send_in_app_notification(
    _pool: &PgPool,
    _user_id: Uuid,
    prefs: &NotificationPreferences,
) -> Result<bool, sqlx::Error> {
    // Check quiet hours
    if is_in_quiet_hours(prefs) {
        return Ok(false);
    }

    Ok(true)
}

// ==============================================================================
// Email Notifications
// ==============================================================================

/// Send email notification
pub async fn send_email_notification(
    pool: &PgPool,
    to_email: &str,
    alert: &AlertHistory,
    _prefs: &NotificationPreferences,
) -> Result<(), sqlx::Error> {
    // Increment email count
    let user = get_user_by_email(pool, to_email)
        .await?
        .ok_or_else(|| sqlx::Error::Protocol("User not found".to_string()))?;

    let new_count = increment_daily_email_count(pool, user.id).await?;

    // Check if SMTP is enabled
    let smtp_enabled = env::var("SMTP_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    if smtp_enabled {
        // Try to send actual email via SMTP
        match send_email_via_smtp(to_email, alert).await {
            Ok(_) => {
                println!("✅ Email sent successfully to {} (#{}) via SMTP", to_email, new_count);
            }
            Err(e) => {
                eprintln!("❌ Failed to send email via SMTP: {}", e);
                log_email_notification(to_email, alert, new_count);
            }
        }
    } else {
        // Log email would be sent (SMTP disabled)
        log_email_notification(to_email, alert, new_count);
    }

    Ok(())
}

/// Check if email notification should be sent
async fn should_send_email_notification(
    pool: &PgPool,
    user_id: Uuid,
    prefs: &NotificationPreferences,
) -> Result<bool, sqlx::Error> {
    // Check quiet hours
    if is_in_quiet_hours(prefs) {
        return Ok(false);
    }

    // Check daily limit
    let count = get_daily_email_count(pool, user_id).await?;
    if count >= prefs.max_daily_emails {
        return Ok(false);
    }

    Ok(true)
}

/// Log email notification (fallback when SMTP is disabled)
fn log_email_notification(to_email: &str, alert: &AlertHistory, count: i32) {
    let subject = if let Some(ticker) = &alert.ticker {
        format!("{} Alert: {}", format_rule_type(&alert.rule_type), ticker)
    } else {
        format!("{} Alert", format_rule_type(&alert.rule_type))
    };

    println!("📧 Email notification #{} would be sent:", count);
    println!("   To: {}", to_email);
    println!("   Subject: {}", subject);
    if let Some(ticker) = &alert.ticker {
        println!("   Ticker: {}", ticker);
    }
    println!("   Message: {}", alert.message);
    println!("   Threshold: {:.2}% | Actual: {:.2}%", alert.threshold, alert.actual_value);
    println!("   Severity: {}", alert.severity);
    println!("   Triggered: {}", alert.triggered_at);
    println!();
}

/// Send email via SMTP using lettre
async fn send_email_via_smtp(to_email: &str, alert: &AlertHistory) -> Result<(), Box<dyn std::error::Error>> {
    // Get SMTP configuration from environment
    let smtp_host = env::var("SMTP_HOST")?;
    let smtp_port = env::var("SMTP_PORT")?.parse::<u16>()?;
    let smtp_username = env::var("SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD")?;
    let smtp_from_email = env::var("SMTP_FROM_EMAIL")?;
    let smtp_from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Rustfolio".to_string());

    // Build email subject with ticker or portfolio info
    let subject = if let Some(ticker) = &alert.ticker {
        format!("🚨 {} Alert: {}", format_rule_type(&alert.rule_type), ticker)
    } else if alert.portfolio_id.is_some() {
        format!("🚨 {} Alert: Portfolio", format_rule_type(&alert.rule_type))
    } else {
        format!("🚨 {} Alert", format_rule_type(&alert.rule_type))
    };

    // Build HTML email body
    let html_body = build_email_html(alert, "http://localhost:5173");

    // Build plain text version with ticker info
    let scope_text = if let Some(ticker) = &alert.ticker {
        format!("Ticker: {}\n", ticker)
    } else if alert.portfolio_id.is_some() {
        "Scope: Portfolio-wide\n".to_string()
    } else {
        "".to_string()
    };

    let text_body = format!(
        "{}\n\n{}{}\n\nThreshold: {:.2}%\nActual Value: {:.2}%\nSeverity: {}\nTriggered: {}\n\nView details at: http://localhost:5173",
        subject,
        scope_text,
        alert.message,
        alert.threshold,
        alert.actual_value,
        alert.severity.to_uppercase(),
        alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Build email message
    let from_address = format!("{} <{}>", smtp_from_name, smtp_from_email)
        .parse()
        .map_err(|e| format!("Invalid from address: {}", e))?;

    let to_address = to_email
        .parse()
        .map_err(|e| format!("Invalid to address: {}", e))?;

    let email = Message::builder()
        .from(from_address)
        .to(to_address)
        .subject(&subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text_body)
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_body)
                ),
        )
        .map_err(|e| format!("Failed to build email: {}", e))?;

    // Create SMTP transport
    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    println!("🔌 Connecting to SMTP server: {}:{}", smtp_host, smtp_port);
    println!("👤 Username: {}", smtp_username);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .port(smtp_port)
        .credentials(creds)
        .build();

    // Send email
    println!("📤 Sending email to {}...", to_email);
    match mailer.send(&email) {
        Ok(_) => {
            println!("✅ Email sent successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ SMTP Error: {:?}", e);
            Err(format!("SMTP send failed: {}. Check your Gmail App Password and ensure 2FA is enabled.", e).into())
        }
    }
}

/// Send test email (for testing SMTP configuration)
pub async fn send_test_email(pool: &PgPool, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    // Get user info
    let user = get_user(pool, user_id).await?;

    // Check if SMTP is enabled
    let smtp_enabled = env::var("SMTP_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    if !smtp_enabled {
        return Err("SMTP is not enabled. Set SMTP_ENABLED=true in .env file".into());
    }

    // Create a test alert for email
    let test_alert = AlertHistory {
        id: Uuid::new_v4(),
        alert_rule_id: Uuid::new_v4(),
        user_id,
        portfolio_id: None,
        ticker: Some("TEST".to_string()),
        rule_type: "test".to_string(),
        threshold: 100.0,
        actual_value: 105.5,
        message: "This is a test email from Rustfolio alert system. If you receive this, your SMTP configuration is working correctly!".to_string(),
        severity: "low".to_string(),
        metadata: serde_json::json!({}),
        triggered_at: Utc::now(),
        created_at: Utc::now(),
    };

    send_email_via_smtp(&user.email, &test_alert).await?;

    Ok(())
}

// ==============================================================================
// Webhook Notifications
// ==============================================================================

/// Send webhook notification
async fn send_webhook_notification(
    webhook_url: &str,
    alert: &AlertHistory,
) -> Result<(), sqlx::Error> {
    // Log webhook would be sent (actual sending would be implemented with reqwest)
    log_webhook_notification(webhook_url, alert);

    // In production, this would use reqwest:
    // let client = reqwest::Client::new();
    // let payload = build_webhook_payload(alert);
    // client.post(webhook_url).json(&payload).send().await?;

    Ok(())
}

/// Log webhook notification (placeholder for actual HTTP POST)
fn log_webhook_notification(webhook_url: &str, alert: &AlertHistory) {
    println!("🔔 Webhook notification would be sent:");
    println!("   URL: {}", webhook_url);
    println!("   Alert ID: {}", alert.id);
    println!("   Message: {}", alert.message);
    println!();
}

// ==============================================================================
// Helper Functions
// ==============================================================================

/// Check if current time is in user's quiet hours
fn is_in_quiet_hours(prefs: &NotificationPreferences) -> bool {
    if prefs.quiet_hours_start.is_none() || prefs.quiet_hours_end.is_none() {
        return false;
    }

    let start = prefs.quiet_hours_start.unwrap();
    let end = prefs.quiet_hours_end.unwrap();

    // Get current time in user's timezone
    // For now, using UTC (would use chrono-tz in production)
    let now = Utc::now().time();

    // Handle cases where quiet hours span midnight
    if start < end {
        now >= start && now <= end
    } else {
        now >= start || now <= end
    }
}

/// Format rule type for display
fn format_rule_type(rule_type: &str) -> String {
    match rule_type {
        "price_change" => "Price Change",
        "volatility_spike" => "Volatility Spike",
        "drawdown_exceeded" => "Drawdown Exceeded",
        "risk_threshold" => "Risk Threshold",
        "sentiment_change" => "Sentiment Change",
        "divergence" => "Divergence",
        _ => "Alert",
    }
    .to_string()
}

// ==============================================================================
// Email Template
// ==============================================================================

fn build_email_html(alert: &AlertHistory, app_url: &str) -> String {
    let severity_color = match alert.severity.as_str() {
        "critical" => "#d32f2f",
        "high" => "#f44336",
        "medium" => "#ff9800",
        "low" => "#2196f3",
        _ => "#757575",
    };

    let link = if let Some(portfolio_id) = alert.portfolio_id {
        format!("{}/portfolios/{}", app_url, portfolio_id)
    } else if let Some(ticker) = &alert.ticker {
        format!("{}/positions/{}", app_url, ticker)
    } else {
        app_url.to_string()
    };

    // Build scope display (ticker or portfolio)
    let scope_display = if let Some(ticker) = &alert.ticker {
        format!("Ticker: <strong style=\"font-size: 18px; color: #1976d2;\">{}</strong>", ticker)
    } else if alert.portfolio_id.is_some() {
        "Portfolio-wide alert".to_string()
    } else {
        "System alert".to_string()
    };

    // Build header title with ticker
    let header_title = if let Some(ticker) = &alert.ticker {
        format!("🚨 {} Alert: {}", format_rule_type(&alert.rule_type), ticker)
    } else {
        format!("🚨 {} Alert", format_rule_type(&alert.rule_type))
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 0; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: {}; color: white; padding: 20px; border-radius: 5px 5px 0 0; }}
        .content {{ padding: 20px; background-color: #f9f9f9; border: 1px solid #ddd; border-top: none; }}
        .footer {{ padding: 10px; text-align: center; color: #666; font-size: 12px; }}
        .button {{ display: inline-block; background-color: #2196f3; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; margin-top: 15px; }}
        .ticker-badge {{ background-color: #e3f2fd; border-left: 4px solid #1976d2; padding: 12px; margin: 15px 0; border-radius: 4px; }}
        table {{ width: 100%; margin: 15px 0; }}
        td {{ padding: 8px; }}
        .label {{ font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{}</h1>
        </div>
        <div class="content">
            <div class="ticker-badge">
                {}
            </div>

            <p><strong>{}</strong></p>

            <table>
                <tr>
                    <td class="label">Threshold:</td>
                    <td>{:.2}%</td>
                </tr>
                <tr>
                    <td class="label">Actual Value:</td>
                    <td>{:.2}%</td>
                </tr>
                <tr>
                    <td class="label">Severity:</td>
                    <td style="color: {}; font-weight: bold; text-transform: uppercase;">{}</td>
                </tr>
                <tr>
                    <td class="label">Triggered At:</td>
                    <td>{}</td>
                </tr>
            </table>

            <a href="{}" class="button">View Details</a>
        </div>
        <div class="footer">
            <p>You're receiving this because you have alert notifications enabled.</p>
            <p>Manage your preferences in your account settings.</p>
            <p>© 2026 Rustfolio - Portfolio Risk Management</p>
        </div>
    </div>
</body>
</html>
"#,
        severity_color,
        header_title,
        scope_display,
        alert.message,
        alert.threshold,
        alert.actual_value,
        severity_color,
        alert.severity,
        alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
        link
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_quiet_hours() {
        use chrono::NaiveTime;

        let mut prefs = NotificationPreferences {
            user_id: Uuid::new_v4(),
            email_enabled: true,
            in_app_enabled: true,
            webhook_enabled: false,
            webhook_url: None,
            quiet_hours_start: Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
            quiet_hours_end: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            timezone: "UTC".to_string(),
            max_daily_emails: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test with no quiet hours
        prefs.quiet_hours_start = None;
        prefs.quiet_hours_end = None;
        assert!(!is_in_quiet_hours(&prefs));
    }

    #[test]
    fn test_format_rule_type() {
        assert_eq!(format_rule_type("price_change"), "Price Change");
        assert_eq!(format_rule_type("volatility_spike"), "Volatility Spike");
        assert_eq!(format_rule_type("unknown"), "Alert");
    }
}

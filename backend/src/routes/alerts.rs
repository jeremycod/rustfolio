use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::alert_queries;
use crate::models::alert::*;
use crate::services::{alert_service, notification_service};
use crate::state::AppState;

// ==============================================================================
// Router
// ==============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // Alert Rules
        .route("/alerts/rules", post(create_alert_rule))
        .route("/alerts/rules", get(list_alert_rules))
        .route("/alerts/rules/:id", get(get_alert_rule))
        .route("/alerts/rules/:id", put(update_alert_rule))
        .route("/alerts/rules/:id", delete(delete_alert_rule))
        .route("/alerts/rules/:id/enable", post(enable_alert_rule))
        .route("/alerts/rules/:id/disable", post(disable_alert_rule))
        .route("/alerts/rules/:id/test", post(test_alert_rule))
        // Alert History
        .route("/alerts/history", get(get_alert_history))
        .route("/alerts/history/:id", get(get_alert_history_by_id))
        .route("/alerts/rules/:rule_id/history", get(get_rule_history))
        // Notifications
        .route("/notifications", get(get_notifications))
        .route("/notifications/unread", get(get_unread_count))
        .route("/notifications/:id/read", post(mark_notification_read))
        .route("/notifications/mark-all-read", post(mark_all_read))
        .route("/notifications/:id", delete(delete_notification))
        // Notification Preferences
        .route("/notifications/preferences", get(get_preferences))
        .route("/notifications/preferences", put(update_preferences))
        // Test Email
        .route("/notifications/test-email", post(send_test_email))
        // Evaluation
        .route("/alerts/evaluate-all", post(evaluate_all_alerts))
}

// ==============================================================================
// Query Parameters
// ==============================================================================

#[derive(Debug, Deserialize)]
struct PaginationParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DaysParam {
    days: Option<i32>,
}

// ==============================================================================
// Alert Rules Handlers
// ==============================================================================

async fn create_alert_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateAlertRuleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    // TODO: Get user_id from auth middleware
    let user_id = get_default_user_id(pool).await?;

    // Serialize the full AlertType to JSON to preserve config
    let rule_type_json = serde_json::to_string(&req.rule_type)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize rule_type: {}", e)))?;

    let comparison_str = req.comparison.to_string();
    let channels: Vec<String> = req
        .notification_channels
        .unwrap_or_else(|| vec![NotificationChannel::Email, NotificationChannel::InApp])
        .iter()
        .map(|c| c.to_string())
        .collect();

    let rule = alert_queries::create_alert_rule(
        pool,
        user_id,
        req.portfolio_id,
        req.ticker,
        &rule_type_json,
        req.threshold,
        &comparison_str,
        &req.name,
        req.description.as_deref(),
        channels,
        req.cooldown_hours.unwrap_or(24),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(AlertRuleResponse::from(rule))))
}

async fn list_alert_rules(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let rules = alert_queries::get_alert_rules_for_user(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<AlertRuleResponse> = rules.into_iter().map(AlertRuleResponse::from).collect();

    Ok(Json(responses))
}

async fn get_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let rule = alert_queries::get_alert_rule(pool, id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(AlertRuleResponse::from(rule)))
}

async fn update_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAlertRuleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let comparison_str = req.comparison.as_ref().map(|c| c.to_string());
    let channels = req
        .notification_channels
        .as_ref()
        .map(|chs| chs.iter().map(|c| c.to_string()).collect());

    let rule = alert_queries::update_alert_rule(
        pool,
        id,
        req.threshold,
        comparison_str.as_deref(),
        req.enabled,
        req.name.as_deref(),
        req.description.as_deref(),
        channels,
        req.cooldown_hours,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AlertRuleResponse::from(rule)))
}

async fn delete_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    alert_queries::delete_alert_rule(pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn enable_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let rule = alert_queries::update_alert_rule(
        pool,
        id,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AlertRuleResponse::from(rule)))
}

async fn disable_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let rule = alert_queries::update_alert_rule(
        pool,
        id,
        None,
        None,
        Some(false),
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AlertRuleResponse::from(rule)))
}

async fn test_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let rule = alert_queries::get_alert_rule(pool, id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let result = alert_service::evaluate_alert_rule_simple(pool, &rule)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    #[derive(Serialize)]
    struct TestResponse {
        rule: AlertRuleResponse,
        evaluation: Option<AlertEvaluationResult>,
        would_trigger: bool,
    }

    let would_trigger = result.as_ref().map(|r| r.triggered).unwrap_or(false);

    Ok(Json(TestResponse {
        rule: AlertRuleResponse::from(rule),
        evaluation: result,
        would_trigger,
    }))
}

// ==============================================================================
// Alert History Handlers
// ==============================================================================

async fn get_alert_history(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let alerts = alert_queries::get_alert_history_for_user(
        pool,
        user_id,
        params.limit,
        params.offset,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(alerts))
}

async fn get_alert_history_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let alert = sqlx::query_as::<_, AlertHistory>(
        "SELECT * FROM alert_history WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(alert))
}

async fn get_rule_history(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let alerts = alert_queries::get_alert_history_for_rule(pool, rule_id, params.limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(alerts))
}

// ==============================================================================
// Notification Handlers
// ==============================================================================

async fn get_notifications(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let notifications = alert_queries::get_user_notifications(
        pool,
        user_id,
        params.limit,
        params.offset,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(notifications))
}

async fn get_unread_count(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let (total, unread) = alert_queries::count_user_notifications(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(NotificationCountResponse { total, unread }))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    alert_queries::mark_notification_read(pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn mark_all_read(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    alert_queries::mark_all_notifications_read(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn delete_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    alert_queries::delete_notification(pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Notification Preferences Handlers
// ==============================================================================

async fn get_preferences(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let prefs = alert_queries::get_or_create_notification_preferences(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(prefs))
}

async fn update_preferences(
    State(state): State<AppState>,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    // Parse quiet hours
    let quiet_hours_start = req
        .quiet_hours_start
        .as_ref()
        .and_then(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M").ok());

    let quiet_hours_end = req
        .quiet_hours_end
        .as_ref()
        .and_then(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M").ok());

    let prefs = alert_queries::update_notification_preferences(
        pool,
        user_id,
        req.email_enabled,
        req.in_app_enabled,
        req.webhook_enabled,
        req.webhook_url.as_deref(),
        quiet_hours_start,
        quiet_hours_end,
        req.timezone.as_deref(),
        req.max_daily_emails,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(prefs))
}

// ==============================================================================
// Alert Evaluation Handler
// ==============================================================================

async fn evaluate_all_alerts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    // Evaluate all active alerts
    let results = alert_service::evaluate_all_alerts(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut triggered_count = 0;

    // Process triggered alerts
    for result in &results {
        if result.triggered {
            let rule = alert_queries::get_alert_rule(pool, result.rule_id)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Create alert history
            let alert_history = alert_service::process_triggered_alert(pool, &rule, result)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Send notifications
            notification_service::send_notification(pool, user_id, &alert_history)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            triggered_count += 1;
        }
    }

    let response = AlertEvaluationResponse {
        evaluated_rules: results.len(),
        triggered_alerts: triggered_count,
        results,
    };

    Ok(Json(response))
}

// ==============================================================================
// Test Email Handler
// ==============================================================================

async fn send_test_email(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    notification_service::send_test_email(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Test email sent successfully"
    })))
}

// ==============================================================================
// Helper Functions
// ==============================================================================

async fn get_default_user_id(pool: &PgPool) -> Result<Uuid, (StatusCode, String)> {
    // TODO: Replace with proper authentication
    // For now, use the default user created in migration
    let default_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Verify user exists
    alert_queries::get_user(pool, default_uuid)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(default_uuid)
}

use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post, put};
use tracing::{info, error};
use uuid::Uuid;

use crate::db::{llm_queries, user_preferences_queries};
use crate::errors::AppError;
use crate::models::{UpdateUserPreferences, UserPreferences, LlmUsageStats};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/:user_id/preferences", get(get_user_preferences))
        .route("/users/:user_id/preferences", put(update_user_preferences))
        .route("/users/:user_id/llm-consent", post(update_llm_consent))
        .route("/users/:user_id/usage", get(get_user_usage_stats))
}

/// GET /api/llm/users/:user_id/preferences
/// Get user preferences for AI features
#[axum::debug_handler]
pub async fn get_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPreferences>, AppError> {
    info!("GET /api/llm/users/{}/preferences", user_id);

    let preferences = user_preferences_queries::get_by_user_id(&state.pool, user_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch user preferences for {}: {}", user_id, e);
            AppError::Db(e)
        })?;

    match preferences {
        Some(prefs) => Ok(Json(prefs)),
        None => {
            // Return default preferences if none exist
            info!("No preferences found for user {}, returning defaults", user_id);
            Ok(Json(UserPreferences {
                id: Uuid::new_v4(),
                user_id,
                llm_enabled: false,
                consent_given_at: None,
                narrative_cache_hours: 24, // Default to 24 hours
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }
    }
}

/// PUT /api/llm/users/:user_id/preferences
/// Update user preferences for AI features
#[axum::debug_handler]
pub async fn update_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<UpdateUserPreferences>,
) -> Result<Json<UserPreferences>, AppError> {
    info!("PUT /api/llm/users/{}/preferences - llm_enabled: {}", user_id, data.llm_enabled);

    let preferences = user_preferences_queries::upsert(&state.pool, user_id, data)
        .await
        .map_err(|e| {
            error!("Failed to update user preferences for {}: {}", user_id, e);
            AppError::Db(e)
        })?;

    Ok(Json(preferences))
}

/// POST /api/llm/users/:user_id/llm-consent
/// Update LLM consent status
#[derive(Debug, serde::Deserialize)]
pub struct ConsentRequest {
    pub consent: bool,
}

#[axum::debug_handler]
pub async fn update_llm_consent(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<ConsentRequest>,
) -> Result<Json<UserPreferences>, AppError> {
    info!("POST /api/llm/users/{}/llm-consent - consent: {}", user_id, data.consent);

    let preferences = user_preferences_queries::update_llm_consent(&state.pool, user_id, data.consent)
        .await
        .map_err(|e| {
            error!("Failed to update LLM consent for {}: {}", user_id, e);
            AppError::Db(e)
        })?;

    Ok(Json(preferences))
}

/// GET /api/llm/users/:user_id/usage
/// Get LLM usage statistics for a user
#[axum::debug_handler]
pub async fn get_user_usage_stats(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<LlmUsageStats>, AppError> {
    info!("GET /api/llm/users/{}/usage", user_id);

    let stats = llm_queries::get_user_usage_stats(&state.pool, user_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch LLM usage stats for {}: {}", user_id, e);
            AppError::Db(e)
        })?;

    Ok(Json(stats))
}

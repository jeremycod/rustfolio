use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json,
    Router,
};
use serde_json::json;
use tracing::info;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{RiskPreferencesResponse, UpdateRiskPreferences};
use crate::services::user_preference_service;
use crate::state::AppState;

/// Create the preferences router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/me/preferences", get(get_preferences))
        .route("/users/me/preferences", put(update_preferences))
        .route("/users/me/preferences/reset", post(reset_preferences))
        .route("/users/me/risk-profile", get(get_risk_profile))
}

/// GET /api/users/me/preferences
pub async fn get_preferences(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    info!("GET /api/users/me/preferences for user {}", user_id);

    let preferences = user_preference_service::get_user_preferences(&state.pool, user_id).await?;

    let response = RiskPreferencesResponse::from(preferences);

    Ok((StatusCode::OK, Json(response)))
}

/// PUT /api/users/me/preferences
pub async fn update_preferences(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(update): Json<UpdateRiskPreferences>,
) -> Result<impl IntoResponse, AppError> {
    info!("PUT /api/users/me/preferences for user {}", user_id);

    let preferences =
        user_preference_service::update_user_preferences(&state.pool, user_id, update).await?;

    let response = RiskPreferencesResponse::from(preferences);

    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/users/me/preferences/reset
pub async fn reset_preferences(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    info!("POST /api/users/me/preferences/reset for user {}", user_id);

    let preferences =
        user_preference_service::reset_user_preferences(&state.pool, user_id).await?;

    let response = RiskPreferencesResponse::from(preferences);

    Ok((StatusCode::OK, Json(response)))
}

/// GET /api/users/me/risk-profile
pub async fn get_risk_profile(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    info!("GET /api/users/me/risk-profile for user {}", user_id);

    let preferences = user_preference_service::get_user_preferences(&state.pool, user_id).await?;

    let description = user_preference_service::get_risk_profile_description(&preferences);

    let profile = json!({
        "user_id": user_id,
        "risk_appetite": preferences.risk_appetite,
        "description": description,
        "forecast_horizon_days": user_preference_service::get_forecast_horizon_days(&preferences),
        "signal_confidence_threshold": preferences.signal_confidence_threshold(),
        "risk_threshold_multiplier": preferences.risk_threshold_multiplier(),
        "emphasize_downside_risk": preferences.emphasize_downside_risk(),
        "emphasize_growth_potential": preferences.emphasize_growth_potential(),
    });

    Ok((StatusCode::OK, Json(profile)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RiskAppetite, SignalSensitivity};

    #[test]
    fn test_update_preferences_serialization() {
        let update = UpdateRiskPreferences {
            llm_enabled: Some(true),
            narrative_cache_hours: Some(48),
            risk_appetite: Some(RiskAppetite::Aggressive),
            forecast_horizon_preference: Some(3),
            signal_sensitivity: Some(SignalSensitivity::High),
            sentiment_weight: Some(0.4),
            technical_weight: Some(0.3),
            fundamental_weight: Some(0.3),
            custom_settings: None,
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("Aggressive"));
        assert!(json.contains("High"));
    }

    #[test]
    fn test_partial_update() {
        let update = UpdateRiskPreferences {
            llm_enabled: None,
            narrative_cache_hours: None,
            risk_appetite: Some(RiskAppetite::Conservative),
            forecast_horizon_preference: None,
            signal_sensitivity: None,
            sentiment_weight: None,
            technical_weight: None,
            fundamental_weight: None,
            custom_settings: None,
        };

        assert!(update.validate().is_ok());
    }
}

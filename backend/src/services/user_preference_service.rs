use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::risk_preferences_queries;
use crate::errors::AppError;
use crate::models::{
    RiskAppetite, RiskPreferences, SignalSensitivity,
    UpdateRiskPreferences,
};

/// Get user preferences with defaults if not set
pub async fn get_user_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<RiskPreferences, AppError> {
    info!("Fetching preferences for user {}", user_id);

    match risk_preferences_queries::get_preferences_by_user_id(pool, user_id).await {
        Ok(Some(prefs)) => Ok(prefs),
        Ok(None) => {
            info!("No preferences found for user {}, creating defaults", user_id);
            let default_prefs = RiskPreferences::default_for_user(user_id);

            // Insert default preferences into database
            risk_preferences_queries::upsert_full_preferences(pool, user_id, &default_prefs)
                .await
                .map_err(AppError::Db)
        }
        Err(e) => {
            warn!("Database error fetching preferences: {}", e);
            Err(AppError::Db(e))
        }
    }
}

/// Update user preferences with validation
pub async fn update_user_preferences(
    pool: &PgPool,
    user_id: Uuid,
    mut update: UpdateRiskPreferences,
) -> Result<RiskPreferences, AppError> {
    info!("Updating preferences for user {}", user_id);

    // Validate the update
    update.validate().map_err(AppError::Validation)?;

    // Normalize weights if any are provided
    if update.sentiment_weight.is_some()
        || update.technical_weight.is_some()
        || update.fundamental_weight.is_some()
    {
        update.normalize_weights();
    }

    // Upsert preferences
    risk_preferences_queries::upsert_preferences(pool, user_id, &update)
        .await
        .map_err(AppError::Db)
}

/// Reset user preferences to defaults
pub async fn reset_user_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<RiskPreferences, AppError> {
    info!("Resetting preferences to defaults for user {}", user_id);

    let _default_prefs = RiskPreferences::default_for_user(user_id);
    let update = UpdateRiskPreferences {
        llm_enabled: Some(false),
        narrative_cache_hours: Some(24),
        risk_appetite: Some(RiskAppetite::Balanced),
        forecast_horizon_preference: Some(6),
        signal_sensitivity: Some(SignalSensitivity::Medium),
        sentiment_weight: Some(0.3),
        technical_weight: Some(0.4),
        fundamental_weight: Some(0.3),
        custom_settings: None,
    };

    risk_preferences_queries::upsert_preferences(pool, user_id, &update)
        .await
        .map_err(AppError::Db)
}

/// Apply risk appetite to risk thresholds
#[allow(dead_code)]
pub fn apply_risk_appetite_to_thresholds(
    base_volatility_threshold: f64,
    base_concentration_threshold: f64,
    risk_appetite: RiskAppetite,
) -> (f64, f64) {
    let multiplier = match risk_appetite {
        RiskAppetite::Conservative => 0.85, // Stricter thresholds
        RiskAppetite::Balanced => 1.0,      // Standard thresholds
        RiskAppetite::Aggressive => 1.2,    // Looser thresholds
    };

    let adjusted_volatility = base_volatility_threshold * multiplier;
    let adjusted_concentration = base_concentration_threshold * multiplier;

    (adjusted_volatility, adjusted_concentration)
}

/// Apply preferences to weight signals
#[allow(dead_code)]
pub fn apply_preferences_to_signals(
    sentiment_score: f64,
    technical_score: f64,
    fundamental_score: f64,
    preferences: &RiskPreferences,
) -> f64 {
    let sentiment_weight: f64 = preferences
        .sentiment_weight
        .to_string()
        .parse()
        .unwrap_or(0.3);
    let technical_weight: f64 = preferences
        .technical_weight
        .to_string()
        .parse()
        .unwrap_or(0.4);
    let fundamental_weight: f64 = preferences
        .fundamental_weight
        .to_string()
        .parse()
        .unwrap_or(0.3);

    // Calculate weighted average
    let weighted_score = (sentiment_score * sentiment_weight)
        + (technical_score * technical_weight)
        + (fundamental_score * fundamental_weight);

    // Normalize to 0-1 range
    weighted_score.max(0.0).min(1.0)
}

/// Filter signals based on user sensitivity setting
#[allow(dead_code)]
pub fn should_include_signal(signal_confidence: f64, preferences: &RiskPreferences) -> bool {
    let threshold = preferences.signal_confidence_threshold();
    signal_confidence >= threshold
}

/// Adjust forecast confidence intervals based on risk appetite
#[allow(dead_code)]
pub fn adjust_confidence_intervals(
    base_lower: f64,
    base_upper: f64,
    current_value: f64,
    risk_appetite: RiskAppetite,
) -> (f64, f64) {
    match risk_appetite {
        RiskAppetite::Conservative => {
            // Widen confidence intervals to show more uncertainty
            let lower_adjustment = (current_value - base_lower) * 1.2;
            let upper_adjustment = (base_upper - current_value) * 1.2;
            (
                current_value - lower_adjustment,
                current_value + upper_adjustment,
            )
        }
        RiskAppetite::Balanced => {
            // Standard intervals
            (base_lower, base_upper)
        }
        RiskAppetite::Aggressive => {
            // Narrow confidence intervals, show more certainty
            let lower_adjustment = (current_value - base_lower) * 0.8;
            let upper_adjustment = (base_upper - current_value) * 0.8;
            (
                current_value - lower_adjustment,
                current_value + upper_adjustment,
            )
        }
    }
}

/// Get risk profile description based on preferences
pub fn get_risk_profile_description(preferences: &RiskPreferences) -> String {
    match preferences.risk_appetite {
        RiskAppetite::Conservative => {
            format!(
                "Conservative investor: {}-month horizon, low signal sensitivity, emphasis on capital preservation and downside risk management.",
                preferences.forecast_horizon_preference
            )
        }
        RiskAppetite::Balanced => {
            format!(
                "Balanced investor: {}-month horizon, medium signal sensitivity, equal focus on risk and return potential.",
                preferences.forecast_horizon_preference
            )
        }
        RiskAppetite::Aggressive => {
            format!(
                "Aggressive investor: {}-month horizon, high signal sensitivity, emphasis on growth potential and short-term opportunities.",
                preferences.forecast_horizon_preference
            )
        }
    }
}

/// Calculate risk-adjusted forecast horizon in days
pub fn get_forecast_horizon_days(preferences: &RiskPreferences) -> i32 {
    // Convert months to days (approximate)
    let months = preferences.forecast_horizon_preference;
    months * 30
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_risk_appetite_to_thresholds() {
        let base_volatility = 0.2;
        let base_concentration = 0.3;

        // Conservative: stricter thresholds
        let (vol, conc) = apply_risk_appetite_to_thresholds(
            base_volatility,
            base_concentration,
            RiskAppetite::Conservative,
        );
        assert_eq!(vol, 0.17);
        assert_eq!(conc, 0.255);

        // Balanced: standard thresholds
        let (vol, conc) =
            apply_risk_appetite_to_thresholds(base_volatility, base_concentration, RiskAppetite::Balanced);
        assert_eq!(vol, 0.2);
        assert_eq!(conc, 0.3);

        // Aggressive: looser thresholds
        let (vol, conc) = apply_risk_appetite_to_thresholds(
            base_volatility,
            base_concentration,
            RiskAppetite::Aggressive,
        );
        assert_eq!(vol, 0.24);
        assert_eq!(conc, 0.36);
    }

    #[test]
    fn test_apply_preferences_to_signals() {
        let user_id = Uuid::new_v4();
        let prefs = RiskPreferences::default_for_user(user_id);

        let sentiment = 0.8;
        let technical = 0.6;
        let fundamental = 0.7;

        let weighted_score = apply_preferences_to_signals(sentiment, technical, fundamental, &prefs);

        // With default weights (0.3, 0.4, 0.3): 0.8*0.3 + 0.6*0.4 + 0.7*0.3 = 0.69
        assert!((weighted_score - 0.69).abs() < 0.01);
    }

    #[test]
    fn test_should_include_signal() {
        let user_id = Uuid::new_v4();
        let mut prefs = RiskPreferences::default_for_user(user_id);

        // Low sensitivity: high threshold (0.75)
        prefs.signal_sensitivity = SignalSensitivity::Low;
        assert!(!should_include_signal(0.70, &prefs));
        assert!(should_include_signal(0.80, &prefs));

        // Medium sensitivity: medium threshold (0.60)
        prefs.signal_sensitivity = SignalSensitivity::Medium;
        assert!(!should_include_signal(0.55, &prefs));
        assert!(should_include_signal(0.65, &prefs));

        // High sensitivity: low threshold (0.45)
        prefs.signal_sensitivity = SignalSensitivity::High;
        assert!(!should_include_signal(0.40, &prefs));
        assert!(should_include_signal(0.50, &prefs));
    }

    #[test]
    fn test_adjust_confidence_intervals() {
        let current = 100.0;
        let base_lower = 90.0;
        let base_upper = 110.0;

        // Conservative: wider intervals
        let (lower, upper) = adjust_confidence_intervals(
            base_lower,
            base_upper,
            current,
            RiskAppetite::Conservative,
        );
        assert!(lower < base_lower);
        assert!(upper > base_upper);

        // Balanced: unchanged
        let (lower, upper) =
            adjust_confidence_intervals(base_lower, base_upper, current, RiskAppetite::Balanced);
        assert_eq!(lower, base_lower);
        assert_eq!(upper, base_upper);

        // Aggressive: narrower intervals
        let (lower, upper) =
            adjust_confidence_intervals(base_lower, base_upper, current, RiskAppetite::Aggressive);
        assert!(lower > base_lower);
        assert!(upper < base_upper);
    }

    #[test]
    fn test_get_forecast_horizon_days() {
        let user_id = Uuid::new_v4();
        let mut prefs = RiskPreferences::default_for_user(user_id);

        prefs.forecast_horizon_preference = 6;
        assert_eq!(get_forecast_horizon_days(&prefs), 180);

        prefs.forecast_horizon_preference = 12;
        assert_eq!(get_forecast_horizon_days(&prefs), 360);

        prefs.forecast_horizon_preference = 3;
        assert_eq!(get_forecast_horizon_days(&prefs), 90);
    }
}

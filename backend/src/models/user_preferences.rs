use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use bigdecimal::FromPrimitive;
use sqlx::FromRow;
use uuid::Uuid;

/// Risk appetite levels for investment preferences
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum RiskAppetite {
    #[serde(rename = "Conservative")]
    Conservative,
    #[serde(rename = "Balanced")]
    Balanced,
    #[serde(rename = "Aggressive")]
    Aggressive,
}

impl Default for RiskAppetite {
    fn default() -> Self {
        RiskAppetite::Balanced
    }
}

impl std::fmt::Display for RiskAppetite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskAppetite::Conservative => write!(f, "Conservative"),
            RiskAppetite::Balanced => write!(f, "Balanced"),
            RiskAppetite::Aggressive => write!(f, "Aggressive"),
        }
    }
}

impl std::str::FromStr for RiskAppetite {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Conservative" => Ok(RiskAppetite::Conservative),
            "Balanced" => Ok(RiskAppetite::Balanced),
            "Aggressive" => Ok(RiskAppetite::Aggressive),
            _ => Err(format!("Invalid risk appetite: {}", s)),
        }
    }
}

/// Signal sensitivity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SignalSensitivity {
    #[serde(rename = "Low")]
    Low,
    #[serde(rename = "Medium")]
    Medium,
    #[serde(rename = "High")]
    High,
}

impl Default for SignalSensitivity {
    fn default() -> Self {
        SignalSensitivity::Medium
    }
}

impl std::fmt::Display for SignalSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalSensitivity::Low => write!(f, "Low"),
            SignalSensitivity::Medium => write!(f, "Medium"),
            SignalSensitivity::High => write!(f, "High"),
        }
    }
}

impl std::str::FromStr for SignalSensitivity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" => Ok(SignalSensitivity::Low),
            "Medium" => Ok(SignalSensitivity::Medium),
            "High" => Ok(SignalSensitivity::High),
            _ => Err(format!("Invalid signal sensitivity: {}", s)),
        }
    }
}

/// User risk preferences for personalized forecasts and signals
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskPreferences {
    pub id: Uuid,
    pub user_id: Uuid,

    // Core LLM preferences (from existing table)
    pub llm_enabled: bool,
    pub consent_given_at: Option<DateTime<Utc>>,
    pub narrative_cache_hours: i32,

    // Risk appetite preferences
    pub risk_appetite: RiskAppetite,
    pub forecast_horizon_preference: i32, // months: 1-24
    pub signal_sensitivity: SignalSensitivity,

    // Factor weights (0.0 to 1.0)
    pub sentiment_weight: BigDecimal,
    pub technical_weight: BigDecimal,
    pub fundamental_weight: BigDecimal,

    // Extensible custom settings (JSONB)
    pub custom_settings: Option<sqlx::types::JsonValue>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RiskPreferences {
    /// Get default preferences for a user
    pub fn default_for_user(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            llm_enabled: false,
            consent_given_at: None,
            narrative_cache_hours: 24,
            risk_appetite: RiskAppetite::Balanced,
            forecast_horizon_preference: 6,
            signal_sensitivity: SignalSensitivity::Medium,
            sentiment_weight: BigDecimal::from_f64(0.3).unwrap(),
            technical_weight: BigDecimal::from_f64(0.4).unwrap(),
            fundamental_weight: BigDecimal::from_f64(0.3).unwrap(),
            custom_settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Get the risk threshold multiplier based on risk appetite
    pub fn risk_threshold_multiplier(&self) -> f64 {
        match self.risk_appetite {
            RiskAppetite::Conservative => 0.85,
            RiskAppetite::Balanced => 1.0,
            RiskAppetite::Aggressive => 1.2,
        }
    }

    /// Get the default forecast horizon based on risk appetite
    #[allow(dead_code)]
    pub fn default_forecast_horizon_days(&self) -> i32 {
        match self.risk_appetite {
            RiskAppetite::Conservative => 365, // 12 months
            RiskAppetite::Balanced => 180,     // 6 months
            RiskAppetite::Aggressive => 90,    // 3 months
        }
    }

    /// Get the signal confidence threshold based on sensitivity
    pub fn signal_confidence_threshold(&self) -> f64 {
        match self.signal_sensitivity {
            SignalSensitivity::Low => 0.75,    // High confidence only
            SignalSensitivity::Medium => 0.60, // Medium confidence
            SignalSensitivity::High => 0.45,   // Lower threshold, more signals
        }
    }

    /// Check if focus should be on downside risk (conservative)
    pub fn emphasize_downside_risk(&self) -> bool {
        matches!(self.risk_appetite, RiskAppetite::Conservative)
    }

    /// Check if focus should be on growth potential (aggressive)
    pub fn emphasize_growth_potential(&self) -> bool {
        matches!(self.risk_appetite, RiskAppetite::Aggressive)
    }
}

/// Input for updating risk preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRiskPreferences {
    // LLM preferences (optional, preserve existing if not provided)
    pub llm_enabled: Option<bool>,
    pub narrative_cache_hours: Option<i32>,

    // Risk preferences
    pub risk_appetite: Option<RiskAppetite>,
    pub forecast_horizon_preference: Option<i32>,
    pub signal_sensitivity: Option<SignalSensitivity>,

    // Factor weights
    pub sentiment_weight: Option<f64>,
    pub technical_weight: Option<f64>,
    pub fundamental_weight: Option<f64>,

    // Custom settings
    pub custom_settings: Option<sqlx::types::JsonValue>,
}

impl UpdateRiskPreferences {
    /// Validate the preference update
    pub fn validate(&self) -> Result<(), String> {
        // Validate forecast horizon (1-24 months)
        if let Some(horizon) = self.forecast_horizon_preference {
            if !(1..=24).contains(&horizon) {
                return Err(format!(
                    "Forecast horizon must be between 1 and 24 months, got {}",
                    horizon
                ));
            }
        }

        // Validate narrative cache hours
        if let Some(hours) = self.narrative_cache_hours {
            if !(1..=168).contains(&hours) {
                return Err(format!(
                    "Narrative cache hours must be between 1 and 168, got {}",
                    hours
                ));
            }
        }

        // Validate individual weights (0.0 to 1.0)
        if let Some(weight) = self.sentiment_weight {
            if !(0.0..=1.0).contains(&weight) {
                return Err(format!("Sentiment weight must be between 0.0 and 1.0, got {}", weight));
            }
        }
        if let Some(weight) = self.technical_weight {
            if !(0.0..=1.0).contains(&weight) {
                return Err(format!("Technical weight must be between 0.0 and 1.0, got {}", weight));
            }
        }
        if let Some(weight) = self.fundamental_weight {
            if !(0.0..=1.0).contains(&weight) {
                return Err(format!(
                    "Fundamental weight must be between 0.0 and 1.0, got {}",
                    weight
                ));
            }
        }

        // Validate weights sum to reasonable range (0.8 to 1.2)
        if self.sentiment_weight.is_some()
            || self.technical_weight.is_some()
            || self.fundamental_weight.is_some()
        {
            let sentiment = self.sentiment_weight.unwrap_or(0.3);
            let technical = self.technical_weight.unwrap_or(0.4);
            let fundamental = self.fundamental_weight.unwrap_or(0.3);
            let sum = sentiment + technical + fundamental;

            if !(0.8..=1.2).contains(&sum) {
                return Err(format!(
                    "Sum of factor weights must be between 0.8 and 1.2, got {:.2}",
                    sum
                ));
            }
        }

        Ok(())
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize_weights(&mut self) {
        if self.sentiment_weight.is_none()
            && self.technical_weight.is_none()
            && self.fundamental_weight.is_none()
        {
            return;
        }

        let sentiment = self.sentiment_weight.unwrap_or(0.3);
        let technical = self.technical_weight.unwrap_or(0.4);
        let fundamental = self.fundamental_weight.unwrap_or(0.3);
        let sum = sentiment + technical + fundamental;

        if sum > 0.0 {
            self.sentiment_weight = Some(sentiment / sum);
            self.technical_weight = Some(technical / sum);
            self.fundamental_weight = Some(fundamental / sum);
        }
    }
}

/// Response for risk preference queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPreferencesResponse {
    pub user_id: Uuid,
    pub risk_appetite: RiskAppetite,
    pub forecast_horizon_preference: i32,
    pub signal_sensitivity: SignalSensitivity,
    pub sentiment_weight: f64,
    pub technical_weight: f64,
    pub fundamental_weight: f64,
    pub llm_enabled: bool,
    pub narrative_cache_hours: i32,
    pub custom_settings: Option<sqlx::types::JsonValue>,
    pub updated_at: DateTime<Utc>,
}

impl From<RiskPreferences> for RiskPreferencesResponse {
    fn from(prefs: RiskPreferences) -> Self {
        Self {
            user_id: prefs.user_id,
            risk_appetite: prefs.risk_appetite,
            forecast_horizon_preference: prefs.forecast_horizon_preference,
            signal_sensitivity: prefs.signal_sensitivity,
            sentiment_weight: prefs.sentiment_weight.to_string().parse().unwrap_or(0.3),
            technical_weight: prefs.technical_weight.to_string().parse().unwrap_or(0.4),
            fundamental_weight: prefs.fundamental_weight.to_string().parse().unwrap_or(0.3),
            llm_enabled: prefs.llm_enabled,
            narrative_cache_hours: prefs.narrative_cache_hours,
            custom_settings: prefs.custom_settings,
            updated_at: prefs.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let user_id = Uuid::new_v4();
        let prefs = RiskPreferences::default_for_user(user_id);

        assert_eq!(prefs.user_id, user_id);
        assert_eq!(prefs.risk_appetite, RiskAppetite::Balanced);
        assert_eq!(prefs.forecast_horizon_preference, 6);
        assert_eq!(prefs.signal_sensitivity, SignalSensitivity::Medium);
    }

    #[test]
    fn test_risk_threshold_multipliers() {
        let user_id = Uuid::new_v4();

        let mut prefs = RiskPreferences::default_for_user(user_id);
        prefs.risk_appetite = RiskAppetite::Conservative;
        assert_eq!(prefs.risk_threshold_multiplier(), 0.85);

        prefs.risk_appetite = RiskAppetite::Balanced;
        assert_eq!(prefs.risk_threshold_multiplier(), 1.0);

        prefs.risk_appetite = RiskAppetite::Aggressive;
        assert_eq!(prefs.risk_threshold_multiplier(), 1.2);
    }

    #[test]
    fn test_signal_confidence_thresholds() {
        let user_id = Uuid::new_v4();

        let mut prefs = RiskPreferences::default_for_user(user_id);
        prefs.signal_sensitivity = SignalSensitivity::Low;
        assert_eq!(prefs.signal_confidence_threshold(), 0.75);

        prefs.signal_sensitivity = SignalSensitivity::Medium;
        assert_eq!(prefs.signal_confidence_threshold(), 0.60);

        prefs.signal_sensitivity = SignalSensitivity::High;
        assert_eq!(prefs.signal_confidence_threshold(), 0.45);
    }

    #[test]
    fn test_validate_weights() {
        let mut update = UpdateRiskPreferences {
            llm_enabled: None,
            narrative_cache_hours: None,
            risk_appetite: None,
            forecast_horizon_preference: None,
            signal_sensitivity: None,
            sentiment_weight: Some(0.3),
            technical_weight: Some(0.4),
            fundamental_weight: Some(0.3),
            custom_settings: None,
        };

        assert!(update.validate().is_ok());

        // Test invalid weight
        update.sentiment_weight = Some(1.5);
        assert!(update.validate().is_err());

        // Test invalid sum
        update.sentiment_weight = Some(0.1);
        update.technical_weight = Some(0.1);
        update.fundamental_weight = Some(0.1);
        assert!(update.validate().is_err());
    }

    #[test]
    fn test_normalize_weights() {
        let mut update = UpdateRiskPreferences {
            llm_enabled: None,
            narrative_cache_hours: None,
            risk_appetite: None,
            forecast_horizon_preference: None,
            signal_sensitivity: None,
            sentiment_weight: Some(0.6),
            technical_weight: Some(0.8),
            fundamental_weight: Some(0.6),
            custom_settings: None,
        };

        update.normalize_weights();

        let sum = update.sentiment_weight.unwrap()
            + update.technical_weight.unwrap()
            + update.fundamental_weight.unwrap();

        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_forecast_horizon_by_risk_appetite() {
        let user_id = Uuid::new_v4();

        let mut prefs = RiskPreferences::default_for_user(user_id);
        prefs.risk_appetite = RiskAppetite::Conservative;
        assert_eq!(prefs.default_forecast_horizon_days(), 365);

        prefs.risk_appetite = RiskAppetite::Balanced;
        assert_eq!(prefs.default_forecast_horizon_days(), 180);

        prefs.risk_appetite = RiskAppetite::Aggressive;
        assert_eq!(prefs.default_forecast_horizon_days(), 90);
    }
}

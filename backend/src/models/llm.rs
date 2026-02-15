use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;

/// LLM usage tracking record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LlmUsage {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_cost: BigDecimal,
    pub created_at: DateTime<Utc>,
}

/// Input for creating LLM usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLlmUsage {
    pub user_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_cost: BigDecimal,
}

/// User preferences for AI features
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub llm_enabled: bool,
    pub consent_given_at: Option<DateTime<Utc>>,
    pub narrative_cache_hours: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for updating user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPreferences {
    pub llm_enabled: bool,
    pub narrative_cache_hours: Option<i32>,
}

/// LLM usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsageStats {
    pub total_requests: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_cost: BigDecimal,
    pub current_month_cost: BigDecimal,
}

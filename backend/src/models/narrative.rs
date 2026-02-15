use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Portfolio narrative response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioNarrative {
    pub summary: String,
    pub performance_explanation: String,
    pub risk_highlights: Vec<String>,
    pub top_contributors: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Request for generating portfolio narrative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateNarrativeRequest {
    pub time_period: Option<String>, // "30d", "90d", "1y"
}

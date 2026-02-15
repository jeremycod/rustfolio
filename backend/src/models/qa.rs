use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Confidence level for AI answers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "high"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::Low => write!(f, "low"),
        }
    }
}

/// User's question about their portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioQuestion {
    pub question: String,
    /// Optional context hint from UI (e.g., "I'm looking at the risk page")
    pub context_hint: Option<String>,
}

/// AI-generated answer with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnswer {
    pub answer: String,
    /// Data sources that informed the answer
    pub sources: Vec<String>,
    /// Confidence in the answer
    pub confidence: Confidence,
    /// Suggested follow-up questions
    pub follow_up_questions: Vec<String>,
    /// Timestamp when answer was generated
    pub generated_at: DateTime<Utc>,
}

/// Full Q&A conversation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAConversation {
    pub question: PortfolioQuestion,
    pub answer: PortfolioAnswer,
}

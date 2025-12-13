use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents a logical grouping of investments (e.g., "Long-term", "Retirement", "Speculative").
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Portfolio {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePortfolio {
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePortfolio {
    pub name: String
}

impl Portfolio {
    pub(crate) fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            created_at: chrono::Utc::now()
        }
    }
}
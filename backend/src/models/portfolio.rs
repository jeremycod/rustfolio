use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Represents a logical grouping of investments (e.g., "Long-term", "Retirement", "Speculative").
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Portfolio {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
    pub(crate) fn new(name: String, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            user_id,
            created_at: chrono::Utc::now(),
        }
    }
}
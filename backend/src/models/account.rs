use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents an account within a portfolio (e.g., "RRSP", "TFSA", "Investment Account")
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: uuid::Uuid,
    pub portfolio_id: uuid::Uuid,
    pub account_number: String,
    pub account_nickname: String,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccount {
    pub account_number: String,
    pub account_nickname: String,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
}

impl Account {
    pub fn new(
        portfolio_id: uuid::Uuid,
        account_number: String,
        account_nickname: String,
        client_id: Option<String>,
        client_name: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            portfolio_id,
            account_number,
            account_nickname,
            client_id,
            client_name,
            created_at: chrono::Utc::now(),
        }
    }
}

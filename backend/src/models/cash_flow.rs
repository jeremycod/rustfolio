use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FlowType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashFlow {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub flow_type: String, // Will be converted to/from FlowType
    pub amount: BigDecimal,
    pub flow_date: NaiveDate,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCashFlow {
    pub flow_type: FlowType,
    pub amount: BigDecimal,
    pub flow_date: NaiveDate,
    pub description: Option<String>,
}

impl CashFlow {
    pub fn new(
        account_id: uuid::Uuid,
        data: CreateCashFlow,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            account_id,
            flow_type: match data.flow_type {
                FlowType::Deposit => "DEPOSIT".to_string(),
                FlowType::Withdrawal => "WITHDRAWAL".to_string(),
            },
            amount: data.amount,
            flow_date: data.flow_date,
            description: data.description,
            created_at: chrono::Utc::now(),
        }
    }
}

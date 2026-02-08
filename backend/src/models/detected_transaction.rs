use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionType {
    Buy,
    Sell,
    Dividend,
    Split,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DetectedTransaction {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub transaction_type: String,
    pub ticker: String,
    pub quantity: Option<BigDecimal>,
    pub price: Option<BigDecimal>,
    pub amount: Option<BigDecimal>,
    pub transaction_date: NaiveDate,
    pub from_snapshot_date: Option<NaiveDate>,
    pub to_snapshot_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDetectedTransaction {
    pub transaction_type: TransactionType,
    pub ticker: String,
    pub quantity: Option<BigDecimal>,
    pub price: Option<BigDecimal>,
    pub amount: Option<BigDecimal>,
    pub from_snapshot_date: Option<NaiveDate>,
    pub to_snapshot_date: Option<NaiveDate>,
    pub description: Option<String>,
}

// Combined view of all account activity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountActivity {
    pub account_id: uuid::Uuid,
    pub activity_type: String, // 'TRANSACTION' or 'CASH_FLOW'
    pub type_detail: String,   // 'BUY', 'SELL', 'DEPOSIT', 'WITHDRAWAL', etc.
    pub ticker: Option<String>,
    pub quantity: Option<BigDecimal>,
    pub amount: Option<BigDecimal>,
    pub activity_date: NaiveDate,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountTruePerformance {
    pub account_id: uuid::Uuid,
    pub account_nickname: String,
    pub account_number: String,
    pub total_deposits: BigDecimal,
    pub total_withdrawals: BigDecimal,
    pub current_value: BigDecimal,
    pub book_value: BigDecimal,
    pub true_gain_loss: BigDecimal,
    pub true_gain_loss_pct: BigDecimal,
    pub as_of_date: Option<NaiveDate>,
}

impl DetectedTransaction {
    pub fn new(
        account_id: uuid::Uuid,
        transaction_date: NaiveDate,
        data: CreateDetectedTransaction,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            account_id,
            transaction_type: match data.transaction_type {
                TransactionType::Buy => "BUY".to_string(),
                TransactionType::Sell => "SELL".to_string(),
                TransactionType::Dividend => "DIVIDEND".to_string(),
                TransactionType::Split => "SPLIT".to_string(),
                TransactionType::Other => "OTHER".to_string(),
            },
            ticker: data.ticker,
            quantity: data.quantity,
            price: data.price,
            amount: data.amount,
            transaction_date,
            from_snapshot_date: data.from_snapshot_date,
            to_snapshot_date: data.to_snapshot_date,
            description: data.description,
            created_at: chrono::Utc::now(),
        }
    }
}

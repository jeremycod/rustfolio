use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Represents a historical snapshot of a holding at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HoldingSnapshot {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub ticker: String,
    pub holding_name: Option<String>,
    pub asset_category: Option<String>,
    pub industry: Option<String>,
    pub quantity: BigDecimal,
    pub price: BigDecimal,
    pub average_cost: BigDecimal,
    pub book_value: BigDecimal,
    pub market_value: BigDecimal,
    pub fund: Option<String>,
    pub accrued_interest: Option<BigDecimal>,
    pub gain_loss: Option<BigDecimal>,
    pub gain_loss_pct: Option<BigDecimal>,
    pub percentage_of_assets: Option<BigDecimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHoldingSnapshot {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub asset_category: Option<String>,
    pub industry: Option<String>,
    pub quantity: BigDecimal,
    pub price: BigDecimal,
    pub average_cost: BigDecimal,
    pub book_value: BigDecimal,
    pub market_value: BigDecimal,
    pub fund: Option<String>,
    pub accrued_interest: Option<BigDecimal>,
    pub gain_loss: Option<BigDecimal>,
    pub gain_loss_pct: Option<BigDecimal>,
    pub percentage_of_assets: Option<BigDecimal>,
}

// View for latest holdings per account
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LatestAccountHolding {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub account_nickname: String,
    pub account_number: String,
    pub ticker: String,
    pub holding_name: Option<String>,
    pub asset_category: Option<String>,
    pub industry: Option<String>,
    pub quantity: BigDecimal,
    pub price: BigDecimal,
    pub market_value: BigDecimal,
    pub gain_loss: Option<BigDecimal>,
    pub gain_loss_pct: Option<BigDecimal>,
    pub snapshot_date: chrono::NaiveDate,
}

// View for account value history over time
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountValueHistory {
    pub account_id: uuid::Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub total_value: BigDecimal,
    pub total_cost: BigDecimal,
    pub total_gain_loss: Option<BigDecimal>,
    pub total_gain_loss_pct: Option<BigDecimal>,
}

impl HoldingSnapshot {
    pub fn new(
        account_id: uuid::Uuid,
        snapshot_date: chrono::NaiveDate,
        data: CreateHoldingSnapshot,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            account_id,
            snapshot_date,
            ticker: data.ticker,
            holding_name: data.holding_name,
            asset_category: data.asset_category,
            industry: data.industry,
            quantity: data.quantity,
            price: data.price,
            average_cost: data.average_cost,
            book_value: data.book_value,
            market_value: data.market_value,
            fund: data.fund,
            accrued_interest: data.accrued_interest,
            gain_loss: data.gain_loss,
            gain_loss_pct: data.gain_loss_pct,
            percentage_of_assets: data.percentage_of_assets,
            created_at: chrono::Utc::now(),
        }
    }
}

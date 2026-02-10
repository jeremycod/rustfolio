use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{AccountValueHistory, CreateHoldingSnapshot, HoldingSnapshot, LatestAccountHolding};

pub async fn create(
    pool: &PgPool,
    account_id: Uuid,
    snapshot_date: NaiveDate,
    input: CreateHoldingSnapshot,
) -> Result<HoldingSnapshot, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, HoldingSnapshot>(
        "INSERT INTO holdings_snapshots
         (id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
          quantity, price, average_cost, book_value, market_value, fund,
          accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
         RETURNING id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
                   quantity, price, average_cost, book_value, market_value, fund,
                   accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets, created_at"
    )
    .bind(id)
    .bind(account_id)
    .bind(snapshot_date)
    .bind(&input.ticker)
    .bind(&input.holding_name)
    .bind(&input.asset_category)
    .bind(&input.industry)
    .bind(&input.quantity)
    .bind(&input.price)
    .bind(&input.average_cost)
    .bind(&input.book_value)
    .bind(&input.market_value)
    .bind(&input.fund)
    .bind(&input.accrued_interest)
    .bind(&input.gain_loss)
    .bind(&input.gain_loss_pct)
    .bind(&input.percentage_of_assets)
    .fetch_one(pool)
    .await
}

pub async fn upsert(
    pool: &PgPool,
    account_id: Uuid,
    snapshot_date: NaiveDate,
    input: CreateHoldingSnapshot,
) -> Result<HoldingSnapshot, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, HoldingSnapshot>(
        "INSERT INTO holdings_snapshots
         (id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
          quantity, price, average_cost, book_value, market_value, fund,
          accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
         ON CONFLICT (account_id, snapshot_date, ticker)
         DO UPDATE SET
             holding_name = EXCLUDED.holding_name,
             asset_category = EXCLUDED.asset_category,
             industry = EXCLUDED.industry,
             quantity = EXCLUDED.quantity,
             price = EXCLUDED.price,
             average_cost = EXCLUDED.average_cost,
             book_value = EXCLUDED.book_value,
             market_value = EXCLUDED.market_value,
             fund = EXCLUDED.fund,
             accrued_interest = EXCLUDED.accrued_interest,
             gain_loss = EXCLUDED.gain_loss,
             gain_loss_pct = EXCLUDED.gain_loss_pct,
             percentage_of_assets = EXCLUDED.percentage_of_assets
         RETURNING id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
                   quantity, price, average_cost, book_value, market_value, fund,
                   accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets, created_at"
    )
    .bind(id)
    .bind(account_id)
    .bind(snapshot_date)
    .bind(&input.ticker)
    .bind(&input.holding_name)
    .bind(&input.asset_category)
    .bind(&input.industry)
    .bind(&input.quantity)
    .bind(&input.price)
    .bind(&input.average_cost)
    .bind(&input.book_value)
    .bind(&input.market_value)
    .bind(&input.fund)
    .bind(&input.accrued_interest)
    .bind(&input.gain_loss)
    .bind(&input.gain_loss_pct)
    .bind(&input.percentage_of_assets)
    .fetch_one(pool)
    .await
}

pub async fn fetch_by_account(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<HoldingSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, HoldingSnapshot>(
        "SELECT id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
                quantity, price, average_cost, book_value, market_value, fund,
                accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets, created_at
         FROM holdings_snapshots
         WHERE account_id = $1
         ORDER BY snapshot_date DESC, ticker"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_by_account_and_date(
    pool: &PgPool,
    account_id: Uuid,
    snapshot_date: NaiveDate,
) -> Result<Vec<HoldingSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, HoldingSnapshot>(
        "SELECT id, account_id, snapshot_date, ticker, holding_name, asset_category, industry,
                quantity, price, average_cost, book_value, market_value, fund,
                accrued_interest, gain_loss, gain_loss_pct, percentage_of_assets, created_at
         FROM holdings_snapshots
         WHERE account_id = $1 AND snapshot_date = $2
         ORDER BY ticker"
    )
    .bind(account_id)
    .bind(snapshot_date)
    .fetch_all(pool)
    .await
}

pub async fn fetch_latest_holdings(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<LatestAccountHolding>, sqlx::Error> {
    sqlx::query_as::<_, LatestAccountHolding>(
        "SELECT * FROM latest_account_holdings WHERE account_id = $1 ORDER BY ticker"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_portfolio_latest_holdings(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<LatestAccountHolding>, sqlx::Error> {
    sqlx::query_as::<_, LatestAccountHolding>(
        "SELECT lah.*
         FROM latest_account_holdings lah
         JOIN accounts a ON lah.account_id = a.id
         WHERE a.portfolio_id = $1
         ORDER BY lah.ticker"
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_account_value_history(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<AccountValueHistory>, sqlx::Error> {
    sqlx::query_as::<_, AccountValueHistory>(
        "SELECT * FROM account_value_history WHERE account_id = $1 ORDER BY snapshot_date"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_portfolio_value_history(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<AccountValueHistory>, sqlx::Error> {
    sqlx::query_as::<_, AccountValueHistory>(
        "SELECT
            avh.account_id,
            avh.snapshot_date,
            avh.total_value,
            avh.total_cost,
            avh.total_gain_loss,
            avh.total_gain_loss_pct
         FROM account_value_history avh
         JOIN accounts a ON avh.account_id = a.id
         WHERE a.portfolio_id = $1
         ORDER BY avh.snapshot_date, avh.account_id"
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_by_account(pool: &PgPool, account_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM holdings_snapshots WHERE account_id = $1", account_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

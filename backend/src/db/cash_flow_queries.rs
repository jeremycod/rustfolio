use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{CashFlow, CreateCashFlow};

pub async fn create(
    pool: &PgPool,
    account_id: Uuid,
    data: CreateCashFlow,
) -> Result<CashFlow, sqlx::Error> {
    let cash_flow = CashFlow::new(account_id, data);

    sqlx::query_as::<_, CashFlow>(
        "INSERT INTO cash_flows (id, account_id, flow_type, amount, flow_date, description)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, account_id, flow_type, amount, flow_date, description, created_at"
    )
    .bind(cash_flow.id)
    .bind(cash_flow.account_id)
    .bind(&cash_flow.flow_type)
    .bind(&cash_flow.amount)
    .bind(cash_flow.flow_date)
    .bind(&cash_flow.description)
    .fetch_one(pool)
    .await
}

pub async fn fetch_by_account(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<CashFlow>, sqlx::Error> {
    sqlx::query_as::<_, CashFlow>(
        "SELECT id, account_id, flow_type, amount, flow_date, description, created_at
         FROM cash_flows
         WHERE account_id = $1
         ORDER BY flow_date DESC"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn fetch_by_date_range(
    pool: &PgPool,
    account_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<CashFlow>, sqlx::Error> {
    sqlx::query_as::<_, CashFlow>(
        "SELECT id, account_id, flow_type, amount, flow_date, description, created_at
         FROM cash_flows
         WHERE account_id = $1 AND flow_date BETWEEN $2 AND $3
         ORDER BY flow_date DESC"
    )
    .bind(account_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

pub async fn update_account_totals(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE accounts
         SET total_deposits = COALESCE((
             SELECT SUM(amount) FROM cash_flows
             WHERE account_id = $1 AND flow_type = 'DEPOSIT'
         ), 0),
         total_withdrawals = COALESCE((
             SELECT SUM(amount) FROM cash_flows
             WHERE account_id = $1 AND flow_type = 'WITHDRAWAL'
         ), 0)
         WHERE id = $1",
        account_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

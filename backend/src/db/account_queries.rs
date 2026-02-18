use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Account, CreateAccount};

pub async fn fetch_all(pool: &PgPool, portfolio_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query_as::<_, Account>(
        "SELECT id, portfolio_id, account_number, account_nickname, client_id, client_name, created_at
         FROM accounts
         WHERE portfolio_id = $1
         ORDER BY created_at DESC"
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_one(pool: &PgPool, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
    sqlx::query_as::<_, Account>(
        "SELECT id, portfolio_id, account_number, account_nickname, client_id, client_name, created_at
         FROM accounts
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_account_number(
    pool: &PgPool,
    portfolio_id: Uuid,
    account_number: &str,
) -> Result<Option<Account>, sqlx::Error> {
    sqlx::query_as::<_, Account>(
        "SELECT id, portfolio_id, account_number, account_nickname, client_id, client_name, created_at
         FROM accounts
         WHERE portfolio_id = $1 AND account_number = $2"
    )
    .bind(portfolio_id)
    .bind(account_number)
    .fetch_optional(pool)
    .await
}

#[allow(dead_code)]
pub async fn create(
    pool: &PgPool,
    portfolio_id: Uuid,
    input: CreateAccount,
) -> Result<Account, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, Account>(
        "INSERT INTO accounts (id, portfolio_id, account_number, account_nickname, client_id, client_name)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, portfolio_id, account_number, account_nickname, client_id, client_name, created_at"
    )
    .bind(id)
    .bind(portfolio_id)
    .bind(input.account_number)
    .bind(input.account_nickname)
    .bind(input.client_id)
    .bind(input.client_name)
    .fetch_one(pool)
    .await
}

pub async fn upsert(
    pool: &PgPool,
    portfolio_id: Uuid,
    input: CreateAccount,
) -> Result<Account, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, Account>(
        "INSERT INTO accounts (id, portfolio_id, account_number, account_nickname, client_id, client_name)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (portfolio_id, account_number)
         DO UPDATE SET
             account_nickname = EXCLUDED.account_nickname,
             client_id = EXCLUDED.client_id,
             client_name = EXCLUDED.client_name
         RETURNING id, portfolio_id, account_number, account_nickname, client_id, client_name, created_at"
    )
    .bind(id)
    .bind(portfolio_id)
    .bind(input.account_number)
    .bind(input.account_nickname)
    .bind(input.client_id)
    .bind(input.client_name)
    .fetch_one(pool)
    .await
}

#[allow(dead_code)]
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM accounts WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

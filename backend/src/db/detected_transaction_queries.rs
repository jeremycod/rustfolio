use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{DetectedTransaction, CreateDetectedTransaction, AccountActivity, AccountTruePerformance};

pub async fn create(
    pool: &PgPool,
    account_id: Uuid,
    transaction_date: NaiveDate,
    data: CreateDetectedTransaction,
) -> Result<DetectedTransaction, sqlx::Error> {
    let transaction = DetectedTransaction::new(account_id, transaction_date, data);

    sqlx::query_as::<_, DetectedTransaction>(
        "INSERT INTO detected_transactions
         (id, account_id, transaction_type, ticker, quantity, price, amount, transaction_date,
          from_snapshot_date, to_snapshot_date, description)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, account_id, transaction_type, ticker, quantity, price, amount,
                   transaction_date, from_snapshot_date, to_snapshot_date, description, created_at"
    )
    .bind(transaction.id)
    .bind(transaction.account_id)
    .bind(&transaction.transaction_type)
    .bind(&transaction.ticker)
    .bind(&transaction.quantity)
    .bind(&transaction.price)
    .bind(&transaction.amount)
    .bind(transaction.transaction_date)
    .bind(transaction.from_snapshot_date)
    .bind(transaction.to_snapshot_date)
    .bind(&transaction.description)
    .fetch_one(pool)
    .await
}

pub async fn fetch_by_account(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<DetectedTransaction>, sqlx::Error> {
    sqlx::query_as::<_, DetectedTransaction>(
        "SELECT id, account_id, transaction_type, ticker, quantity, price, amount,
                transaction_date, from_snapshot_date, to_snapshot_date, description, created_at
         FROM detected_transactions
         WHERE account_id = $1
         ORDER BY transaction_date DESC"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_account_activity(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<AccountActivity>, sqlx::Error> {
    sqlx::query_as::<_, AccountActivity>(
        "SELECT account_id, activity_type, type_detail, ticker, quantity, amount, activity_date, description
         FROM account_activity
         WHERE account_id = $1
         ORDER BY activity_date DESC"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_true_performance(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<AccountTruePerformance>, sqlx::Error> {
    sqlx::query_as::<_, AccountTruePerformance>(
        "SELECT account_id, account_nickname, account_number, total_deposits, total_withdrawals,
                current_value, book_value, true_gain_loss, true_gain_loss_pct, as_of_date
         FROM account_true_performance
         WHERE account_id = $1"
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
}

pub async fn fetch_all_true_performance(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<AccountTruePerformance>, sqlx::Error> {
    sqlx::query_as::<_, AccountTruePerformance>(
        "SELECT atp.account_id, atp.account_nickname, atp.account_number, atp.total_deposits,
                atp.total_withdrawals, atp.current_value, atp.book_value, atp.true_gain_loss,
                atp.true_gain_loss_pct, atp.as_of_date
         FROM account_true_performance atp
         JOIN accounts a ON atp.account_id = a.id
         WHERE a.portfolio_id = $1"
    )
    .bind(portfolio_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_transactions_for_snapshot(
    pool: &PgPool,
    account_id: Uuid,
    snapshot_date: NaiveDate,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM detected_transactions
         WHERE account_id = $1 AND to_snapshot_date = $2",
        account_id,
        snapshot_date
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

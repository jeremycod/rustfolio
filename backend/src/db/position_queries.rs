use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{CreatePosition, Portfolio, Position, UpdatePortfolio, UpdatePosition};

pub async fn create(
    pool: &PgPool,
    portfolio_id: Uuid,
    input: CreatePosition,
) -> Result<Position, sqlx::Error> {
    sqlx::query_as::<_, Position>(
        "INSERT INTO positions (id, portfolio_id, ticker, shares, avg_buy_price)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, portfolio_id, ticker, shares, avg_buy_price, created_at"
    )
        .bind(Uuid::new_v4())
        .bind(portfolio_id)
        .bind(input.ticker)
        .bind(input.shares)
        .bind(input.avg_buy_price)
        .fetch_one(pool)
        .await
}

pub async fn fetch_one(pool: &PgPool, id: Uuid) -> Result<Option<Position>, sqlx::Error> {
    sqlx::query_as::<_, Position>( "SELECT id, name, created_at
                      FROM positions
                      WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: UpdatePosition
) -> Result<Option<Position>, sqlx::Error> {
    sqlx::query_as::<_, Position>(
        "UPDATE positions
         SET shares = $2, avg_buy_price = $3
         WHERE id = $1
         RETURNING id, portfolio_id, ticker, shares, avg_buy_price, created_at"
    )
        .bind(id)
        .bind(input.shares)
        .bind(input.avg_buy_price)
        .fetch_optional(pool)
        .await
}

pub async fn delete(pool: &PgPool, id: Uuid)
                    -> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM positions WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn fetch_all(
    pool: &PgPool,
    portfolio_id: Uuid
) -> Result<Vec<Position>, sqlx::Error> {
    sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE portfolio_id = $1 ORDER BY created_at DESC"
    )
        .bind(portfolio_id)
        .fetch_all(pool)
        .await
}
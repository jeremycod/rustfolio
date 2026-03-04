use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Portfolio, UpdatePortfolio};

pub async fn fetch_all(pool: &PgPool, user_id: Uuid) -> Result<Vec<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>(
        "SELECT id, name, user_id, created_at
         FROM portfolios
         WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_one(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>(
        "SELECT id, name, user_id, created_at
         FROM portfolios
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &PgPool, input: Portfolio) -> Result<Portfolio, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>(
        "INSERT INTO portfolios (id, name, user_id, created_at)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, user_id, created_at",
    )
    .bind(input.id)
    .bind(input.name)
    .bind(input.user_id)
    .bind(input.created_at)
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, id: Uuid, input: UpdatePortfolio) -> Result<Option<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>(
        "UPDATE portfolios SET name = $1 WHERE id = $2
         RETURNING id, name, user_id, created_at",
    )
    .bind(input.name)
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Fetch a portfolio by ID without an ownership check — for internal services only.
pub async fn fetch_one_unchecked(pool: &PgPool, id: Uuid) -> Result<Option<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>(
        "SELECT id, name, user_id, created_at FROM portfolios WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM portfolios WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn exists(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM portfolios WHERE id = $1)")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(result.0)
}

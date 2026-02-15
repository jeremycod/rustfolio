use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Portfolio, UpdatePortfolio};

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Portfolio>, sqlx::Error> {
    sqlx::query_as!(Portfolio, "SELECT id, name, created_at
                      FROM portfolios
                      ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn fetch_one(pool: &PgPool, id: Uuid) -> Result<Option<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>( "SELECT id, name, created_at
                      FROM portfolios
                      WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &PgPool, name: String)
-> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!("INSERT INTO portfolios (id, name)
                  VALUES ($1, $2)", id, name)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn insert(pool: &PgPool, input: Portfolio)
                              -> Result<Portfolio, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>("INSERT INTO portfolios (id, name, created_at)
                  VALUES ($1, $2, $3)
                    RETURNING id, name, created_at")
        .bind(input.id)
        .bind(input.name)
        .bind(input.created_at)
        .fetch_one(pool)
        .await
}

pub async fn update(pool: &PgPool, id: Uuid, input: UpdatePortfolio)
-> Result<Option<Portfolio>, sqlx::Error> {
    sqlx::query_as::<_, Portfolio>("UPDATE portfolios
        SET name = $1 WHERE id = $2")
        .bind(input.name)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn delete(pool: &PgPool, id: Uuid)
-> Result<u64, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM portfolios WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn exists(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("SELECT EXISTS(SELECT 1 FROM portfolios WHERE id = $1)", id)
        .fetch_one(pool)
        .await?;
    Ok(result.exists.unwrap_or(false))
}

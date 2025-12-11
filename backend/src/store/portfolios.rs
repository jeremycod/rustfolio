use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::Portfolio;

pub async fn fetch_portfolios(pool: &PgPool) -> Result<Vec<Portfolio>, sqlx::Error> {
    let portfolios = sqlx::query_as!(Portfolio, "SELECT id, name, created_at
                      FROM portfolios
                      ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(portfolios)
}

pub async fn create_portfolio(pool: &PgPool, name: String)
-> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!("INSERT INTO portfolios (id, name)
                  VALUES ($1, $2)", id, name)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn rename_portfolio(pool: &PgPool, id: Uuid, name: String)
-> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE portfolios SET name = $1 WHERE id = $2", name, id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_portfolio(pool: &PgPool, id: Uuid)
-> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM portfolios WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{error, warn};
use crate::models::PricePoint;
use crate::external::price_provider::ExternalPricePoint;

pub async fn insert_many(
    pool: &PgPool,
    items: &[PricePoint]
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for p in items {
        sqlx::query!(
            "INSERT INTO price_points (id, ticker, date, close_price)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (ticker, date) DO UPDATE SET close_price = EXCLUDED.close_price",
            Uuid::new_v4(),
            p.ticker,
            p.date,
            p.close_price
        )
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())

}

pub async fn fetch_all(
    pool: &PgPool,
    ticker: &str
) -> Result<Vec<PricePoint>, sqlx::Error> {
    sqlx::query_as!(
        PricePoint,
        "SELECT id, ticker, date, close_price, created_at
         FROM price_points
         WHERE ticker = $1
         ORDER BY date ASC",
        ticker
    )
        .fetch_all(pool)
        .await
}

pub async fn fetch_latest(
    pool: &PgPool,
    ticker: &str
) -> Result<Option<PricePoint>, sqlx::Error> {
    sqlx::query_as!(
        PricePoint,
        "SELECT id, ticker, date, close_price, created_at
         FROM price_points
         WHERE ticker = $1
         ORDER BY date DESC
         LIMIT 1",
        ticker
    )
        .fetch_optional(pool)
        .await
}

pub async fn upsert_external_points(
    pool: &PgPool,
    ticker: &str,
    points: &[ExternalPricePoint],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction for ticker {}: {}", ticker, e);
        e
    })?;

    for (i, p) in points.iter().enumerate() {
        if let Err(e) = sqlx::query!(
            r#"
            INSERT INTO price_points (id, ticker, date, close_price)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (ticker, date)
            DO UPDATE SET close_price = EXCLUDED.close_price
            "#,
            Uuid::new_v4(),
            ticker,
            p.date,
            p.close
        )
            .execute(&mut *tx)
            .await {
            error!("Failed to upsert price point {} for ticker {} (date: {}, price: {}): {}", 
                   i, ticker, p.date, p.close, e);
            return Err(e);
        }
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction for ticker {}: {}", ticker, e);
        e
    })?;
    Ok(())
}
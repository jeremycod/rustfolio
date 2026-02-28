use sqlx::PgPool;
use uuid::Uuid;
use tracing::error;
use crate::models::PricePoint;
use crate::external::price_provider::ExternalPricePoint;

#[allow(dead_code)]
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

pub async fn fetch_latest_batch(
    pool: &PgPool,
    tickers: &[String],
) -> Result<std::collections::HashMap<String, PricePoint>, sqlx::Error> {
    if tickers.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // Use DISTINCT ON to get the latest price for each ticker efficiently
    let prices = sqlx::query_as::<_, PricePoint>(
        r#"
        SELECT DISTINCT ON (ticker) id, ticker, date, close_price, created_at
        FROM price_points
        WHERE ticker = ANY($1)
        ORDER BY ticker, date DESC
        "#,
    )
    .bind(tickers)
    .fetch_all(pool)
    .await?;

    // Convert to HashMap for O(1) lookups
    let map = prices.into_iter()
        .map(|p| (p.ticker.clone(), p))
        .collect();

    Ok(map)
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

/// Fetch the most recent N days of price history for a ticker.
///
/// Returns price points ordered by date in ascending order (oldest first).
pub async fn fetch_window(
    pool: &PgPool,
    ticker: &str,
    days: i64,
) -> Result<Vec<PricePoint>, sqlx::Error> {
    sqlx::query_as!(
        PricePoint,
        r#"
        SELECT id, ticker, date, close_price, created_at
        FROM price_points
        WHERE ticker = $1
        ORDER BY date DESC
        LIMIT $2
        "#,
        ticker,
        days
    )
    .fetch_all(pool)
    .await
    .map(|mut points| {
        // Reverse to get ascending order (oldest first)
        points.reverse();
        points
    })
}

/// Fetch the most recent N days of price history for multiple tickers in one query.
///
/// Returns a map of ticker -> price points ordered by date ascending (oldest first).
pub async fn fetch_window_batch(
    pool: &PgPool,
    tickers: &[String],
    days: i64,
) -> Result<std::collections::HashMap<String, Vec<PricePoint>>, sqlx::Error> {
    use std::collections::HashMap;

    if tickers.is_empty() {
        return Ok(HashMap::new());
    }

    // Use query_as instead of query_as! to avoid compile-time verification issues with arrays
    let points = sqlx::query_as::<_, PricePoint>(
        r#"
        SELECT id, ticker, date, close_price, created_at
        FROM price_points
        WHERE ticker = ANY($1)
        ORDER BY ticker, date DESC
        "#,
    )
    .bind(tickers)
    .fetch_all(pool)
    .await?;

    // Group by ticker and take only the most recent N days for each
    let mut result: HashMap<String, Vec<PricePoint>> = HashMap::new();

    for point in points {
        result
            .entry(point.ticker.clone())
            .or_insert_with(Vec::new)
            .push(point);
    }

    // Limit each ticker to N days and reverse to ascending order
    for (_, points) in result.iter_mut() {
        points.truncate(days as usize);
        points.reverse();
    }

    Ok(result)
}
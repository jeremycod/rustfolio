use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PortfolioValueRow {
    pub date: NaiveDate,
    pub value: f64,
}

pub async fn fetch_portfolio_value_series(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<PortfolioValueRow>, sqlx::Error> {
    // Key idea: join positions -> price_points by ticker, sum(shares * close_price) grouped by date
    let rows = sqlx::query!(
        r#"
        SELECT
          pp.date as "date!",
          SUM(p.shares * pp.close_price)::double precision as "value!"
        FROM positions p
        JOIN price_points pp
          ON pp.ticker = p.ticker
        WHERE p.portfolio_id = $1
        GROUP BY pp.date
        ORDER BY pp.date ASC
        "#,
        portfolio_id
    )
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| PortfolioValueRow {
            date: r.date,
            value: r.value,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct AllocationRow {
    pub ticker: String,
    pub value: f64,
}

pub async fn fetch_allocations_at_latest_date(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<AllocationRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH latest AS (
          SELECT MAX(pp.date) AS date
          FROM positions p
          JOIN price_points pp ON pp.ticker = p.ticker
          WHERE p.portfolio_id = $1
        )
        SELECT
          p.ticker as "ticker!",
          (p.shares * pp.close_price)::double precision as "value!"
        FROM positions p
        JOIN latest l ON TRUE
        JOIN price_points pp
          ON pp.ticker = p.ticker
         AND pp.date = l.date
        WHERE p.portfolio_id = $1
        ORDER BY p.ticker ASC
        "#,
        portfolio_id
    )
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| AllocationRow { ticker: r.ticker, value: r.value })
        .collect())
}
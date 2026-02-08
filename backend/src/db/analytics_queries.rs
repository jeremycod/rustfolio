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
    // Aggregate holdings_snapshots across all accounts in a portfolio
    let rows = sqlx::query!(
        r#"
        SELECT
          h.snapshot_date as "date!",
          SUM(h.market_value)::double precision as "value!"
        FROM holdings_snapshots h
        JOIN accounts a ON h.account_id = a.id
        WHERE a.portfolio_id = $1
        GROUP BY h.snapshot_date
        ORDER BY h.snapshot_date ASC
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
        WITH latest_snapshot AS (
          SELECT MAX(h.snapshot_date) AS snapshot_date
          FROM holdings_snapshots h
          JOIN accounts a ON h.account_id = a.id
          WHERE a.portfolio_id = $1
        )
        SELECT
          h.ticker as "ticker!",
          SUM(h.market_value)::double precision as "value!"
        FROM holdings_snapshots h
        JOIN accounts a ON h.account_id = a.id
        JOIN latest_snapshot l ON h.snapshot_date = l.snapshot_date
        WHERE a.portfolio_id = $1
          AND h.ticker != ''
        GROUP BY h.ticker
        ORDER BY h.ticker ASC
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
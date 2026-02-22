use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::risk_snapshot::{CreateRiskSnapshot, RiskSnapshot};

/// Upsert a risk snapshot (idempotent daily snapshots)
pub async fn upsert_snapshot(
    pool: &PgPool,
    snapshot: CreateRiskSnapshot,
) -> Result<RiskSnapshot, sqlx::Error> {
    sqlx::query_as::<_, RiskSnapshot>(
        r#"
        INSERT INTO risk_snapshots (
            portfolio_id, ticker, snapshot_date, snapshot_type,
            volatility, max_drawdown, beta, sharpe, value_at_risk,
            var_95, var_99, expected_shortfall_95, expected_shortfall_99,
            risk_score, risk_level, total_value, market_value
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (portfolio_id, ticker, snapshot_date, snapshot_type)
        DO UPDATE SET
            volatility = EXCLUDED.volatility,
            max_drawdown = EXCLUDED.max_drawdown,
            beta = EXCLUDED.beta,
            sharpe = EXCLUDED.sharpe,
            value_at_risk = EXCLUDED.value_at_risk,
            var_95 = EXCLUDED.var_95,
            var_99 = EXCLUDED.var_99,
            expected_shortfall_95 = EXCLUDED.expected_shortfall_95,
            expected_shortfall_99 = EXCLUDED.expected_shortfall_99,
            risk_score = EXCLUDED.risk_score,
            risk_level = EXCLUDED.risk_level,
            total_value = EXCLUDED.total_value,
            market_value = EXCLUDED.market_value,
            created_at = NOW()
        RETURNING *
        "#,
    )
    .bind(snapshot.portfolio_id)
    .bind(snapshot.ticker)
    .bind(snapshot.snapshot_date)
    .bind(snapshot.snapshot_type)
    .bind(snapshot.volatility)
    .bind(snapshot.max_drawdown)
    .bind(snapshot.beta)
    .bind(snapshot.sharpe)
    .bind(snapshot.value_at_risk)
    .bind(snapshot.var_95)
    .bind(snapshot.var_99)
    .bind(snapshot.expected_shortfall_95)
    .bind(snapshot.expected_shortfall_99)
    .bind(snapshot.risk_score)
    .bind(snapshot.risk_level)
    .bind(snapshot.total_value)
    .bind(snapshot.market_value)
    .fetch_one(pool)
    .await
}

/// Fetch risk history for a portfolio or position within a date range
pub async fn fetch_history(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: Option<&str>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<RiskSnapshot>, sqlx::Error> {
    if let Some(ticker) = ticker {
        // Position-specific history
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker = $2
              AND snapshot_date BETWEEN $3 AND $4
              AND snapshot_type = 'position'
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(portfolio_id)
        .bind(ticker)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await
    } else {
        // Portfolio-level history
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker IS NULL
              AND snapshot_date BETWEEN $2 AND $3
              AND snapshot_type = 'portfolio'
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(portfolio_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await
    }
}

/// Get the latest snapshot for a portfolio or position
#[allow(dead_code)]
pub async fn fetch_latest(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: Option<&str>,
) -> Result<Option<RiskSnapshot>, sqlx::Error> {
    if let Some(ticker) = ticker {
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker = $2
              AND snapshot_type = 'position'
            ORDER BY snapshot_date DESC
            LIMIT 1
            "#,
        )
        .bind(portfolio_id)
        .bind(ticker)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker IS NULL
              AND snapshot_type = 'portfolio'
            ORDER BY snapshot_date DESC
            LIMIT 1
            "#,
        )
        .bind(portfolio_id)
        .fetch_optional(pool)
        .await
    }
}

/// Fetch all position snapshots for a portfolio on a specific date
#[allow(dead_code)]
pub async fn fetch_portfolio_positions_by_date(
    pool: &PgPool,
    portfolio_id: Uuid,
    date: NaiveDate,
) -> Result<Vec<RiskSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, RiskSnapshot>(
        r#"
        SELECT *
        FROM risk_snapshots
        WHERE portfolio_id = $1
          AND snapshot_date = $2
          AND snapshot_type = 'position'
          AND ticker IS NOT NULL
        ORDER BY risk_score DESC
        "#,
    )
    .bind(portfolio_id)
    .bind(date)
    .fetch_all(pool)
    .await
}

/// Fetch snapshots for multiple dates (for trend analysis)
#[allow(dead_code)]
pub async fn fetch_snapshots_by_dates(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: Option<&str>,
    dates: &[NaiveDate],
) -> Result<Vec<RiskSnapshot>, sqlx::Error> {
    if let Some(ticker) = ticker {
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker = $2
              AND snapshot_date = ANY($3)
              AND snapshot_type = 'position'
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(portfolio_id)
        .bind(ticker)
        .bind(dates)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, RiskSnapshot>(
            r#"
            SELECT *
            FROM risk_snapshots
            WHERE portfolio_id = $1
              AND ticker IS NULL
              AND snapshot_date = ANY($2)
              AND snapshot_type = 'portfolio'
            ORDER BY snapshot_date ASC
            "#,
        )
        .bind(portfolio_id)
        .bind(dates)
        .fetch_all(pool)
        .await
    }
}

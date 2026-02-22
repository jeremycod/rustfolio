use crate::models::{CreateMarketRegime, MarketRegime};
use chrono::NaiveDate;
use sqlx::PgPool;

/// Get the most recent market regime (current regime)
pub async fn get_current_regime(pool: &PgPool) -> Result<MarketRegime, sqlx::Error> {
    sqlx::query_as::<_, MarketRegime>(
        r#"
        SELECT
            id,
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier,
            created_at,
            updated_at
        FROM market_regimes
        ORDER BY date DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
}

/// Get market regime for a specific date
#[allow(dead_code)]
pub async fn get_regime_by_date(
    pool: &PgPool,
    date: NaiveDate,
) -> Result<MarketRegime, sqlx::Error> {
    sqlx::query_as::<_, MarketRegime>(
        r#"
        SELECT
            id,
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier,
            created_at,
            updated_at
        FROM market_regimes
        WHERE date = $1
        "#,
    )
    .bind(date)
    .fetch_one(pool)
    .await
}

/// Get historical market regimes for a date range
pub async fn get_regime_history(
    pool: &PgPool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<MarketRegime>, sqlx::Error> {
    sqlx::query_as::<_, MarketRegime>(
        r#"
        SELECT
            id,
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier,
            created_at,
            updated_at
        FROM market_regimes
        WHERE date BETWEEN $1 AND $2
        ORDER BY date DESC
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

/// Get most recent N regimes
#[allow(dead_code)]
pub async fn get_recent_regimes(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<MarketRegime>, sqlx::Error> {
    sqlx::query_as::<_, MarketRegime>(
        r#"
        SELECT
            id,
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier,
            created_at,
            updated_at
        FROM market_regimes
        ORDER BY date DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Insert or update a market regime for a specific date (upsert)
pub async fn upsert_regime(
    pool: &PgPool,
    regime: CreateMarketRegime,
) -> Result<MarketRegime, sqlx::Error> {
    sqlx::query_as::<_, MarketRegime>(
        r#"
        INSERT INTO market_regimes (
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (date)
        DO UPDATE SET
            regime_type = EXCLUDED.regime_type,
            volatility_level = EXCLUDED.volatility_level,
            market_return = EXCLUDED.market_return,
            confidence = EXCLUDED.confidence,
            benchmark_ticker = EXCLUDED.benchmark_ticker,
            lookback_days = EXCLUDED.lookback_days,
            threshold_multiplier = EXCLUDED.threshold_multiplier,
            updated_at = NOW()
        RETURNING
            id,
            date,
            regime_type,
            volatility_level,
            market_return,
            confidence,
            benchmark_ticker,
            lookback_days,
            threshold_multiplier,
            created_at,
            updated_at
        "#,
    )
    .bind(regime.date)
    .bind(regime.regime_type)
    .bind(regime.volatility_level)
    .bind(regime.market_return)
    .bind(regime.confidence)
    .bind(regime.benchmark_ticker)
    .bind(regime.lookback_days)
    .bind(regime.threshold_multiplier)
    .fetch_one(pool)
    .await
}

/// Get count of regimes by type within a date range
#[allow(dead_code)]
pub async fn get_regime_type_counts(
    pool: &PgPool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT regime_type, COUNT(*) as count
        FROM market_regimes
        WHERE date BETWEEN $1 AND $2
        GROUP BY regime_type
        ORDER BY count DESC
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

/// Delete regimes older than a certain date (for cleanup)
#[allow(dead_code)]
pub async fn delete_old_regimes(
    pool: &PgPool,
    before_date: NaiveDate,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM market_regimes
        WHERE date < $1
        "#,
    )
    .bind(before_date)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queries_compile() {
        // This test just ensures the queries compile
        // Integration tests would require a test database
    }
}

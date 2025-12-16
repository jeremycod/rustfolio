use crate::db;
use crate::errors::AppError;
use crate::models::{AllocationPoint, AnalyticsMeta, AnalyticsResponse, ChartPoint};
use crate::services::indicators;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_analytics(pool: &PgPool, portfolio_id: Uuid) -> Result<AnalyticsResponse, AppError> {
    let rows = db::analytics_queries::fetch_portfolio_value_series(pool, portfolio_id).await?;
    let values: Vec<f64> = rows.iter().map(|r| r.value).collect();

    let sma20 = indicators::sma(&values, 20);
    let ema20 = indicators::ema(&values, 20);
    let (m, b) = indicators::regression_trend(&values);

    // Build chart series with a single iterator pipeline
    let series: Vec<ChartPoint> = rows
        .iter()
        .zip(sma20.into_iter())
        .zip(ema20.into_iter())
        .enumerate()
        .map(|(i, ((r, sma), ema))| ChartPoint {
            date: r.date,
            value: r.value,
            sma20: sma,
            ema20: ema,
            trend: Some(m * i as f64 + b),
        })
        .collect();

    let allocations = compute_allocations(pool, portfolio_id).await?;

    let meta = AnalyticsMeta {
        points: series.len(),
        start: series.first().map(|p| p.date),
        end: series.last().map(|p| p.date),
    };

    Ok(AnalyticsResponse {
        series,
        allocations,
        meta,
    })
}

/// Keep allocation calculation separated (pure-ish mapping + DB call).
async fn compute_allocations(pool: &PgPool, portfolio_id: Uuid) -> Result<Vec<AllocationPoint>, AppError> {
    let rows = db::analytics_queries::fetch_allocations_at_latest_date(pool, portfolio_id).await?;
    let total: f64 = rows.iter().map(|r| r.value).sum();

    Ok(rows
        .into_iter()
        .filter(|r| r.value.is_finite() && r.value > 0.0)
        .map(|r| AllocationPoint {
            ticker: r.ticker,
            value: r.value,
            weight: if total > 0.0 { r.value / total } else { 0.0 },
        })
        .collect())
}
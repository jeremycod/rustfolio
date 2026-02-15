use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::{holding_snapshot_queries, risk_snapshot_queries};
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::risk_snapshot::{Aggregation, CreateRiskSnapshot, RiskAlert, RiskSnapshot};
use crate::models::RiskLevel;
use crate::services::failure_cache::FailureCache;
use crate::services::risk_service;

/// Create daily risk snapshots for a portfolio and all its positions
pub async fn create_daily_snapshots(
    pool: &PgPool,
    portfolio_id: Uuid,
    date: NaiveDate,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<Vec<RiskSnapshot>, AppError> {
    info!(
        "Creating risk snapshots for portfolio {} on {}",
        portfolio_id, date
    );

    // Fetch all holdings for the portfolio
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(pool, portfolio_id)
        .await?;

    if holdings.is_empty() {
        return Err(AppError::NotFound);
    }

    let mut snapshots = Vec::new();

    // Aggregate holdings by ticker (same ticker across multiple accounts)
    let mut ticker_aggregates: HashMap<String, (f64, f64)> = HashMap::new(); // (quantity, market_value)

    for holding in &holdings {
        let market_value = holding.market_value.to_f64().unwrap_or(0.0);
        let quantity = holding.quantity.to_f64().unwrap_or(0.0);

        ticker_aggregates
            .entry(holding.ticker.clone())
            .and_modify(|(q, mv)| {
                *q += quantity;
                *mv += market_value;
            })
            .or_insert((quantity, market_value));
    }

    // Create position-level snapshots for each unique ticker
    for (ticker, (_quantity, market_value)) in &ticker_aggregates {
        match create_position_snapshot(
            pool,
            portfolio_id,
            ticker,
            date,
            *market_value,
            price_provider,
            failure_cache,
            risk_free_rate,
        )
        .await
        {
            Ok(snapshot) => snapshots.push(snapshot),
            Err(e) => {
                warn!(
                    "Failed to create snapshot for {}: {}. Skipping.",
                    ticker, e
                );
            }
        }
    }

    // Create portfolio-level snapshot
    match create_portfolio_snapshot(pool, portfolio_id, date, &ticker_aggregates, price_provider, failure_cache, risk_free_rate)
        .await
    {
        Ok(snapshot) => snapshots.push(snapshot),
        Err(e) => {
            warn!("Failed to create portfolio snapshot: {}", e);
        }
    }

    info!(
        "Created {} snapshots for portfolio {}",
        snapshots.len(),
        portfolio_id
    );

    Ok(snapshots)
}

/// Create a snapshot for a single position
async fn create_position_snapshot(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: &str,
    date: NaiveDate,
    market_value: f64,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<RiskSnapshot, AppError> {
    // Compute risk metrics for the position
    let risk_assessment =
        risk_service::compute_risk_metrics(pool, ticker, 90, "SPY", price_provider, failure_cache, risk_free_rate)
            .await?;

    let position_risk = &risk_assessment.metrics;

    let snapshot = CreateRiskSnapshot {
        portfolio_id,
        ticker: Some(ticker.to_string()),
        snapshot_date: date,
        snapshot_type: "position".to_string(),
        volatility: BigDecimal::from_f64(position_risk.volatility).unwrap_or_else(|| BigDecimal::from(0)),
        max_drawdown: BigDecimal::from_f64(position_risk.max_drawdown).unwrap_or_else(|| BigDecimal::from(0)),
        beta: position_risk.beta.and_then(|b| BigDecimal::from_f64(b)),
        sharpe: position_risk.sharpe.and_then(|s| BigDecimal::from_f64(s)),
        value_at_risk: position_risk.value_at_risk.and_then(|v| BigDecimal::from_f64(v)),
        risk_score: BigDecimal::from_f64(risk_assessment.risk_score).unwrap_or_else(|| BigDecimal::from(0)),
        risk_level: risk_assessment.risk_level.to_string(),
        total_value: None,
        market_value: Some(BigDecimal::from_f64(market_value).unwrap_or_else(|| BigDecimal::from(0))),
    };

    risk_snapshot_queries::upsert_snapshot(pool, snapshot)
        .await
        .map_err(|e| AppError::Db(e))
}

/// Create a snapshot for the entire portfolio
async fn create_portfolio_snapshot(
    pool: &PgPool,
    portfolio_id: Uuid,
    date: NaiveDate,
    ticker_aggregates: &HashMap<String, (f64, f64)>,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
    risk_free_rate: f64,
) -> Result<RiskSnapshot, AppError> {
    // Calculate total portfolio value
    let total_value: f64 = ticker_aggregates.values().map(|(_, mv)| mv).sum();

    if total_value == 0.0 {
        return Err(AppError::External(
            "Portfolio has no holdings with market value".to_string()
        ));
    }

    // Compute weighted portfolio risk metrics
    let mut weighted_volatility = 0.0;
    let mut weighted_max_drawdown = 0.0;
    let mut weighted_beta = 0.0;
    let mut weighted_sharpe = 0.0;
    let mut beta_count = 0;
    let mut sharpe_count = 0;

    for (ticker, (_quantity, market_value)) in ticker_aggregates {
        let weight = market_value / total_value;
        if weight < 0.001 {
            continue; // Skip negligible positions
        }

        match risk_service::compute_risk_metrics(
            pool,
            ticker,
            90,
            "SPY",
            price_provider,
            failure_cache,
            risk_free_rate,
        ).await {
            Ok(assessment) => {
                weighted_volatility += assessment.metrics.volatility * weight;
                weighted_max_drawdown += assessment.metrics.max_drawdown * weight;

                if let Some(beta) = assessment.metrics.beta {
                    weighted_beta += beta * weight;
                    beta_count += 1;
                }

                if let Some(sharpe) = assessment.metrics.sharpe {
                    weighted_sharpe += sharpe * weight;
                    sharpe_count += 1;
                }
            },
            Err(e) => {
                warn!("Could not compute risk for {} in portfolio: {}", ticker, e);
            }
        }
    }

    // Calculate portfolio-level risk score
    let portfolio_risk_score = risk_service::score_risk(&crate::models::PositionRisk {
        volatility: weighted_volatility,
        max_drawdown: weighted_max_drawdown,
        beta: if beta_count > 0 { Some(weighted_beta) } else { None },
        sharpe: if sharpe_count > 0 { Some(weighted_sharpe) } else { None },
        sortino: None,
        annualized_return: None,
        value_at_risk: None,
    });

    let risk_level = RiskLevel::from_score(portfolio_risk_score);

    let snapshot = CreateRiskSnapshot {
        portfolio_id,
        ticker: None,
        snapshot_date: date,
        snapshot_type: "portfolio".to_string(),
        volatility: BigDecimal::from_f64(weighted_volatility).unwrap_or_else(|| BigDecimal::from(0)),
        max_drawdown: BigDecimal::from_f64(weighted_max_drawdown).unwrap_or_else(|| BigDecimal::from(0)),
        beta: if beta_count > 0 { BigDecimal::from_f64(weighted_beta) } else { None },
        sharpe: if sharpe_count > 0 { BigDecimal::from_f64(weighted_sharpe) } else { None },
        value_at_risk: None,
        risk_score: BigDecimal::from_f64(portfolio_risk_score).unwrap_or_else(|| BigDecimal::from(0)),
        risk_level: risk_level.to_string(),
        total_value: Some(BigDecimal::from_f64(total_value).unwrap_or_else(|| BigDecimal::from(0))),
        market_value: None,
    };

    risk_snapshot_queries::upsert_snapshot(pool, snapshot)
        .await
        .map_err(|e| AppError::Db(e))
}

/// Detect significant risk increases (>threshold% in risk score)
pub async fn detect_risk_increases(
    pool: &PgPool,
    portfolio_id: Uuid,
    lookback_days: i64,
    threshold: f64,
) -> Result<Vec<RiskAlert>, AppError> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(lookback_days);

    // Fetch portfolio-level history
    let history = risk_snapshot_queries::fetch_history(pool, portfolio_id, None, start_date, end_date)
        .await
        .map_err(|e| AppError::Db(e))?;

    let mut alerts = Vec::new();

    // Check for consecutive days with risk score increases
    for i in 1..history.len() {
        let prev = &history[i - 1];
        let curr = &history[i];

        let prev_score = prev.risk_score.to_f64().unwrap_or(0.0);
        let curr_score = curr.risk_score.to_f64().unwrap_or(0.0);

        if prev_score > 0.0 {
            let change_percent = ((curr_score - prev_score) / prev_score) * 100.0;

            if change_percent >= threshold {
                alerts.push(RiskAlert {
                    portfolio_id: portfolio_id.to_string(),
                    ticker: None,
                    alert_type: "risk_increase".to_string(),
                    previous_value: prev_score,
                    current_value: curr_score,
                    change_percent,
                    date: curr.snapshot_date,
                    metric_name: "risk_score".to_string(),
                });
            }
        }
    }

    Ok(alerts)
}

/// Get risk trend for visualization with optional aggregation
pub async fn get_risk_trend(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: Option<&str>,
    days: i64,
    aggregation: Aggregation,
) -> Result<Vec<RiskSnapshot>, AppError> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(days);

    let history = risk_snapshot_queries::fetch_history(pool, portfolio_id, ticker, start_date, end_date)
        .await
        .map_err(|e| AppError::Db(e))?;

    // Apply aggregation if needed
    match aggregation {
        Aggregation::Daily => Ok(history),
        Aggregation::Weekly => Ok(aggregate_by_week(history)),
        Aggregation::Monthly => Ok(aggregate_by_month(history)),
    }
}

/// Aggregate snapshots by week (take the last snapshot of each week)
fn aggregate_by_week(snapshots: Vec<RiskSnapshot>) -> Vec<RiskSnapshot> {
    if snapshots.is_empty() {
        return snapshots;
    }

    let mut result = Vec::new();
    let mut current_year_week: Option<(i32, u32)> = None;

    for snapshot in snapshots {
        let year = snapshot.snapshot_date.year();
        let week = snapshot.snapshot_date.iso_week().week();
        let year_week = (year, week);

        if current_year_week != Some(year_week) {
            result.push(snapshot.clone());
            current_year_week = Some(year_week);
        } else {
            // Replace with the latest snapshot in the week
            result.pop();
            result.push(snapshot.clone());
        }
    }

    result
}

/// Aggregate snapshots by month (take the last snapshot of each month)
fn aggregate_by_month(snapshots: Vec<RiskSnapshot>) -> Vec<RiskSnapshot> {
    if snapshots.is_empty() {
        return snapshots;
    }

    let mut result = Vec::new();
    let mut current_year_month: Option<(i32, u32)> = None;

    for snapshot in snapshots {
        let year = snapshot.snapshot_date.year();
        let month = snapshot.snapshot_date.month();
        let year_month = (year, month);

        if current_year_month != Some(year_month) {
            result.push(snapshot.clone());
            current_year_month = Some(year_month);
        } else {
            // Replace with the latest snapshot in the month
            result.pop();
            result.push(snapshot.clone());
        }
    }

    result
}

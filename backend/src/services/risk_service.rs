use crate::db::price_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::risk::{PositionRisk, RiskAssessment, RiskLevel};
use crate::models::PricePoint;
use crate::services::price_service;
use crate::services::failure_cache::FailureCache;
use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use tracing::info;

/// Compute comprehensive risk metrics for a ticker over a rolling window.
///
/// This function automatically ensures price data is fresh by fetching from
/// the external price provider if the local data is stale or missing.
///
/// # Arguments
/// * `pool` – Postgres connection pool
/// * `ticker` – the symbol to analyze
/// * `days` – number of trading days in the window (e.g., 90)
/// * `benchmark` – symbol of the benchmark index for beta calculation (e.g., "SPY")
/// * `price_provider` – external price data provider for fetching fresh data
/// * `failure_cache` – cache to avoid retrying known-bad tickers
///
/// # Returns
/// A `RiskAssessment` containing all risk metrics and an overall risk score.
pub async fn compute_risk_metrics(
    pool: &PgPool,
    ticker: &str,
    days: i64,
    benchmark: &str,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
) -> Result<RiskAssessment, AppError> {
    // Ensure we have recent price data for both ticker and benchmark
    info!("Ensuring fresh price data for ticker: {}", ticker);
    let ticker_fetch_failed = price_service::refresh_from_api(pool, price_provider, ticker, failure_cache).await.is_err();

    info!("Ensuring fresh price data for benchmark: {}", benchmark);
    let benchmark_fetch_failed = price_service::refresh_from_api(pool, price_provider, benchmark, failure_cache).await.is_err();

    // Fetch price history for the ticker and benchmark
    let series = price_queries::fetch_window(pool, ticker, days).await?;
    let bench = price_queries::fetch_window(pool, benchmark, days).await?;

    if series.is_empty() {
        let error_msg = if ticker_fetch_failed {
            format!(
                "No price data found for ticker {}. Failed to fetch from external API. The ticker may not be available in your price provider's free tier, may not exist, or you may have hit rate limits.",
                ticker
            )
        } else {
            format!(
                "No price data found for ticker {}. The ticker may not exist or has no trading history.",
                ticker
            )
        };
        return Err(AppError::External(error_msg));
    }

    if bench.len() < 2 {
        let error_msg = if benchmark_fetch_failed {
            format!(
                "Insufficient benchmark data for {}. Failed to fetch from external API.",
                benchmark
            )
        } else {
            format!(
                "Insufficient benchmark data for {}. Need at least 2 data points.",
                benchmark
            )
        };
        return Err(AppError::External(error_msg));
    }

    // Compute individual risk metrics
    let (volatility, max_drawdown) = compute_vol_drawdown(&series);
    let beta = compute_beta(&series, &bench);
    let sharpe = compute_sharpe(&series);
    let var = compute_var(&series);

    let metrics = PositionRisk {
        volatility,
        max_drawdown,
        beta,
        sharpe,
        value_at_risk: var,
    };

    // Calculate overall risk score
    let risk_score = score_risk(&metrics);
    let risk_level = RiskLevel::from_score(risk_score);

    Ok(RiskAssessment {
        ticker: ticker.to_string(),
        metrics,
        risk_score,
        risk_level,
    })
}

/// Compute volatility (annualized) and max drawdown for a price series.
///
/// Returns `(volatility_pct, max_drawdown_pct)`.
fn compute_vol_drawdown(series: &[PricePoint]) -> (f64, f64) {
    if series.len() < 2 {
        return (0.0, 0.0);
    }

    // Convert prices to f64 and compute daily returns
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() < 2 {
        return (0.0, 0.0);
    }

    let mut returns = Vec::new();
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let cur = prices[i];
        if prev > 0.0 {
            returns.push((cur - prev) / prev);
        }
    }

    if returns.is_empty() {
        return (0.0, 0.0);
    }

    // Calculate volatility (annualized)
    let mean = returns.iter().copied().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (returns.len() as f64 - 1.0);
    let daily_volatility = variance.sqrt();
    let volatility = daily_volatility * (252.0_f64).sqrt() * 100.0; // Annualized as percentage

    // Calculate max drawdown
    let mut peak = prices[0];
    let mut max_dd = 0.0;
    for &price in &prices {
        if price > peak {
            peak = price;
        }
        let dd = (price - peak) / peak;
        if dd < max_dd {
            max_dd = dd;
        }
    }

    (volatility, max_dd * 100.0) // Convert to percentage
}

/// Compute beta relative to a benchmark return series.
///
/// Beta measures the systematic risk of a security relative to the market (benchmark).
/// A beta > 1 indicates higher volatility than the market, < 1 indicates lower volatility.
fn compute_beta(series: &[PricePoint], bench: &[PricePoint]) -> Option<f64> {
    if series.len() != bench.len() || series.len() < 2 {
        return None;
    }

    // Convert to f64 prices
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();
    let bench_prices: Vec<f64> = bench
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() != bench_prices.len() || prices.len() < 2 {
        return None;
    }

    // Calculate daily returns
    let returns: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();
    let bench_returns: Vec<f64> = bench_prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    if returns.is_empty() {
        return None;
    }

    // Calculate means
    let mean_r = returns.iter().sum::<f64>() / returns.len() as f64;
    let mean_b = bench_returns.iter().sum::<f64>() / bench_returns.len() as f64;

    // Calculate covariance and benchmark variance
    let mut cov = 0.0;
    let mut var_b = 0.0;
    for (r, b) in returns.iter().zip(bench_returns.iter()) {
        cov += (r - mean_r) * (b - mean_b);
        var_b += (b - mean_b).powi(2);
    }

    if var_b.abs() < f64::EPSILON {
        return None;
    }

    Some(cov / var_b)
}

/// Compute the annualized Sharpe ratio using a fixed risk-free rate.
///
/// The Sharpe ratio measures risk-adjusted return. Higher values indicate better
/// risk-adjusted performance.
fn compute_sharpe(series: &[PricePoint]) -> Option<f64> {
    if series.len() < 2 {
        return None;
    }

    // Convert to f64 prices
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() < 2 {
        return None;
    }

    // Calculate daily returns
    let mut returns = Vec::new();
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let cur = prices[i];
        if prev > 0.0 {
            returns.push((cur - prev) / prev);
        }
    }

    if returns.is_empty() {
        return None;
    }

    // Calculate mean return and volatility
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (returns.len() as f64 - 1.0);
    let volatility = variance.sqrt() * (252.0_f64).sqrt(); // Annualized

    // Assume 2% annual risk-free rate
    let risk_free = 0.02 / 252.0;

    // Annualized Sharpe ratio
    Some(((mean - risk_free) * 252.0) / volatility)
}

/// Compute a 5% Value at Risk (VaR) using historical simulation.
///
/// VaR represents the maximum expected loss at a given confidence level.
/// A 5% VaR means there's a 5% chance of losing more than this amount in a single day.
fn compute_var(series: &[PricePoint]) -> Option<f64> {
    if series.len() < 2 {
        return None;
    }

    // Convert to f64 prices
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() < 2 {
        return None;
    }

    // Calculate daily returns
    let mut returns = Vec::new();
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let cur = prices[i];
        if prev > 0.0 {
            returns.push((cur - prev) / prev);
        }
    }

    if returns.is_empty() {
        return None;
    }

    // Sort returns to find the 5th percentile
    let mut sorted_returns = returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let idx = (sorted_returns.len() as f64 * 0.05).floor() as usize;
    Some(sorted_returns[idx] * 100.0) // Convert to percentage
}

/// Score a PositionRisk into a 0–100 risk rating.
///
/// Higher scores indicate higher risk. The score is calculated as a weighted
/// combination of various risk metrics.
///
/// # Weighting
/// - 40% volatility (normalized to 50% max)
/// - 30% drawdown severity (normalized to -50% max)
/// - 20% beta magnitude (normalized to 2.0 max)
/// - 10% VaR (normalized to -10% max)
pub fn score_risk(risk: &PositionRisk) -> f64 {
    // Volatility component (40 points max)
    // Assume 50% annualized volatility as extreme
    let vol_score = (risk.volatility / 50.0).min(1.0) * 40.0;

    // Drawdown component (30 points max)
    // Assume -50% max drawdown as extreme
    let dd_score = (-risk.max_drawdown / 50.0).min(1.0) * 30.0;

    // Beta component (20 points max)
    // Assume beta of 2.0 as extreme
    let beta_score = risk
        .beta
        .map(|b| (b.abs().min(2.0) / 2.0) * 20.0)
        .unwrap_or(0.0);

    // VaR component (10 points max)
    // Assume -10% VaR as extreme
    let var_score = risk
        .value_at_risk
        .map(|v| (v.abs().min(10.0) / 10.0) * 10.0)
        .unwrap_or(0.0);

    (vol_score + dd_score + beta_score + var_score).min(100.0)
}

/// Calculate the correlation coefficient between two price series.
///
/// Correlation measures how two securities move together:
/// - +1.0: Perfect positive correlation (move together)
/// -  0.0: No correlation (independent movement)
/// - -1.0: Perfect negative correlation (move opposite)
pub fn compute_correlation(series1: &[PricePoint], series2: &[PricePoint]) -> Option<f64> {
    if series1.len() != series2.len() || series1.len() < 2 {
        return None;
    }

    // Convert to f64 prices
    let prices1: Vec<f64> = series1
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();
    let prices2: Vec<f64> = series2
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices1.len() != prices2.len() || prices1.len() < 2 {
        return None;
    }

    // Calculate daily returns
    let returns1: Vec<f64> = prices1
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();
    let returns2: Vec<f64> = prices2
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    if returns1.is_empty() {
        return None;
    }

    // Calculate means
    let mean1 = returns1.iter().sum::<f64>() / returns1.len() as f64;
    let mean2 = returns2.iter().sum::<f64>() / returns2.len() as f64;

    // Calculate covariance and standard deviations
    let mut cov = 0.0;
    let mut var1 = 0.0;
    let mut var2 = 0.0;

    for (r1, r2) in returns1.iter().zip(returns2.iter()) {
        let diff1 = r1 - mean1;
        let diff2 = r2 - mean2;
        cov += diff1 * diff2;
        var1 += diff1 * diff1;
        var2 += diff2 * diff2;
    }

    let std1 = var1.sqrt();
    let std2 = var2.sqrt();

    if std1 < f64::EPSILON || std2 < f64::EPSILON {
        return None;
    }

    // Pearson correlation coefficient
    Some(cov / (std1 * std2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn create_test_price_point(date: &str, price: f64) -> PricePoint {
        PricePoint {
            id: Uuid::new_v4(),
            ticker: "TEST".to_string(),
            date: NaiveDate::from_str(date).unwrap(),
            close_price: BigDecimal::from_str(&price.to_string()).unwrap(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_compute_vol_drawdown_with_flat_prices() {
        let series = vec![
            create_test_price_point("2024-01-01", 100.0),
            create_test_price_point("2024-01-02", 100.0),
            create_test_price_point("2024-01-03", 100.0),
        ];

        let (vol, dd) = compute_vol_drawdown(&series);
        assert_eq!(vol, 0.0);
        assert_eq!(dd, 0.0);
    }

    #[test]
    fn test_compute_vol_drawdown_with_decline() {
        let series = vec![
            create_test_price_point("2024-01-01", 100.0),
            create_test_price_point("2024-01-02", 90.0),
            create_test_price_point("2024-01-03", 80.0),
        ];

        let (vol, dd) = compute_vol_drawdown(&series);
        assert!(vol > 0.0); // Should have volatility
        assert!(dd < 0.0); // Should have negative drawdown
        assert!(dd <= -20.0); // At least -20% drawdown
    }

    #[test]
    fn test_score_risk_zero_risk() {
        let risk = PositionRisk {
            volatility: 0.0,
            max_drawdown: 0.0,
            beta: Some(0.0),
            sharpe: Some(0.0),
            value_at_risk: Some(0.0),
        };

        let score = score_risk(&risk);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_risk_high_risk() {
        let risk = PositionRisk {
            volatility: 50.0,     // High volatility
            max_drawdown: -50.0,  // Large drawdown
            beta: Some(2.0),      // High beta
            sharpe: Some(1.0),    // Sharpe ratio doesn't affect score
            value_at_risk: Some(-10.0), // High VaR
        };

        let score = score_risk(&risk);
        assert_eq!(score, 100.0); // Should hit max score
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(RiskLevel::from_score(20.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(50.0), RiskLevel::Moderate);
        assert_eq!(RiskLevel::from_score(80.0), RiskLevel::High);
    }
}

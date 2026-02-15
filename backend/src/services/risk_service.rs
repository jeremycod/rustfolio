use crate::db::price_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::risk::{PositionRisk, RiskAssessment, RiskLevel, RiskDecomposition};
use crate::models::PricePoint;
use crate::services::price_service;
use crate::services::failure_cache::FailureCache;
use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use tracing::{info, warn};

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
/// * `risk_free_rate` – annual risk-free rate for Sharpe/Sortino calculations (e.g., 0.045 for 4.5%)
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
    risk_free_rate: f64,
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
    let sharpe = compute_sharpe(&series, risk_free_rate);
    let sortino = compute_sortino(&series, risk_free_rate);
    let annualized_return = compute_annualized_return(&series);
    let var = compute_var(&series);
    let (var_95, var_99) = compute_var_multi(&series);
    let (es_95, es_99) = compute_expected_shortfall(&series);

    // Compute multi-benchmark betas
    let (beta_spy, beta_qqq, beta_iwm) =
        compute_multi_benchmark_beta(pool, &series, days, price_provider, failure_cache).await;

    // Compute risk decomposition (requires benchmark data)
    let risk_decomposition = if beta.is_some() {
        compute_risk_decomposition(&series, &bench, volatility)
    } else {
        None
    };

    let metrics = PositionRisk {
        volatility,
        max_drawdown,
        beta,
        beta_spy,
        beta_qqq,
        beta_iwm,
        risk_decomposition,
        sharpe,
        sortino,
        annualized_return,
        value_at_risk: var,
        var_95,
        var_99,
        expected_shortfall_95: es_95,
        expected_shortfall_99: es_99,
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

/// Compute the annualized return from a price series.
///
/// Returns the mean daily return extrapolated to one year, expressed as a percentage.
fn compute_annualized_return(series: &[PricePoint]) -> Option<f64> {
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

    // Calculate mean return and annualize
    let mean_daily = returns.iter().sum::<f64>() / returns.len() as f64;
    let annualized = mean_daily * 252.0 * 100.0; // Annualized and convert to percentage

    Some(annualized)
}

/// Compute the annualized Sharpe ratio using the provided risk-free rate.
///
/// The Sharpe ratio measures risk-adjusted return. Higher values indicate better
/// risk-adjusted performance. Formula: (portfolio_return - risk_free_rate) / volatility
///
/// # Arguments
/// * `series` - Price history for the asset
/// * `risk_free_rate` - Annual risk-free rate (e.g., 0.045 for 4.5%)
fn compute_sharpe(series: &[PricePoint], risk_free_rate: f64) -> Option<f64> {
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

    if volatility.abs() < f64::EPSILON {
        return None; // Avoid division by zero
    }

    // Daily risk-free rate
    let risk_free_daily = risk_free_rate / 252.0;

    // Annualized Sharpe ratio
    Some(((mean - risk_free_daily) * 252.0) / volatility)
}

/// Compute the annualized Sortino ratio using the provided risk-free rate.
///
/// The Sortino ratio is similar to Sharpe but only considers downside volatility
/// (negative returns), making it better for assessing downside risk.
/// Formula: (portfolio_return - risk_free_rate) / downside_deviation
///
/// # Arguments
/// * `series` - Price history for the asset
/// * `risk_free_rate` - Annual risk-free rate (e.g., 0.045 for 4.5%)
fn compute_sortino(series: &[PricePoint], risk_free_rate: f64) -> Option<f64> {
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

    // Calculate mean return
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;

    // Daily risk-free rate
    let risk_free_daily = risk_free_rate / 252.0;

    // Calculate downside deviation (only negative returns below risk-free rate)
    let downside_returns: Vec<f64> = returns
        .iter()
        .filter(|&&r| r < risk_free_daily)
        .copied()
        .collect();

    if downside_returns.is_empty() {
        // No negative returns - infinite Sortino ratio, but we'll return None or a high value
        // For practical purposes, return None (can't divide by zero downside deviation)
        return None;
    }

    let downside_variance: f64 = downside_returns
        .iter()
        .map(|r| (r - risk_free_daily).powi(2))
        .sum::<f64>()
        / (downside_returns.len() as f64 - 1.0);

    let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt(); // Annualized

    if downside_deviation.abs() < f64::EPSILON {
        return None; // Avoid division by zero
    }

    // Annualized Sortino ratio
    Some(((mean - risk_free_daily) * 252.0) / downside_deviation)
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

/// Compute Value at Risk (VaR) at multiple confidence levels using historical simulation.
///
/// Returns (var_95, var_99) as a tuple of negative percentages.
/// - var_95: 95% confidence (5% chance of exceeding this loss)
/// - var_99: 99% confidence (1% chance of exceeding this loss)
fn compute_var_multi(series: &[PricePoint]) -> (Option<f64>, Option<f64>) {
    if series.len() < 2 {
        return (None, None);
    }

    // Convert to f64 prices
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() < 2 {
        return (None, None);
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
        return (None, None);
    }

    // Sort returns to find percentiles
    let mut sorted_returns = returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 95% VaR (5th percentile)
    let idx_95 = (sorted_returns.len() as f64 * 0.05).floor() as usize;
    let var_95 = Some(sorted_returns[idx_95] * 100.0); // Convert to percentage

    // 99% VaR (1st percentile)
    let idx_99 = (sorted_returns.len() as f64 * 0.01).floor() as usize;
    let var_99 = Some(sorted_returns[idx_99] * 100.0); // Convert to percentage

    (var_95, var_99)
}

/// Compute Expected Shortfall (CVaR) at 95% and 99% confidence levels.
///
/// Expected Shortfall is the average loss beyond the VaR threshold.
/// It's a more conservative risk measure than VaR.
///
/// Returns (es_95, es_99) as a tuple of negative percentages.
fn compute_expected_shortfall(series: &[PricePoint]) -> (Option<f64>, Option<f64>) {
    if series.len() < 2 {
        return (None, None);
    }

    // Convert to f64 prices
    let prices: Vec<f64> = series
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if prices.len() < 2 {
        return (None, None);
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
        return (None, None);
    }

    // Sort returns
    let mut sorted_returns = returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // ES at 95% confidence (average of worst 5% returns)
    let cutoff_95 = (sorted_returns.len() as f64 * 0.05).ceil() as usize;
    let es_95 = if cutoff_95 > 0 {
        let worst_returns: Vec<f64> = sorted_returns.iter().take(cutoff_95).copied().collect();
        let sum: f64 = worst_returns.iter().sum();
        Some((sum / worst_returns.len() as f64) * 100.0) // Convert to percentage
    } else {
        None
    };

    // ES at 99% confidence (average of worst 1% returns)
    let cutoff_99 = (sorted_returns.len() as f64 * 0.01).ceil() as usize;
    let es_99 = if cutoff_99 > 0 {
        let worst_returns: Vec<f64> = sorted_returns.iter().take(cutoff_99).copied().collect();
        let sum: f64 = worst_returns.iter().sum();
        Some((sum / worst_returns.len() as f64) * 100.0) // Convert to percentage
    } else {
        None
    };

    (es_95, es_99)
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

/// Compute beta against multiple benchmark indices (SPY, QQQ, IWM).
///
/// Returns a tuple of (beta_spy, beta_qqq, beta_iwm) where each beta measures
/// the asset's systematic risk relative to that specific benchmark.
///
/// # Arguments
/// * `pool` – Postgres connection pool
/// * `ticker_series` – Price history for the ticker
/// * `days` – Number of trading days to analyze
/// * `price_provider` – External price data provider
/// * `failure_cache` – Cache to avoid retrying known-bad tickers
///
/// # Returns
/// Tuple of (beta_spy, beta_qqq, beta_iwm) where each is Option<f64>
async fn compute_multi_benchmark_beta(
    pool: &PgPool,
    ticker_series: &[PricePoint],
    days: i64,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let benchmarks = ["SPY", "QQQ", "IWM"];
    let mut betas = Vec::new();

    for benchmark in &benchmarks {
        // Ensure fresh benchmark data
        if let Err(e) = price_service::refresh_from_api(pool, price_provider, benchmark, failure_cache).await {
            warn!("Failed to refresh {} data: {}", benchmark, e);
            betas.push(None);
            continue;
        }

        // Fetch benchmark price history
        match price_queries::fetch_window(pool, benchmark, days).await {
            Ok(bench_series) => {
                if bench_series.len() >= 2 {
                    let beta = compute_beta(ticker_series, &bench_series);
                    betas.push(beta);
                } else {
                    warn!("Insufficient data for benchmark {}", benchmark);
                    betas.push(None);
                }
            }
            Err(e) => {
                warn!("Failed to fetch {} data: {}", benchmark, e);
                betas.push(None);
            }
        }
    }

    (
        betas.get(0).copied().flatten(),
        betas.get(1).copied().flatten(),
        betas.get(2).copied().flatten(),
    )
}

/// Compute risk decomposition: systematic vs idiosyncratic risk.
///
/// Systematic risk is the portion of total risk explained by market movements (beta),
/// while idiosyncratic risk is stock-specific and can be diversified away.
///
/// Formula:
/// - R² = correlation² (% of variance explained by beta)
/// - Systematic risk = R² * total_risk²
/// - Idiosyncratic risk = (1 - R²) * total_risk²
///
/// # Arguments
/// * `ticker_series` – Price history for the ticker
/// * `benchmark_series` – Price history for the benchmark
/// * `total_volatility` – Annualized volatility of the ticker (as %)
///
/// # Returns
/// RiskDecomposition struct with systematic and idiosyncratic risk components
fn compute_risk_decomposition(
    ticker_series: &[PricePoint],
    benchmark_series: &[PricePoint],
    total_volatility: f64,
) -> Option<RiskDecomposition> {
    // Calculate correlation between ticker and benchmark
    let correlation = compute_correlation(ticker_series, benchmark_series)?;

    // R² = correlation²
    let r_squared = correlation.powi(2);

    // Total risk (variance)
    let total_variance = (total_volatility / 100.0).powi(2);

    // Systematic risk (explained by market/beta)
    let systematic_variance = r_squared * total_variance;
    let systematic_risk = (systematic_variance.sqrt() * 100.0).max(0.0);

    // Idiosyncratic risk (stock-specific, diversifiable)
    let idiosyncratic_variance = ((1.0 - r_squared) * total_variance).max(0.0);
    let idiosyncratic_risk = (idiosyncratic_variance.sqrt() * 100.0).max(0.0);

    Some(RiskDecomposition {
        systematic_risk,
        idiosyncratic_risk,
        r_squared,
        total_risk: total_volatility,
    })
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
            beta_spy: Some(0.0),
            beta_qqq: None,
            beta_iwm: None,
            risk_decomposition: None,
            sharpe: Some(0.0),
            sortino: None,
            annualized_return: None,
            value_at_risk: Some(0.0),
            var_95: Some(0.0),
            var_99: Some(0.0),
            expected_shortfall_95: Some(0.0),
            expected_shortfall_99: Some(0.0),
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
            beta_spy: Some(2.0),
            beta_qqq: None,
            beta_iwm: None,
            risk_decomposition: None,
            sharpe: Some(1.0),    // Sharpe ratio doesn't affect score
            sortino: None,
            annualized_return: None,
            value_at_risk: Some(-10.0), // High VaR
            var_95: Some(-10.0),
            var_99: Some(-15.0),
            expected_shortfall_95: Some(-12.0),
            expected_shortfall_99: Some(-18.0),
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

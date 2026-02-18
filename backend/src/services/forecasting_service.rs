use chrono::{Duration, NaiveDate, Utc};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::holding_snapshot_queries;
use crate::errors::AppError;
use crate::external::price_provider::PriceProvider;
use crate::models::{ForecastMethod, ForecastPoint, HistoricalDataPoint, LatestAccountHolding, PortfolioForecast};
use crate::services::failure_cache::FailureCache;
use sqlx::PgPool;

/// Generate a portfolio value forecast
#[allow(dead_code)]
pub async fn generate_portfolio_forecast(
    pool: &PgPool,
    portfolio_id: Uuid,
    days_ahead: i32,
    method: Option<ForecastMethod>,
) -> Result<PortfolioForecast, AppError> {
    info!(
        "Generating forecast for portfolio {} ({} days ahead)",
        portfolio_id, days_ahead
    );

    // Fetch historical portfolio values (adjusted for cash flows to get true growth rates)
    let historical_data = fetch_historical_portfolio_values(pool, portfolio_id).await?;

    if historical_data.len() < 3 {
        return Err(AppError::Validation(
            format!(
                "Insufficient data for forecasting. Need at least 3 data points, got {}",
                historical_data.len()
            )
        ));
    }

    // Get REAL current portfolio value (unadjusted, including deposits)
    let real_current_value = fetch_current_portfolio_value(pool, portfolio_id).await?;

    // Choose forecasting method
    let forecast_method = method.unwrap_or(ForecastMethod::Ensemble);

    // Generate forecast based on method (using adjusted historical data for growth patterns)
    let mut forecast_points = match forecast_method {
        ForecastMethod::LinearRegression => {
            linear_regression_forecast(&historical_data, days_ahead)?
        }
        ForecastMethod::ExponentialSmoothing => {
            exponential_smoothing_forecast(&historical_data, days_ahead)?
        }
        ForecastMethod::MovingAverage => moving_average_forecast(&historical_data, days_ahead)?,
        ForecastMethod::Ensemble => ensemble_forecast(&historical_data, days_ahead)?,
    };

    // Get the adjusted baseline (last point in adjusted data)
    let adjusted_baseline = historical_data
        .last()
        .map(|p| p.value)
        .unwrap_or(0.0);

    // Scale forecasts from adjusted baseline to real current value
    scale_forecasts_to_real_value(&mut forecast_points, adjusted_baseline, real_current_value);

    // Apply reality checks and cap unrealistic growth (based on real current value)
    apply_sanity_caps(&mut forecast_points, real_current_value, days_ahead, historical_data.len());

    // Generate warnings based on data quality and volatility
    let warnings = generate_warnings(&historical_data, &forecast_points);

    Ok(PortfolioForecast {
        portfolio_id: portfolio_id.to_string(),
        current_value: real_current_value,
        forecast_points,
        methodology: forecast_method,
        confidence_level: 0.95,
        warnings,
        generated_at: Utc::now(),
    })
}

/// Fetch the current (latest) portfolio value, unadjusted
#[allow(dead_code)]
async fn fetch_current_portfolio_value(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<f64, AppError> {
    let history = holding_snapshot_queries::fetch_portfolio_value_history(pool, portfolio_id)
        .await
        .map_err(AppError::Db)?;

    if history.is_empty() {
        return Ok(0.0);
    }

    // Get the latest date
    let latest_date = history.iter().map(|r| r.snapshot_date).max();

    if latest_date.is_none() {
        return Ok(0.0);
    }

    let latest_date = latest_date.unwrap();

    // Sum all values for the latest date
    let total: f64 = history
        .iter()
        .filter(|r| r.snapshot_date == latest_date)
        .map(|r| r.total_value.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    Ok(total)
}

/// Scale forecasts from the adjusted baseline to the real current value
/// This preserves growth rates while starting from the correct current value
#[allow(dead_code)]
fn scale_forecasts_to_real_value(
    forecast_points: &mut [ForecastPoint],
    adjusted_baseline: f64,
    real_current_value: f64,
) {
    if adjusted_baseline <= 0.0 {
        // Can't scale from zero, just use the real value as is
        for point in forecast_points.iter_mut() {
            point.predicted_value = real_current_value;
            point.lower_bound = real_current_value * 0.8;
            point.upper_bound = real_current_value * 1.2;
        }
        return;
    }

    // Calculate scaling factor (how much bigger is the real portfolio vs adjusted?)
    let scale_factor = real_current_value / adjusted_baseline;

    // Apply scaling to all forecast points
    for point in forecast_points.iter_mut() {
        point.predicted_value *= scale_factor;
        point.lower_bound *= scale_factor;
        point.upper_bound *= scale_factor;
    }
}

/// Fetch historical portfolio values from database, adjusted for cash flows
#[allow(dead_code)]
async fn fetch_historical_portfolio_values(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<HistoricalDataPoint>, AppError> {
    // Get portfolio value history
    let history = holding_snapshot_queries::fetch_portfolio_value_history(pool, portfolio_id)
        .await
        .map_err(AppError::Db)?;

    // Get all cash flows for this portfolio's accounts
    let cash_flows = fetch_portfolio_cash_flows(pool, portfolio_id).await?;

    // Group by date and sum values across all accounts
    let mut date_values: HashMap<String, f64> = HashMap::new();

    for record in history {
        let date_str = record.snapshot_date.to_string();
        let value = record.total_value.to_string().parse::<f64>().unwrap_or(0.0);

        *date_values.entry(date_str).or_insert(0.0) += value;
    }

    // Convert to sorted vector
    let mut data_points: Vec<HistoricalDataPoint> = date_values
        .into_iter()
        .map(|(date, value)| HistoricalDataPoint { date, value })
        .collect();

    data_points.sort_by(|a, b| a.date.cmp(&b.date));

    // Adjust for cash flows to get true investment growth
    adjust_for_cash_flows(&mut data_points, &cash_flows);

    Ok(data_points)
}

/// Fetch all cash flows for a portfolio
#[allow(dead_code)]
async fn fetch_portfolio_cash_flows(
    pool: &PgPool,
    portfolio_id: Uuid,
) -> Result<Vec<(NaiveDate, f64, String)>, AppError> {
    // Query to get all cash flows for accounts in this portfolio
    let records = sqlx::query!(
        r#"
        SELECT cf.flow_date, cf.amount, cf.flow_type
        FROM cash_flows cf
        JOIN accounts a ON cf.account_id = a.id
        WHERE a.portfolio_id = $1
        ORDER BY cf.flow_date
        "#,
        portfolio_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(records
        .into_iter()
        .map(|r| (r.flow_date, r.amount.to_string().parse::<f64>().unwrap_or(0.0), r.flow_type))
        .collect())
}

/// Adjust historical values to remove the effect of deposits/withdrawals
/// This gives us true investment growth, not growth from new money
#[allow(dead_code)]
fn adjust_for_cash_flows(
    data_points: &mut [HistoricalDataPoint],
    cash_flows: &[(NaiveDate, f64, String)],
) {
    if cash_flows.is_empty() || data_points.is_empty() {
        return;
    }

    // Calculate cumulative net deposits (deposits - withdrawals) up to each date
    let mut cumulative_by_date: HashMap<String, f64> = HashMap::new();
    let mut running_total = 0.0;

    for (date, amount, flow_type) in cash_flows {
        let date_str = date.to_string();

        // Add to running total (deposits positive, withdrawals negative)
        if flow_type == "DEPOSIT" {
            running_total += amount;
        } else if flow_type == "WITHDRAWAL" {
            running_total -= amount;
        }

        cumulative_by_date.insert(date_str.clone(), running_total);

        // Also set cumulative for all future dates until next cash flow
        // This ensures dates between cash flows have the right cumulative amount
    }

    // For each data point, subtract the cumulative deposits up to that date
    // This normalizes to "what would the portfolio be worth with no deposits?"
    for point in data_points.iter_mut() {
        // Find cumulative deposits up to this date
        let point_date = NaiveDate::parse_from_str(&point.date, "%Y-%m-%d").ok();
        if point_date.is_none() {
            continue;
        }
        let point_date = point_date.unwrap();

        // Get cumulative deposits/withdrawals up to (but not including) this date
        let mut cumulative = 0.0;
        for (cf_date, amount, flow_type) in cash_flows {
            if cf_date <= &point_date {
                if flow_type == "DEPOSIT" {
                    cumulative += amount;
                } else if flow_type == "WITHDRAWAL" {
                    cumulative -= amount;
                }
            }
        }

        // Adjust: subtract deposits (since they aren't growth), add back withdrawals
        point.value -= cumulative;

        // Ensure we don't go negative
        if point.value < 0.0 {
            point.value = 0.0;
        }
    }
}

/// Linear regression forecast
fn linear_regression_forecast(
    data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> Result<Vec<ForecastPoint>, AppError> {
    let n = data.len() as f64;
    let values: Vec<f64> = data.iter().map(|p| p.value).collect();

    // Calculate linear regression: y = mx + b
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean) * (x - x_mean);
    }

    let slope = numerator / denominator;
    let intercept = y_mean - slope * x_mean;

    // Calculate residual standard error for confidence intervals
    let mut sum_squared_residuals = 0.0;
    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        let predicted = slope * x + intercept;
        sum_squared_residuals += (y - predicted).powi(2);
    }
    let std_error = (sum_squared_residuals / (n - 2.0)).sqrt();

    // Generate forecast points
    let last_date = chrono::NaiveDate::parse_from_str(&data.last().unwrap().date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;

    let mut forecast_points = Vec::new();

    for day in 1..=days_ahead {
        let x = n + day as f64 - 1.0;
        let predicted_value = slope * x + intercept;

        // Confidence intervals widen with forecast horizon
        let confidence_factor = 1.96 * std_error * (1.0 + (day as f64 / days_ahead as f64));

        let forecast_date = last_date + Duration::days(day as i64);

        forecast_points.push(ForecastPoint {
            date: forecast_date.to_string(),
            predicted_value: predicted_value.max(0.0), // Ensure non-negative
            lower_bound: (predicted_value - confidence_factor).max(0.0),
            upper_bound: predicted_value + confidence_factor,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

/// Exponential smoothing forecast
fn exponential_smoothing_forecast(
    data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> Result<Vec<ForecastPoint>, AppError> {
    let values: Vec<f64> = data.iter().map(|p| p.value).collect();

    // Holt's linear trend method (double exponential smoothing)
    let alpha = 0.3; // Level smoothing parameter
    let beta = 0.1; // Trend smoothing parameter

    let mut level = values[0];
    let mut trend = values[1] - values[0];

    // Apply exponential smoothing to historical data
    for &value in &values[1..] {
        let prev_level = level;
        level = alpha * value + (1.0 - alpha) * (level + trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * trend;
    }

    // Calculate standard deviation for confidence intervals
    let mut residuals = Vec::new();
    let mut test_level = values[0];
    let mut test_trend = values[1] - values[0];

    for &value in &values[1..] {
        let forecast = test_level + test_trend;
        residuals.push(value - forecast);

        let prev_level = test_level;
        test_level = alpha * value + (1.0 - alpha) * (test_level + test_trend);
        test_trend = beta * (test_level - prev_level) + (1.0 - beta) * test_trend;
    }

    let std_dev = calculate_std_dev(&residuals);

    // Generate forecasts
    let last_date = chrono::NaiveDate::parse_from_str(&data.last().unwrap().date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;

    let mut forecast_points = Vec::new();

    for day in 1..=days_ahead {
        let predicted_value = level + trend * day as f64;
        let confidence_factor = 1.96 * std_dev * ((day as f64) / days_ahead as f64).sqrt();

        let forecast_date = last_date + Duration::days(day as i64);

        forecast_points.push(ForecastPoint {
            date: forecast_date.to_string(),
            predicted_value: predicted_value.max(0.0),
            lower_bound: (predicted_value - confidence_factor).max(0.0),
            upper_bound: predicted_value + confidence_factor,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

/// Moving average forecast
fn moving_average_forecast(
    data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> Result<Vec<ForecastPoint>, AppError> {
    let window_size = (data.len() / 3).max(3).min(10); // Adaptive window
    let values: Vec<f64> = data.iter().map(|p| p.value).collect();

    // Calculate moving average of last window
    let recent_avg: f64 = values.iter().rev().take(window_size).sum::<f64>() / window_size as f64;

    // Calculate trend from last two windows
    let first_window: f64 = values
        .iter()
        .rev()
        .skip(window_size)
        .take(window_size)
        .sum::<f64>()
        / window_size as f64;

    let trend = (recent_avg - first_window) / window_size as f64;

    // Calculate volatility for confidence intervals
    let volatility = calculate_std_dev(&values);

    let last_date = chrono::NaiveDate::parse_from_str(&data.last().unwrap().date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;

    let mut forecast_points = Vec::new();

    for day in 1..=days_ahead {
        let predicted_value = recent_avg + trend * day as f64;
        let confidence_factor = 1.96 * volatility * ((day as f64) / days_ahead as f64).sqrt();

        let forecast_date = last_date + Duration::days(day as i64);

        forecast_points.push(ForecastPoint {
            date: forecast_date.to_string(),
            predicted_value: predicted_value.max(0.0),
            lower_bound: (predicted_value - confidence_factor).max(0.0),
            upper_bound: predicted_value + confidence_factor,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

/// Ensemble forecast (weighted average of all methods)
fn ensemble_forecast(
    data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> Result<Vec<ForecastPoint>, AppError> {
    let linear = linear_regression_forecast(data, days_ahead)?;
    let exponential = exponential_smoothing_forecast(data, days_ahead)?;
    let moving_avg = moving_average_forecast(data, days_ahead)?;

    let mut forecast_points = Vec::new();

    for i in 0..days_ahead as usize {
        let predicted_value = linear[i].predicted_value * 0.4
            + exponential[i].predicted_value * 0.4
            + moving_avg[i].predicted_value * 0.2;

        let lower_bound = (linear[i].lower_bound.min(exponential[i].lower_bound)).min(moving_avg[i].lower_bound);

        let upper_bound = (linear[i].upper_bound.max(exponential[i].upper_bound)).max(moving_avg[i].upper_bound);

        forecast_points.push(ForecastPoint {
            date: linear[i].date.clone(),
            predicted_value: predicted_value.max(0.0),
            lower_bound: lower_bound.max(0.0),
            upper_bound,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

/// Apply sanity caps to prevent unrealistic forecasts
fn apply_sanity_caps(
    forecast_points: &mut [ForecastPoint],
    current_value: f64,
    _days_ahead: i32,
    data_points: usize,
) {
    // For short-term forecasts (<= 1 year), use monthly caps
    // For long-term forecasts (> 1 year), use annual compound growth rates

    // Cap each forecast point individually based on how many days ahead it is
    for (i, point) in forecast_points.iter_mut().enumerate() {
        let days_ahead_for_this_point = (i + 1) as f64;
        let years = days_ahead_for_this_point / 365.0;

        let (max_reasonable_value, min_reasonable_value) = if days_ahead_for_this_point <= 365.0 {
            // Short-term (0-1 year): Use monthly growth caps
            let max_monthly_return = if data_points < 10 { 0.08_f64 } else { 0.15_f64 };
            let months = days_ahead_for_this_point / 30.0;

            let max_val = current_value * (1.0_f64 + max_monthly_return).powf(months);
            let min_val = current_value * (1.0_f64 - 0.20_f64).powf(months);
            (max_val, min_val)
        } else {
            // Long-term (1+ years): Use annual compound growth
            // Historical S&P 500 average: ~10% annually
            // Cap at 12% annually for aggressive portfolios
            // Conservative floor: -5% annually (accounts for bad years but long-term growth)
            let max_annual_return = 0.12_f64; // 12% annually
            let min_annual_return = -0.05_f64; // Can decline 5% annually in worst case

            let max_val = current_value * (1.0_f64 + max_annual_return).powf(years);
            let min_val = current_value * (1.0_f64 + min_annual_return).powf(years);
            (max_val, min_val)
        };

        // Cap predicted value
        if point.predicted_value > max_reasonable_value {
            point.predicted_value = max_reasonable_value;
        }
        if point.predicted_value < min_reasonable_value {
            point.predicted_value = min_reasonable_value;
        }

        // Cap confidence bounds
        // For long-term forecasts, confidence intervals widen significantly
        let confidence_multiplier = if years > 1.0 {
            1.0_f64 + (years * 0.15_f64) // Widen by 15% per year
        } else {
            1.0_f64
        };

        // Upper bound: allow 2x the max reasonable growth for confidence interval
        let max_upper = if days_ahead_for_this_point <= 365.0 {
            let max_monthly_return = if data_points < 10 { 0.08_f64 } else { 0.15_f64 };
            let months = days_ahead_for_this_point / 30.0;
            current_value * (1.0_f64 + max_monthly_return * 2.0_f64).powf(months)
        } else {
            current_value * (1.0_f64 + 0.18_f64).powf(years) // 18% annually for upper bound
        } * confidence_multiplier;

        if point.upper_bound > max_upper {
            point.upper_bound = max_upper;
        }

        // Lower bound: more conservative for long-term
        let min_lower = if years > 5.0 {
            current_value * 0.5_f64 // Can drop to 50% over very long periods
        } else if years > 1.0 {
            current_value * 0.7_f64 // Can drop to 70% over multi-year periods
        } else {
            current_value * 0.2_f64 // Can drop to 20% in short term
        };

        if point.lower_bound < min_lower {
            point.lower_bound = min_lower;
        }

        // Ensure bounds make sense relative to predicted value
        if point.lower_bound > point.predicted_value {
            point.lower_bound = point.predicted_value * 0.8_f64;
        }
        if point.upper_bound < point.predicted_value {
            point.upper_bound = point.predicted_value * 1.2_f64;
        }
    }
}

/// Generate warnings based on data quality
#[allow(dead_code)]
fn generate_warnings(
    historical: &[HistoricalDataPoint],
    forecast: &[ForecastPoint],
) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check data quantity
    if historical.len() < 30 {
        warnings.push(format!(
            "Limited historical data ({} points). Forecasts may be less reliable.",
            historical.len()
        ));
    }

    // Warn about very limited data
    if historical.len() < 10 {
        warnings.push(
            "Very limited data (<10 points). Forecasts are capped at 8% monthly growth to prevent unrealistic projections."
                .to_string(),
        );
    }

    // Check volatility
    let values: Vec<f64> = historical.iter().map(|p| p.value).collect();
    let volatility = calculate_std_dev(&values);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let cv = volatility / mean; // Coefficient of variation

    if cv > 0.15 {
        warnings.push(
            "High portfolio volatility detected. Confidence intervals are wider."
                .to_string(),
        );
    }

    // Check forecast reasonableness
    if let (Some(first), Some(last)) = (forecast.first(), forecast.last()) {
        let forecast_change = ((last.predicted_value - first.predicted_value) / first.predicted_value).abs();
        if forecast_change > 0.50 {
            warnings.push(
                "Forecast projects significant change (>50%). This is based on extrapolation and should be viewed with caution."
                    .to_string(),
            );
        }
    }

    // General disclaimer
    warnings.push(
        "Forecasts are statistical projections based on past data. They are not guarantees of future performance."
            .to_string(),
    );

    // Note about cash flow adjustment
    warnings.push(
        "Forecasts are adjusted to exclude deposits and withdrawals, showing only investment growth."
            .to_string(),
    );

    warnings
}

/// Calculate standard deviation
fn calculate_std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

/// Generate forecast using compound growth (for long-term projections)
fn generate_compound_growth_forecast(
    current_value: f64,
    days_ahead: i32,
    annual_return: f64,
    historical_data: &[HistoricalDataPoint],
) -> Result<Vec<ForecastPoint>, AppError> {
    let last_date = chrono::NaiveDate::parse_from_str(&historical_data.last().unwrap().date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;

    // Calculate daily growth rate from annual rate
    let daily_rate = (1.0_f64 + annual_return).powf(1.0_f64 / 365.0_f64) - 1.0_f64;

    // Calculate volatility from historical data for confidence intervals
    let values: Vec<f64> = historical_data.iter().map(|p| p.value).collect();
    let volatility = calculate_std_dev(&values);
    let mean_value = values.iter().sum::<f64>() / values.len() as f64;
    let annual_volatility = if mean_value > 0.0 {
        (volatility / mean_value) * (365.0_f64).sqrt() // Annualized volatility
    } else {
        0.15 // Default 15% volatility
    };

    let mut forecast_points = Vec::new();

    for day in 1..=days_ahead {
        let days_elapsed = day as f64;

        // Compound growth
        let predicted_value = current_value * (1.0_f64 + daily_rate).powf(days_elapsed);

        // Confidence intervals widen with time (square root of time rule)
        let years_elapsed = days_elapsed / 365.0_f64;
        let time_adjusted_volatility = annual_volatility * years_elapsed.sqrt();

        // 95% confidence interval (1.96 standard deviations)
        let confidence_factor = 1.96_f64 * time_adjusted_volatility * predicted_value;

        let forecast_date = last_date + Duration::days(day as i64);

        forecast_points.push(ForecastPoint {
            date: forecast_date.to_string(),
            predicted_value: predicted_value.max(0.0),
            lower_bound: (predicted_value - confidence_factor).max(current_value * 0.3), // Floor at 30% of current
            upper_bound: predicted_value + confidence_factor,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

// ============================================================================
// BENCHMARK-BASED FORECASTING
// ============================================================================

/// Benchmark to use for different asset categories
#[derive(Debug, Clone)]
enum Benchmark {
    Equity,        // SPY (S&P 500)
    FixedIncome,   // AGG (US Bond Index)
    Blended,       // 50/50 SPY/AGG
}

/// Holding grouped by benchmark category
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BenchmarkedHolding {
    ticker: String,
    market_value: f64,
    benchmark: Benchmark,
}

/// Map asset category to appropriate benchmark
fn map_asset_category_to_benchmark(asset_category: &Option<String>) -> Benchmark {
    match asset_category.as_deref() {
        Some("EQUITIES") => Benchmark::Equity,
        Some("FIXED INCOME") => Benchmark::FixedIncome,
        Some("ALTERNATIVES AND OTHER") => Benchmark::Blended,
        _ => Benchmark::Equity, // Default to equity if unknown
    }
}

/// Generate synthetic portfolio history using benchmark returns
/// This allows forecasting when we don't have historical snapshots but have current holdings
pub async fn generate_benchmark_based_forecast(
    pool: &PgPool,
    portfolio_id: Uuid,
    days_ahead: i32,
    method: Option<ForecastMethod>,
    price_provider: &dyn PriceProvider,
    failure_cache: &FailureCache,
) -> Result<PortfolioForecast, AppError> {
    info!(
        "Generating benchmark-based forecast for portfolio {} ({} days ahead)",
        portfolio_id, days_ahead
    );

    // Fetch current holdings for the portfolio
    let holdings = holding_snapshot_queries::fetch_portfolio_latest_holdings(pool, portfolio_id)
        .await
        .map_err(AppError::Db)?;

    if holdings.is_empty() {
        return Err(AppError::Validation(
            "No holdings found for portfolio".to_string()
        ));
    }

    // Calculate current portfolio value
    let current_value: f64 = holdings
        .iter()
        .map(|h| h.market_value.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    if current_value <= 0.0 {
        return Err(AppError::Validation(
            "Portfolio has no value".to_string()
        ));
    }

    // Group holdings by benchmark
    let benchmarked_holdings = categorize_holdings_by_benchmark(&holdings);

    // Calculate weights for each benchmark category
    let equity_value: f64 = benchmarked_holdings
        .iter()
        .filter(|h| matches!(h.benchmark, Benchmark::Equity))
        .map(|h| h.market_value)
        .sum();

    let fixed_income_value: f64 = benchmarked_holdings
        .iter()
        .filter(|h| matches!(h.benchmark, Benchmark::FixedIncome))
        .map(|h| h.market_value)
        .sum();

    let blended_value: f64 = benchmarked_holdings
        .iter()
        .filter(|h| matches!(h.benchmark, Benchmark::Blended))
        .map(|h| h.market_value)
        .sum();

    let equity_weight = equity_value / current_value;
    let fixed_income_weight = fixed_income_value / current_value;
    let blended_weight = blended_value / current_value;

    info!(
        "Portfolio composition: {:.1}% equity, {:.1}% fixed income, {:.1}% blended",
        equity_weight * 100.0,
        fixed_income_weight * 100.0,
        blended_weight * 100.0
    );

    // Fetch benchmark price histories (365 days)
    let days_history = 365;
    let spy_history = fetch_benchmark_history(pool, "SPY", days_history, price_provider, failure_cache).await?;
    let agg_history = fetch_benchmark_history(pool, "AGG", days_history, price_provider, failure_cache).await?;

    // Generate synthetic portfolio history
    let synthetic_history = generate_synthetic_portfolio_history(
        &spy_history,
        &agg_history,
        equity_weight,
        fixed_income_weight,
        blended_weight,
        current_value,
    )?;

    if synthetic_history.len() < 3 {
        return Err(AppError::Validation(
            format!(
                "Insufficient benchmark data for forecasting. Need at least 3 data points, got {}",
                synthetic_history.len()
            )
        ));
    }

    info!(
        "Generated {} synthetic historical data points from benchmarks",
        synthetic_history.len()
    );

    // Choose forecasting method
    let forecast_method = method.unwrap_or(ForecastMethod::Ensemble);

    // For long-term forecasts (> 1 year), use compound growth based on historical averages
    // For short-term, use statistical algorithms
    let mut forecast_points = if days_ahead > 365 {
        // Long-term: Use compound growth based on portfolio composition
        // Historical averages: Equities ~10%, Fixed Income ~4%
        let expected_annual_return =
            equity_weight * 0.10 + // Equities: 10% annually
            fixed_income_weight * 0.04 + // Fixed Income: 4% annually
            blended_weight * 0.07; // Blended: 7% annually

        generate_compound_growth_forecast(
            current_value,
            days_ahead,
            expected_annual_return,
            &synthetic_history,
        )?
    } else {
        // Short-term: Use statistical algorithms
        match forecast_method {
            ForecastMethod::LinearRegression => {
                linear_regression_forecast(&synthetic_history, days_ahead)?
            }
            ForecastMethod::ExponentialSmoothing => {
                exponential_smoothing_forecast(&synthetic_history, days_ahead)?
            }
            ForecastMethod::MovingAverage => moving_average_forecast(&synthetic_history, days_ahead)?,
            ForecastMethod::Ensemble => ensemble_forecast(&synthetic_history, days_ahead)?,
        }
    };

    // Apply sanity caps (validation only, not enforcement)
    apply_sanity_caps(&mut forecast_points, current_value, days_ahead, synthetic_history.len());

    // Generate warnings
    let mut warnings = vec![
        "Forecast based on benchmark data (S&P 500 for equities, Bond Index for fixed income).".to_string(),
        format!(
            "Portfolio composition: {:.0}% equity-like, {:.0}% fixed income-like, {:.0}% blended.",
            equity_weight * 100.0,
            fixed_income_weight * 100.0,
            blended_weight * 100.0
        ),
    ];

    // Add data quality warnings
    if synthetic_history.len() < 90 {
        warnings.push(format!(
            "Limited benchmark data ({} days). Forecasts may be less reliable.",
            synthetic_history.len()
        ));
    }

    // Add warnings for long-term forecasts
    if days_ahead > 365 {
        let years = days_ahead as f64 / 365.0;
        let expected_annual_return =
            equity_weight * 0.10 + fixed_income_weight * 0.04 + blended_weight * 0.07;

        warnings.push(format!(
            "Long-term forecast ({:.1} years): Uses compound growth model with {:.1}% expected annual return \
            (based on your portfolio composition: {:.0}% equity-like, {:.0}% fixed income-like). \
            Predictions become increasingly uncertain over time.",
            years,
            expected_annual_return * 100.0,
            equity_weight * 100.0,
            fixed_income_weight * 100.0
        ));
    }

    if days_ahead > 1825 { // 5+ years
        warnings.push(
            "Very long-term forecasts (5+ years) should be used for general planning only. \
            Actual results will vary significantly due to economic cycles, recessions, inflation, and market regime changes.".to_string()
        );
    }

    warnings.push(
        "Benchmark-based forecasts assume your holdings behave similarly to market indices.".to_string()
    );

    Ok(PortfolioForecast {
        portfolio_id: portfolio_id.to_string(),
        current_value,
        forecast_points,
        methodology: forecast_method,
        confidence_level: 0.95,
        warnings,
        generated_at: Utc::now(),
    })
}

/// Categorize holdings by their appropriate benchmark
fn categorize_holdings_by_benchmark(holdings: &[LatestAccountHolding]) -> Vec<BenchmarkedHolding> {
    holdings
        .iter()
        .map(|h| {
            let market_value = h.market_value.to_string().parse::<f64>().unwrap_or(0.0);
            let benchmark = map_asset_category_to_benchmark(&h.asset_category);

            BenchmarkedHolding {
                ticker: h.ticker.clone(),
                market_value,
                benchmark,
            }
        })
        .collect()
}

/// Fetch benchmark price history
async fn fetch_benchmark_history(
    pool: &PgPool,
    ticker: &str,
    days: i64,
    price_provider: &dyn PriceProvider,
    _failure_cache: &FailureCache,
) -> Result<Vec<(NaiveDate, f64)>, AppError> {
    // First try to get from database
    let cutoff_date = Utc::now().date_naive() - Duration::days(days);

    let prices = sqlx::query!(
        "SELECT date, close_price FROM price_points
         WHERE ticker = $1 AND date >= $2
         ORDER BY date",
        ticker,
        cutoff_date
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    // If we have enough data, use it
    if prices.len() >= (days as usize / 2) {
        return Ok(prices
            .into_iter()
            .map(|p| (p.date, p.close_price.to_string().parse::<f64>().unwrap_or(0.0)))
            .collect());
    }

    // Otherwise, try to fetch from API
    warn!(
        "Insufficient {} data in database ({} points). Attempting to fetch from API...",
        ticker,
        prices.len()
    );

    match price_provider.fetch_daily_history(ticker, days as u32).await {
        Ok(api_prices) => {
            // Convert to (NaiveDate, f64) and store in database
            let mut result = Vec::new();

            for point in &api_prices {
                let price_f64 = point.close.to_string().parse::<f64>().unwrap_or(0.0);
                result.push((point.date, price_f64));

                // Store in database for future use
                let _ = sqlx::query!(
                    "INSERT INTO price_points (id, ticker, date, close_price)
                     VALUES (gen_random_uuid(), $1, $2, $3)
                     ON CONFLICT (ticker, date) DO NOTHING",
                    ticker,
                    point.date,
                    point.close
                )
                .execute(pool)
                .await;
            }

            Ok(result)
        }
        Err(e) => {
            Err(AppError::Validation(format!(
                "Failed to fetch benchmark {} data: {}. Cannot generate benchmark-based forecast.",
                ticker, e
            )))
        }
    }
}

/// Generate synthetic portfolio history using weighted benchmark returns
fn generate_synthetic_portfolio_history(
    spy_history: &[(NaiveDate, f64)],
    agg_history: &[(NaiveDate, f64)],
    equity_weight: f64,
    fixed_income_weight: f64,
    blended_weight: f64,
    current_value: f64,
) -> Result<Vec<HistoricalDataPoint>, AppError> {
    if spy_history.is_empty() || agg_history.is_empty() {
        return Err(AppError::Validation(
            "Benchmark history is empty".to_string()
        ));
    }

    // Find common dates (where we have both SPY and AGG data)
    let spy_map: HashMap<NaiveDate, f64> = spy_history.iter().copied().collect();
    let agg_map: HashMap<NaiveDate, f64> = agg_history.iter().copied().collect();

    let mut common_dates: Vec<NaiveDate> = spy_map
        .keys()
        .filter(|date| agg_map.contains_key(date))
        .copied()
        .collect();

    common_dates.sort();

    if common_dates.len() < 3 {
        return Err(AppError::Validation(
            format!("Insufficient overlapping benchmark data: {} days", common_dates.len())
        ));
    }

    // Calculate portfolio values for each historical date
    // We work backwards from current value, applying inverse returns
    let mut synthetic_history = Vec::new();

    // Start with the most recent date (today's value)
    let latest_date = common_dates.last().unwrap();
    synthetic_history.push(HistoricalDataPoint {
        date: latest_date.to_string(),
        value: current_value,
    });

    // Calculate daily returns and work backwards
    let latest_spy = spy_map[latest_date];
    let latest_agg = agg_map[latest_date];

    for date in common_dates.iter().rev().skip(1) {
        let spy_price = spy_map[date];
        let agg_price = agg_map[date];

        // Calculate returns from this date to latest date
        let spy_return = latest_spy / spy_price;
        let agg_return = latest_agg / agg_price;

        // Weighted portfolio return
        // For blended: use 50/50 SPY/AGG
        let portfolio_return =
            equity_weight * spy_return +
            fixed_income_weight * agg_return +
            blended_weight * (0.5 * spy_return + 0.5 * agg_return);

        // Calculate what the portfolio value would have been on this date
        let historical_value = current_value / portfolio_return;

        synthetic_history.push(HistoricalDataPoint {
            date: date.to_string(),
            value: historical_value.max(0.0),
        });
    }

    // Reverse to get chronological order
    synthetic_history.reverse();

    Ok(synthetic_history)
}

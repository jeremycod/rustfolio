use chrono::{Duration, NaiveDate, Utc};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::db::holding_snapshot_queries;
use crate::errors::AppError;
use crate::models::{ForecastMethod, ForecastPoint, HistoricalDataPoint, PortfolioForecast};
use sqlx::PgPool;

/// Generate a portfolio value forecast
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
        let x = (n + day as f64 - 1.0);
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
        let predicted_value = (linear[i].predicted_value * 0.4
            + exponential[i].predicted_value * 0.4
            + moving_avg[i].predicted_value * 0.2);

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
    // Maximum reasonable monthly return: 15% (180% annually - already very aggressive)
    // For limited data (<10 points), be even more conservative: 8% monthly
    let max_monthly_return = if data_points < 10 { 0.08_f64 } else { 0.15_f64 };

    // Cap each forecast point individually based on how many days ahead it is
    for (i, point) in forecast_points.iter_mut().enumerate() {
        let days_ahead_for_this_point = (i + 1) as f64; // Day 1, Day 2, etc.
        let months = days_ahead_for_this_point / 30.0;

        // Calculate max reasonable value for THIS specific point
        let max_reasonable_value = current_value * (1.0_f64 + max_monthly_return).powf(months);

        // Also set a minimum (max reasonable decline: -20% monthly)
        let min_reasonable_value = current_value * (1.0_f64 - 0.20_f64).powf(months);

        // Cap predicted value
        if point.predicted_value > max_reasonable_value {
            point.predicted_value = max_reasonable_value;
        }
        if point.predicted_value < min_reasonable_value {
            point.predicted_value = min_reasonable_value;
        }

        // Cap confidence bounds to be within reasonable range
        // Upper bound: allow 2x the max reasonable growth (for confidence interval)
        let max_upper = current_value * (1.0_f64 + max_monthly_return * 2.0_f64).powf(months);
        if point.upper_bound > max_upper {
            point.upper_bound = max_upper;
        }

        // Lower bound: can't go below 20% of current value
        let min_lower = current_value * 0.2_f64;
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

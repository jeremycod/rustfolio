use crate::errors::AppError;
use crate::models::forecast::{BetaForecast, BetaForecastPoint, BetaRegimeChange, ForecastMethod};
use crate::models::risk::BetaPoint;
use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;

/// Generate beta forecast for a position
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `ticker` - Ticker symbol to forecast
/// * `benchmark` - Benchmark ticker (e.g., SPY)
/// * `days_ahead` - Number of days to forecast (1-90)
/// * `method` - Forecasting method (defaults to Ensemble)
/// * `price_provider` - Price data provider
/// * `failure_cache` - Failure cache for rate limiting
pub async fn generate_beta_forecast(
    pool: &PgPool,
    ticker: &str,
    benchmark: &str,
    days_ahead: i32,
    method: Option<ForecastMethod>,
    price_provider: &dyn crate::external::price_provider::PriceProvider,
    failure_cache: &crate::services::failure_cache::FailureCache,
) -> Result<BetaForecast, AppError> {
    let method = method.unwrap_or(ForecastMethod::Ensemble);

    // Check cache first
    if let Some(cached_forecast) = check_cache(pool, ticker, benchmark, days_ahead, &method).await? {
        return Ok(cached_forecast);
    }

    // Fetch historical rolling beta data (need at least 90 days)
    let rolling_beta = crate::services::risk_service::compute_rolling_beta(
        pool,
        ticker,
        benchmark,
        365, // Get a year of data for better forecasting
        price_provider,
        failure_cache,
    )
    .await?;

    // Validate we have enough data
    if rolling_beta.beta_90d.len() < 60 {
        return Err(AppError::External(format!(
            "Insufficient historical beta data for forecasting. Need at least 60 days, got {}",
            rolling_beta.beta_90d.len()
        )));
    }

    // Detect regime changes in historical data
    let regime_changes = detect_regime_changes(&rolling_beta.beta_90d);

    // Generate forecast points based on selected method
    let forecast_points = match method {
        ForecastMethod::MovingAverage => {
            // Use MovingAverage enum variant for mean reversion
            mean_reversion_forecast(
                rolling_beta.current_beta,
                rolling_beta.beta_volatility,
                days_ahead,
            )
        }
        ForecastMethod::ExponentialSmoothing => {
            exponential_smoothing_forecast(
                &rolling_beta.beta_90d,
                rolling_beta.beta_volatility,
                days_ahead,
            )
        }
        ForecastMethod::LinearRegression => {
            linear_regression_forecast(
                &rolling_beta.beta_90d,
                rolling_beta.beta_volatility,
                days_ahead,
            )
        }
        ForecastMethod::Ensemble => {
            ensemble_forecast(
                &rolling_beta.beta_90d,
                rolling_beta.current_beta,
                rolling_beta.beta_volatility,
                days_ahead,
            )
        }
    };

    // Generate warnings based on data quality
    let mut warnings = Vec::new();

    if rolling_beta.beta_volatility > 0.5 {
        warnings.push("High beta volatility detected. Forecast confidence may be lower.".to_string());
    }

    if !regime_changes.is_empty() {
        let recent_changes: Vec<&BetaRegimeChange> = regime_changes
            .iter()
            .filter(|rc| {
                if let Ok(date) = NaiveDate::parse_from_str(&rc.date, "%Y-%m-%d") {
                    (Utc::now().naive_utc().date() - date).num_days() < 30
                } else {
                    false
                }
            })
            .collect();

        if !recent_changes.is_empty() {
            warnings.push(format!(
                "Recent regime change detected (within last 30 days). Forecast may not reflect new regime."
            ));
        }
    }

    if rolling_beta.beta_90d.len() < 90 {
        warnings.push(format!(
            "Limited historical data ({} days). Forecast confidence may be lower.",
            rolling_beta.beta_90d.len()
        ));
    }

    let forecast = BetaForecast {
        ticker: ticker.to_string(),
        benchmark: benchmark.to_string(),
        current_beta: rolling_beta.current_beta,
        beta_volatility: rolling_beta.beta_volatility,
        forecast_points,
        methodology: method.clone(),
        confidence_level: 0.95,
        regime_changes,
        warnings,
        generated_at: Utc::now(),
    };

    // Cache the result
    save_to_cache(pool, &forecast, days_ahead, &method).await?;

    Ok(forecast)
}

/// Mean reversion forecast: beta tends toward 1.0 over time
fn mean_reversion_forecast(
    current_beta: f64,
    beta_volatility: f64,
    days_ahead: i32,
) -> Vec<BetaForecastPoint> {
    let mut forecast_points = Vec::new();
    let decay_rate = 0.005; // Half-life ~140 days
    let start_date = Utc::now().naive_utc().date();

    for day in 1..=days_ahead {
        let alpha = (-decay_rate * day as f64).exp();
        let predicted_beta = alpha * current_beta + (1.0 - alpha) * 1.0;

        // Confidence intervals widen with time
        let time_factor = (day as f64 / 30.0).sqrt();
        let std_dev = beta_volatility * time_factor;
        let confidence_factor = 1.96 * std_dev; // 95% CI

        let date = start_date + Duration::days(day as i64);

        forecast_points.push(BetaForecastPoint {
            date: date.format("%Y-%m-%d").to_string(),
            predicted_beta,
            lower_bound: (predicted_beta - confidence_factor).max(0.0),
            upper_bound: (predicted_beta + confidence_factor).min(3.0),
            confidence_level: 0.95,
        });
    }

    forecast_points
}

/// Exponential smoothing forecast: captures recent trends
fn exponential_smoothing_forecast(
    historical_beta: &[BetaPoint],
    beta_volatility: f64,
    days_ahead: i32,
) -> Vec<BetaForecastPoint> {
    let mut forecast_points = Vec::new();
    let start_date = Utc::now().naive_utc().date();

    if historical_beta.is_empty() {
        return forecast_points;
    }

    // Calculate level and trend using double exponential smoothing
    let alpha = 0.3; // Level smoothing factor
    let beta = 0.1; // Trend smoothing factor

    let mut level = historical_beta.last().unwrap().beta;
    let mut trend = if historical_beta.len() > 1 {
        historical_beta.last().unwrap().beta - historical_beta[historical_beta.len() - 2].beta
    } else {
        0.0
    };

    // Apply smoothing to recent data
    for window in historical_beta.windows(2).rev().take(10) {
        let observed = window[1].beta;
        let prev_level = level;

        level = alpha * observed + (1.0 - alpha) * (level + trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * trend;
    }

    // Generate forecast
    for day in 1..=days_ahead {
        let predicted_beta = level + trend * day as f64;

        // Confidence intervals widen with time
        let time_factor = (day as f64 / 30.0).sqrt();
        let std_dev = beta_volatility * time_factor * 1.2; // Slightly wider for trend method
        let confidence_factor = 1.96 * std_dev;

        let date = start_date + Duration::days(day as i64);

        forecast_points.push(BetaForecastPoint {
            date: date.format("%Y-%m-%d").to_string(),
            predicted_beta: predicted_beta.clamp(0.0, 3.0),
            lower_bound: (predicted_beta - confidence_factor).max(0.0),
            upper_bound: (predicted_beta + confidence_factor).min(3.0),
            confidence_level: 0.95,
        });
    }

    forecast_points
}

/// Linear regression forecast: simple trend extrapolation
fn linear_regression_forecast(
    historical_beta: &[BetaPoint],
    beta_volatility: f64,
    days_ahead: i32,
) -> Vec<BetaForecastPoint> {
    let mut forecast_points = Vec::new();
    let start_date = Utc::now().naive_utc().date();

    if historical_beta.len() < 2 {
        return forecast_points;
    }

    // Take last 30 points for trend calculation
    let recent_points: Vec<&BetaPoint> = historical_beta.iter().rev().take(30).rev().collect();

    // Simple linear regression: y = a + bx
    let n = recent_points.len() as f64;
    let sum_x: f64 = (1..=recent_points.len()).map(|i| i as f64).sum();
    let sum_y: f64 = recent_points.iter().map(|p| p.beta).sum();
    let sum_xy: f64 = recent_points
        .iter()
        .enumerate()
        .map(|(i, p)| (i + 1) as f64 * p.beta)
        .sum();
    let sum_x_sq: f64 = (1..=recent_points.len()).map(|i| (i * i) as f64).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_sq - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    // Generate forecast
    let last_x = recent_points.len() as f64;

    for day in 1..=days_ahead {
        let x = last_x + day as f64;
        let predicted_beta = intercept + slope * x;

        // Confidence intervals widen with time
        let time_factor = (day as f64 / 30.0).sqrt();
        let std_dev = beta_volatility * time_factor * 1.3; // Wider for linear method
        let confidence_factor = 1.96 * std_dev;

        let date = start_date + Duration::days(day as i64);

        forecast_points.push(BetaForecastPoint {
            date: date.format("%Y-%m-%d").to_string(),
            predicted_beta: predicted_beta.clamp(0.0, 3.0),
            lower_bound: (predicted_beta - confidence_factor).max(0.0),
            upper_bound: (predicted_beta + confidence_factor).min(3.0),
            confidence_level: 0.95,
        });
    }

    forecast_points
}

/// Ensemble forecast: weighted combination of methods
/// 60% mean reversion + 30% exponential smoothing + 10% linear
fn ensemble_forecast(
    historical_beta: &[BetaPoint],
    current_beta: f64,
    beta_volatility: f64,
    days_ahead: i32,
) -> Vec<BetaForecastPoint> {
    let mean_rev = mean_reversion_forecast(current_beta, beta_volatility, days_ahead);
    let exp_smooth = exponential_smoothing_forecast(historical_beta, beta_volatility, days_ahead);
    let linear = linear_regression_forecast(historical_beta, beta_volatility, days_ahead);

    let mut forecast_points = Vec::new();

    for i in 0..days_ahead as usize {
        if i >= mean_rev.len() || i >= exp_smooth.len() || i >= linear.len() {
            break;
        }

        let predicted_beta = 0.6 * mean_rev[i].predicted_beta
            + 0.3 * exp_smooth[i].predicted_beta
            + 0.1 * linear[i].predicted_beta;

        let lower_bound = 0.6 * mean_rev[i].lower_bound
            + 0.3 * exp_smooth[i].lower_bound
            + 0.1 * linear[i].lower_bound;

        let upper_bound = 0.6 * mean_rev[i].upper_bound
            + 0.3 * exp_smooth[i].upper_bound
            + 0.1 * linear[i].upper_bound;

        forecast_points.push(BetaForecastPoint {
            date: mean_rev[i].date.clone(),
            predicted_beta: predicted_beta.clamp(0.0, 3.0),
            lower_bound: lower_bound.max(0.0),
            upper_bound: upper_bound.min(3.0),
            confidence_level: 0.95,
        });
    }

    forecast_points
}

/// Detect regime changes in historical beta data using z-score test
fn detect_regime_changes(beta_points: &[BetaPoint]) -> Vec<BetaRegimeChange> {
    let window = 30; // 30-day windows for comparison
    let mut changes = Vec::new();

    if beta_points.len() < window * 2 {
        return changes;
    }

    for i in window..beta_points.len() - window {
        let before = &beta_points[i - window..i];
        let after = &beta_points[i..i + window];

        let mean_before = before.iter().map(|p| p.beta).sum::<f64>() / before.len() as f64;
        let mean_after = after.iter().map(|p| p.beta).sum::<f64>() / after.len() as f64;

        // Calculate standard deviation of before period
        let variance_before = before
            .iter()
            .map(|p| (p.beta - mean_before).powi(2))
            .sum::<f64>()
            / before.len() as f64;
        let std_before = variance_before.sqrt();

        if std_before < 0.01 {
            continue; // Skip if variance is too low
        }

        // Z-score test for significant change
        let z_score = (mean_after - mean_before).abs() / std_before;

        if z_score > 2.0 {
            let regime_type = classify_regime(mean_before, mean_after, std_before);

            changes.push(BetaRegimeChange {
                date: beta_points[i].date.clone(),
                beta_before: mean_before,
                beta_after: mean_after,
                z_score,
                regime_type,
            });
        }
    }

    changes
}

/// Classify the type of regime change
fn classify_regime(mean_before: f64, mean_after: f64, std_dev: f64) -> String {
    let change = mean_after - mean_before;

    if std_dev > 0.3 {
        "high_volatility".to_string()
    } else if change.abs() > 0.5 {
        "structural_break".to_string()
    } else if (mean_before - 1.0).abs() > 0.3 && (mean_after - 1.0).abs() < 0.2 {
        "mean_reversion".to_string()
    } else if change > 0.0 {
        "increasing_beta".to_string()
    } else {
        "decreasing_beta".to_string()
    }
}

/// Check cache for existing forecast
async fn check_cache(
    pool: &PgPool,
    ticker: &str,
    benchmark: &str,
    days_ahead: i32,
    method: &ForecastMethod,
) -> Result<Option<BetaForecast>, AppError> {
    let method_str = format!("{:?}", method).to_lowercase();

    let cache_result = sqlx::query!(
        r#"
        SELECT
            forecast_points,
            regime_changes,
            warnings,
            current_beta,
            beta_volatility,
            calculated_at,
            expires_at
        FROM beta_forecast_cache
        WHERE ticker = $1
          AND benchmark = $2
          AND days_ahead = $3
          AND method = $4
          AND expires_at > CURRENT_TIMESTAMP
        "#,
        ticker,
        benchmark,
        days_ahead,
        method_str
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Db(e))?;

    if let Some(cached) = cache_result {
        let forecast_points: Vec<BetaForecastPoint> =
            serde_json::from_value(cached.forecast_points).unwrap_or_default();
        let regime_changes: Vec<BetaRegimeChange> =
            serde_json::from_value(cached.regime_changes).unwrap_or_default();
        let warnings: Vec<String> = serde_json::from_value(cached.warnings).unwrap_or_default();

        Ok(Some(BetaForecast {
            ticker: ticker.to_string(),
            benchmark: benchmark.to_string(),
            current_beta: cached.current_beta,
            beta_volatility: cached.beta_volatility,
            forecast_points,
            methodology: method.clone(),
            confidence_level: 0.95,
            regime_changes,
            warnings,
            generated_at: cached.calculated_at.and_utc(),
        }))
    } else {
        Ok(None)
    }
}

/// Save forecast to cache
async fn save_to_cache(
    pool: &PgPool,
    forecast: &BetaForecast,
    days_ahead: i32,
    method: &ForecastMethod,
) -> Result<(), AppError> {
    let method_str = format!("{:?}", method).to_lowercase();
    let calculated_at = Utc::now().naive_utc();
    let expires_at = calculated_at + chrono::Duration::hours(24);

    let forecast_points_json = serde_json::to_value(&forecast.forecast_points)
        .map_err(|e| AppError::External(e.to_string()))?;
    let regime_changes_json = serde_json::to_value(&forecast.regime_changes)
        .map_err(|e| AppError::External(e.to_string()))?;
    let warnings_json =
        serde_json::to_value(&forecast.warnings).map_err(|e| AppError::External(e.to_string()))?;

    sqlx::query!(
        r#"
        INSERT INTO beta_forecast_cache
        (ticker, benchmark, days_ahead, method, calculated_at, expires_at,
         forecast_points, regime_changes, warnings, current_beta, beta_volatility)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (ticker, benchmark, days_ahead, method)
        DO UPDATE SET
            calculated_at = $5,
            expires_at = $6,
            forecast_points = $7,
            regime_changes = $8,
            warnings = $9,
            current_beta = $10,
            beta_volatility = $11
        "#,
        &forecast.ticker,
        &forecast.benchmark,
        days_ahead,
        method_str,
        calculated_at,
        expires_at,
        forecast_points_json,
        regime_changes_json,
        warnings_json,
        forecast.current_beta,
        forecast.beta_volatility
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::Db(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_reversion_forecast() {
        let forecast = mean_reversion_forecast(1.5, 0.2, 30);

        assert_eq!(forecast.len(), 30);
        assert!(forecast[0].predicted_beta > 1.4);
        assert!(forecast[29].predicted_beta < forecast[0].predicted_beta);

        // Confidence intervals should widen over time
        let ci_width_day1 = forecast[0].upper_bound - forecast[0].lower_bound;
        let ci_width_day30 = forecast[29].upper_bound - forecast[29].lower_bound;
        assert!(ci_width_day30 > ci_width_day1);
    }

    #[test]
    fn test_regime_change_detection() {
        let mut beta_points = Vec::new();

        // Create stable period with beta = 1.0
        for i in 0..50 {
            beta_points.push(BetaPoint {
                date: format!("2024-01-{:02}", i + 1),
                beta: 1.0 + (i as f64 * 0.001),
                r_squared: 0.8,
                alpha: None,
            });
        }

        // Create regime shift to beta = 1.5
        for i in 0..50 {
            beta_points.push(BetaPoint {
                date: format!("2024-02-{:02}", i + 1),
                beta: 1.5 + (i as f64 * 0.001),
                r_squared: 0.8,
                alpha: None,
            });
        }

        let changes = detect_regime_changes(&beta_points);

        assert!(!changes.is_empty());
        assert!(changes[0].z_score > 2.0);
    }

    #[test]
    fn test_ensemble_forecast() {
        let mut beta_points = Vec::new();
        for i in 0..90 {
            beta_points.push(BetaPoint {
                date: format!("2024-01-{:02}", i + 1),
                beta: 1.2,
                r_squared: 0.8,
                alpha: None,
            });
        }

        let forecast = ensemble_forecast(&beta_points, 1.2, 0.15, 30);

        assert_eq!(forecast.len(), 30);
        assert!(forecast[0].predicted_beta > 1.0);
        assert!(forecast[0].predicted_beta < 1.5);
    }
}

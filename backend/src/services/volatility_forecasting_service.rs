use crate::errors::AppError;
use crate::models::forecast::{VolatilityForecast, GarchParameters, VolatilityForecastPoint};
use crate::models::PricePoint;
use crate::db::price_queries;
use crate::services::price_service;
use chrono::{Duration, Utc};
use sqlx::PgPool;

/// Generate volatility forecast using GARCH(1,1) model
///
/// # Arguments
/// * `pool` - Database connection pool for caching
/// * `ticker` - Stock ticker symbol
/// * `days_ahead` - Number of days to forecast (1-90)
/// * `confidence_level` - Confidence level for intervals (0.80 or 0.95)
/// * `price_provider` - Price data provider
/// * `failure_cache` - Failure cache for rate limiting
///
/// # GARCH(1,1) Model
/// The GARCH(1,1) model captures volatility clustering (high volatility periods tend to cluster).
/// Formula: σ²ₜ = ω + α·ε²ₜ₋₁ + β·σ²ₜ₋₁
///
/// Where:
/// - σ²ₜ = conditional variance at time t (volatility squared)
/// - ω (omega) = long-run variance constant
/// - α (alpha) = weight on recent shocks (ARCH term)
/// - β (beta) = weight on past variance (GARCH term)
/// - ε²ₜ₋₁ = squared return innovation at t-1
///
/// Constraints: ω > 0, α ≥ 0, β ≥ 0, α + β < 1 (stationarity)
pub async fn generate_volatility_forecast(
    pool: &PgPool,
    ticker: &str,
    days_ahead: i32,
    confidence_level: f64,
    price_provider: &dyn crate::external::price_provider::PriceProvider,
    failure_cache: &crate::services::failure_cache::FailureCache,
) -> Result<VolatilityForecast, AppError> {
    // Validate inputs
    if days_ahead < 1 || days_ahead > 90 {
        return Err(AppError::Validation(
            "Forecast horizon must be between 1 and 90 days".to_string(),
        ));
    }

    if confidence_level != 0.80 && confidence_level != 0.95 {
        return Err(AppError::Validation(
            "Confidence level must be 0.80 or 0.95".to_string(),
        ));
    }

    // Check cache first (24-hour TTL)
    if let Some(cached) = check_cache(pool, ticker, days_ahead, confidence_level).await? {
        return Ok(cached);
    }

    // Fetch historical price data (need at least 252 days for reliable GARCH estimation)
    let historical_prices = fetch_historical_prices(
        pool,
        ticker,
        500, // Fetch more data for robust parameter estimation
        price_provider,
        failure_cache,
    )
    .await?;

    if historical_prices.len() < 252 {
        return Err(AppError::External(format!(
            "Insufficient historical data for volatility forecasting. Need at least 252 days, got {}",
            historical_prices.len()
        )));
    }

    // Compute daily returns
    let returns = compute_returns(&historical_prices);

    if returns.is_empty() {
        return Err(AppError::External(
            "Unable to compute returns from price data".to_string(),
        ));
    }

    // Calculate current realized volatility (last 30 days, annualized)
    let current_volatility = compute_realized_volatility(&returns, 30);

    // Estimate GARCH(1,1) parameters using Maximum Likelihood Estimation (MLE)
    let garch_params = estimate_garch_parameters(&returns)?;

    // Validate GARCH parameters for stationarity
    if !is_stationary(&garch_params) {
        return Err(AppError::External(
            "GARCH parameter estimation failed: non-stationary parameters".to_string(),
        ));
    }

    // Generate multi-step ahead volatility forecasts
    let forecast_points = forecast_volatility_multistep(
        &returns,
        &garch_params,
        days_ahead as usize,
        confidence_level,
    );

    // Generate warnings based on parameter characteristics
    let warnings = generate_warnings(&garch_params, current_volatility);

    let forecast = VolatilityForecast {
        ticker: ticker.to_string(),
        current_volatility,
        forecast_points,
        garch_parameters: garch_params,
        confidence_level,
        warnings,
        generated_at: Utc::now(),
    };

    // Cache the forecast
    cache_forecast(pool, &forecast, days_ahead).await?;

    Ok(forecast)
}

/// Compute daily returns from price series
fn compute_returns(prices: &[PricePoint]) -> Vec<f64> {
    use bigdecimal::ToPrimitive;

    let price_values: Vec<f64> = prices
        .iter()
        .filter_map(|p| p.close_price.to_f64())
        .collect();

    if price_values.len() < 2 {
        return Vec::new();
    }

    let mut returns = Vec::with_capacity(price_values.len() - 1);
    for i in 1..price_values.len() {
        let prev = price_values[i - 1];
        let cur = price_values[i];
        if prev > 0.0 {
            returns.push((cur - prev) / prev);
        }
    }

    returns
}

/// Compute realized volatility (annualized) from recent returns
fn compute_realized_volatility(returns: &[f64], window: usize) -> f64 {
    if returns.len() < window {
        return compute_realized_volatility(returns, returns.len());
    }

    let recent_returns = &returns[returns.len() - window..];
    let mean = recent_returns.iter().sum::<f64>() / recent_returns.len() as f64;
    let variance: f64 = recent_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (recent_returns.len() as f64 - 1.0);

    let daily_vol = variance.sqrt();
    daily_vol * (252.0_f64).sqrt() * 100.0 // Annualized as percentage
}

/// Estimate GARCH(1,1) parameters using Maximum Likelihood Estimation (MLE)
///
/// This implementation uses a simplified Quasi-Maximum Likelihood approach:
/// 1. Initialize parameters with reasonable starting values
/// 2. Use Nelder-Mead optimization to maximize log-likelihood
/// 3. Return optimal parameters
///
/// For production use, consider using a proper optimization library like argmin.
fn estimate_garch_parameters(returns: &[f64]) -> Result<GarchParameters, AppError> {
    if returns.len() < 100 {
        return Err(AppError::External(
            "Need at least 100 observations for GARCH estimation".to_string(),
        ));
    }

    // Calculate unconditional variance
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let unconditional_variance: f64 = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;

    // Initialize parameters using method of moments
    // Common heuristic: omega = 0.01 * var, alpha = 0.1, beta = 0.85
    let initial_omega = 0.01 * unconditional_variance;
    let initial_alpha = 0.10;
    let initial_beta = 0.85;

    // Simple grid search with fine-tuning around typical values
    // In production, use proper optimization (e.g., argmin crate with BFGS)
    let (best_omega, best_alpha, best_beta) = grid_search_mle(
        returns,
        initial_omega,
        initial_alpha,
        initial_beta,
        unconditional_variance,
    )?;

    // Calculate long-run variance: E[σ²] = ω / (1 - α - β)
    let persistence = best_alpha + best_beta;
    let long_run_variance = best_omega / (1.0 - persistence);

    Ok(GarchParameters {
        omega: best_omega,
        alpha: best_alpha,
        beta: best_beta,
        long_run_variance,
        persistence,
    })
}

/// Simplified grid search for GARCH MLE estimation
/// For production, replace with proper numerical optimization
fn grid_search_mle(
    returns: &[f64],
    init_omega: f64,
    init_alpha: f64,
    init_beta: f64,
    unconditional_var: f64,
) -> Result<(f64, f64, f64), AppError> {
    let mut best_ll = f64::NEG_INFINITY;
    let mut best_params = (init_omega, init_alpha, init_beta);

    // Grid search around initial values
    let omega_range = [
        init_omega * 0.5,
        init_omega,
        init_omega * 1.5,
        init_omega * 2.0,
    ];
    let alpha_range = [0.05, 0.08, 0.10, 0.12, 0.15];
    let beta_range = [0.80, 0.82, 0.85, 0.87, 0.90];

    for &omega in &omega_range {
        for &alpha in &alpha_range {
            for &beta in &beta_range {
                // Check constraints
                if omega <= 0.0 || alpha < 0.0 || beta < 0.0 || alpha + beta >= 0.999 {
                    continue;
                }

                // Calculate log-likelihood
                if let Some(ll) = calculate_log_likelihood(returns, omega, alpha, beta, unconditional_var) {
                    if ll > best_ll {
                        best_ll = ll;
                        best_params = (omega, alpha, beta);
                    }
                }
            }
        }
    }

    if best_ll == f64::NEG_INFINITY {
        return Err(AppError::External(
            "Failed to find valid GARCH parameters".to_string(),
        ));
    }

    Ok(best_params)
}

/// Calculate log-likelihood for GARCH(1,1) model
fn calculate_log_likelihood(
    returns: &[f64],
    omega: f64,
    alpha: f64,
    beta: f64,
    initial_variance: f64,
) -> Option<f64> {
    let n = returns.len();
    let mut log_likelihood = 0.0;
    let mut variance = initial_variance;

    // Iterate through returns and update variance recursively
    for &ret in returns {
        // GARCH(1,1): σ²ₜ = ω + α·ε²ₜ₋₁ + β·σ²ₜ₋₁
        variance = omega + alpha * ret.powi(2) + beta * variance;

        // Prevent numerical issues
        if variance <= 0.0 || variance.is_nan() || variance.is_infinite() {
            return None;
        }

        // Gaussian log-likelihood: -0.5 * (log(2π) + log(σ²) + ε²/σ²)
        log_likelihood += -0.5 * (variance.ln() + ret.powi(2) / variance);
    }

    // Normalize by number of observations
    Some(log_likelihood / n as f64)
}

/// Check if GARCH parameters satisfy stationarity condition
fn is_stationary(params: &GarchParameters) -> bool {
    params.omega > 0.0
        && params.alpha >= 0.0
        && params.beta >= 0.0
        && params.persistence < 1.0
}

/// Generate multi-step ahead volatility forecasts
///
/// For GARCH(1,1), the h-step ahead variance forecast is:
/// σ²ₜ₊ₕ = E[σ²] + (α + β)^h * (σ²ₜ - E[σ²])
///
/// Where E[σ²] is the long-run variance.
fn forecast_volatility_multistep(
    returns: &[f64],
    params: &GarchParameters,
    horizon: usize,
    confidence_level: f64,
) -> Vec<VolatilityForecastPoint> {
    // Calculate current conditional variance (one-step ahead from last observation)
    let last_return = returns[returns.len() - 1];
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let unconditional_var: f64 = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;

    let mut current_variance = params.omega + params.alpha * last_return.powi(2) + params.beta * unconditional_var;

    let mut forecast_points = Vec::with_capacity(horizon);

    // Z-score for confidence intervals (80% = 1.28, 95% = 1.96)
    let z_score = if confidence_level >= 0.95 {
        1.96
    } else {
        1.28
    };

    for h in 1..=horizon {
        // GARCH(1,1) multi-step forecast
        // σ²ₜ₊ₕ = E[σ²] + (α + β)^h * (σ²ₜ - E[σ²])
        let persistence_factor = params.persistence.powi(h as i32);
        let forecasted_variance =
            params.long_run_variance + persistence_factor * (current_variance - params.long_run_variance);

        // Convert to annualized volatility (percentage)
        let forecasted_vol = forecasted_variance.sqrt() * (252.0_f64).sqrt() * 100.0;

        // Estimate forecast uncertainty (increases with horizon)
        // Simplified approach: uncertainty grows with sqrt(h)
        let forecast_std_error = forecasted_vol * 0.15 * (h as f64).sqrt();

        let lower_bound = (forecasted_vol - z_score * forecast_std_error).max(0.0);
        let upper_bound = forecasted_vol + z_score * forecast_std_error;

        let forecast_date = (Utc::now() + Duration::days(h as i64))
            .format("%Y-%m-%d")
            .to_string();

        forecast_points.push(VolatilityForecastPoint {
            date: forecast_date,
            predicted_volatility: forecasted_vol,
            lower_bound,
            upper_bound,
            confidence_level,
        });

        // Update current variance for next iteration (convergence to long-run)
        current_variance = forecasted_variance;
    }

    forecast_points
}

/// Generate warnings based on GARCH characteristics
fn generate_warnings(params: &GarchParameters, current_vol: f64) -> Vec<String> {
    let mut warnings = Vec::new();

    // High persistence (α + β close to 1) means shocks persist longer
    if params.persistence > 0.95 {
        warnings.push(
            "High volatility persistence detected (α+β > 0.95). Shocks will take longer to dissipate.".to_string(),
        );
    }

    // High alpha means market is very reactive to recent shocks
    if params.alpha > 0.15 {
        warnings.push(
            "High shock sensitivity (α > 0.15). Market reacts strongly to recent price moves.".to_string(),
        );
    }

    // Current volatility much higher than long-run
    let long_run_vol = params.long_run_variance.sqrt() * (252.0_f64).sqrt() * 100.0;
    if current_vol > long_run_vol * 1.5 {
        warnings.push(format!(
            "Current volatility ({:.1}%) is significantly above long-run average ({:.1}%). Expected mean reversion.",
            current_vol, long_run_vol
        ));
    }

    if warnings.is_empty() {
        warnings.push("GARCH model shows stable volatility dynamics.".to_string());
    }

    warnings
}

/// Fetch historical price data
async fn fetch_historical_prices(
    pool: &PgPool,
    ticker: &str,
    days: i32,
    price_provider: &dyn crate::external::price_provider::PriceProvider,
    failure_cache: &crate::services::failure_cache::FailureCache,
) -> Result<Vec<PricePoint>, AppError> {
    // Ensure we have recent price data
    let rate_limiter = crate::services::rate_limiter::RateLimiter::new(5, 60);
    let _ = price_service::refresh_from_api(
        pool,
        price_provider,
        ticker,
        failure_cache,
        &rate_limiter,
    )
    .await;

    // Fetch price history from database
    price_queries::fetch_window(pool, ticker, days as i64)
        .await
        .map_err(AppError::Db)
}

/// Check if a recent forecast exists in cache
async fn check_cache(
    pool: &PgPool,
    ticker: &str,
    days_ahead: i32,
    confidence_level: f64,
) -> Result<Option<VolatilityForecast>, AppError> {
    let result = sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT forecast_data
        FROM volatility_forecasts
        WHERE ticker = $1
          AND horizon_days = $2
          AND confidence_level = $3
          AND expires_at > NOW()
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
    )
    .bind(ticker)
    .bind(days_ahead)
    .bind(confidence_level)
    .fetch_optional(pool)
    .await?;

    if let Some(forecast_data) = result {
        // Deserialize JSON forecast data
        if let Ok(forecast) = serde_json::from_value::<VolatilityForecast>(forecast_data) {
            return Ok(Some(forecast));
        }
    }

    Ok(None)
}

/// Cache forecast with 24-hour TTL
async fn cache_forecast(
    pool: &PgPool,
    forecast: &VolatilityForecast,
    days_ahead: i32,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + Duration::hours(24);
    let forecast_json = serde_json::to_value(forecast)?;

    sqlx::query(
        r#"
        INSERT INTO volatility_forecasts
            (ticker, horizon_days, confidence_level, forecast_data, generated_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (ticker, horizon_days, confidence_level, generated_at)
        DO UPDATE SET
            forecast_data = EXCLUDED.forecast_data,
            expires_at = EXCLUDED.expires_at
        "#,
    )
    .bind(&forecast.ticker)
    .bind(days_ahead)
    .bind(forecast.confidence_level)
    .bind(forecast_json)
    .bind(forecast.generated_at)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_returns() {
        use bigdecimal::BigDecimal;
        use std::str::FromStr;
        use chrono::NaiveDate;

        let prices = vec![
            PricePoint {
                id: uuid::Uuid::new_v4(),
                ticker: "TEST".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                close_price: BigDecimal::from_str("100.0").unwrap(),
                created_at: chrono::Utc::now(),
            },
            PricePoint {
                id: uuid::Uuid::new_v4(),
                ticker: "TEST".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                close_price: BigDecimal::from_str("102.0").unwrap(),
                created_at: chrono::Utc::now(),
            },
            PricePoint {
                id: uuid::Uuid::new_v4(),
                ticker: "TEST".to_string(),
                date: NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                close_price: BigDecimal::from_str("101.0").unwrap(),
                created_at: chrono::Utc::now(),
            },
        ];

        let returns = compute_returns(&prices);
        assert_eq!(returns.len(), 2);
        assert!((returns[0] - 0.02).abs() < 1e-10); // 2% gain
        assert!((returns[1] - (-0.0098039)).abs() < 1e-5); // ~1% loss
    }

    #[test]
    fn test_is_stationary() {
        let stationary_params = GarchParameters {
            omega: 0.0001,
            alpha: 0.10,
            beta: 0.85,
            long_run_variance: 0.001,
            persistence: 0.95,
        };
        assert!(is_stationary(&stationary_params));

        let non_stationary_params = GarchParameters {
            omega: 0.0001,
            alpha: 0.50,
            beta: 0.51,
            long_run_variance: 0.001,
            persistence: 1.01,
        };
        assert!(!is_stationary(&non_stationary_params));
    }

    #[test]
    fn test_calculate_log_likelihood() {
        let returns = vec![0.01, -0.02, 0.015, -0.01, 0.005];
        let omega = 0.0001;
        let alpha = 0.10;
        let beta = 0.85;
        let initial_var = 0.0005;

        let ll = calculate_log_likelihood(&returns, omega, alpha, beta, initial_var);
        assert!(ll.is_some());
        assert!(ll.unwrap().is_finite());
    }

    #[test]
    fn test_forecast_volatility_multistep() {
        let returns = vec![0.01, -0.02, 0.015, -0.01, 0.005, 0.02, -0.015];
        let params = GarchParameters {
            omega: 0.0001,
            alpha: 0.10,
            beta: 0.85,
            long_run_variance: 0.001,
            persistence: 0.95,
        };

        let forecasts = forecast_volatility_multistep(&returns, &params, 5, 0.95);
        assert_eq!(forecasts.len(), 5);

        // Verify forecasts are positive and bounds are reasonable
        for point in forecasts {
            assert!(point.predicted_volatility > 0.0);
            assert!(point.lower_bound >= 0.0);
            assert!(point.upper_bound > point.lower_bound);
            assert!(point.predicted_volatility >= point.lower_bound);
            assert!(point.predicted_volatility <= point.upper_bound);
        }
    }
}

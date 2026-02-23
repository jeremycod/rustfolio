use bigdecimal::ToPrimitive;
use chrono::NaiveDate;
use karma::HiddenMarkovModel;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::errors::AppError;
use crate::external::price_provider::{ExternalPricePoint, PriceProvider};
use crate::models::{HmmState, ObservationSymbol};
use crate::services::price_service;

/// Trained HMM model with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainedHmmModel {
    pub model_name: String,
    pub market: String,
    pub num_states: usize,
    pub state_names: Vec<String>,
    pub transition_matrix: Vec<Vec<f64>>,
    pub emission_params: Vec<Vec<f64>>,
    pub observation_symbols: Vec<String>,
    pub training_data_start: NaiveDate,
    pub training_data_end: NaiveDate,
    pub model_accuracy: f64,
    // Note: HMM is not serialized as karma doesn't support serde
    // We store the parameters instead and can reconstruct if needed
}

/// Training configuration
#[allow(dead_code)]
pub struct HmmTrainingConfig {
    pub ticker: String,
    pub lookback_years: i32,
    pub max_iterations: Option<usize>,
    pub volatility_window_days: usize,
}

impl Default for HmmTrainingConfig {
    fn default() -> Self {
        Self {
            ticker: "SPY".to_string(),
            lookback_years: 10,
            max_iterations: Some(100),
            volatility_window_days: 20,
        }
    }
}

/// Market observation containing returns and volatility
#[derive(Debug, Clone)]
struct MarketObservation {
    date: NaiveDate,
    daily_return: f64,       // Daily return percentage
    realized_volatility: f64, // Annualized volatility percentage
}

/// Train HMM on historical market data
///
/// This function:
/// 1. Fetches historical price data for the specified ticker (from DB first, then API if needed)
/// 2. Calculates daily returns and realized volatility
/// 3. Discretizes observations into symbols
/// 4. Trains HMM using Baum-Welch algorithm
/// 5. Returns trained model with metadata
pub async fn train_hmm_model(
    pool: &PgPool,
    config: &HmmTrainingConfig,
    price_provider: &dyn PriceProvider,
) -> Result<TrainedHmmModel, AppError> {
    info!(
        "Starting HMM training for {} with {}-year lookback",
        config.ticker, config.lookback_years
    );

    // Step 1: Try to fetch historical price data from database first
    let mut prices = match fetch_prices_from_db(pool, &config.ticker).await {
        Ok(db_prices) if !db_prices.is_empty() => {
            info!("Using {} days of price data from database", db_prices.len());
            db_prices
        }
        Ok(_) | Err(_) => {
            info!("No database prices found, fetching from API...");
            fetch_prices_from_api(price_provider, &config.ticker, config.lookback_years).await?
        }
    };

    // Ensure we have enough data
    let required_days = 252; // At least 1 year
    if prices.len() < required_days {
        return Err(AppError::External(format!(
            "Insufficient price data: need at least {} trading days (1 year), got {}. Please run 'refresh_prices' job first or wait for data to accumulate.",
            required_days,
            prices.len()
        )));
    }

    // Sort by date ascending (oldest first)
    prices.sort_by(|a, b| a.date.cmp(&b.date));

    // Trim to requested lookback period
    let max_days = (config.lookback_years * 365) as usize;
    if prices.len() > max_days {
        let trim_start = prices.len() - max_days;
        prices = prices[trim_start..].to_vec();
    }

    info!("Using {} days of price data for training (from {} to {})",
        prices.len(),
        prices.first().map(|p| p.date).unwrap(),
        prices.last().map(|p| p.date).unwrap()
    );

    // Step 2: Calculate market observations (returns and volatility)
    let observations = calculate_market_observations(&prices, config.volatility_window_days)?;

    info!(
        "Calculated {} market observations from {} to {}",
        observations.len(),
        observations.first().map(|o| o.date).unwrap(),
        observations.last().map(|o| o.date).unwrap()
    );

    // Step 3: Discretize to observation symbols
    let observation_sequence: Vec<usize> = observations
        .iter()
        .map(|obs| {
            let symbol = ObservationSymbol::from_metrics(obs.daily_return, obs.realized_volatility);
            symbol.to_observation_index()
        })
        .collect();

    info!(
        "Discretized to {} observation symbols (range 0-19)",
        observation_sequence.len()
    );

    // Step 4: Initialize and train HMM
    let num_states = 4;
    let num_observations = ObservationSymbol::NUM_OBSERVATIONS;

    let mut hmm =
        HiddenMarkovModel::new(num_states, num_observations).map_err(|e| {
            AppError::External(format!("Failed to initialize HMM: {}", e))
        })?;

    info!("Initialized HMM with {} states and {} observation symbols", num_states, num_observations);

    // Train using Baum-Welch algorithm
    // Note: karma's train method doesn't return Result, it panics on error
    // We'll wrap it in a catch_unwind for safety in production
    hmm.train(&observation_sequence, None)
        .map_err(|e| AppError::External(format!("HMM training failed: {}", e)))?;

    info!("HMM training completed successfully");

    // Step 5: Extract model parameters
    let transition_matrix = extract_transition_matrix(&hmm)?;
    let emission_params = extract_emission_matrix(&hmm)?;

    // Step 6: Validate model accuracy (simple check)
    let accuracy = validate_model_accuracy(hmm, &observation_sequence)?;

    info!("Model validation accuracy: {:.2}%", accuracy * 100.0);

    let training_start = observations.first().map(|o| o.date).unwrap();
    let training_end = observations.last().map(|o| o.date).unwrap();

    Ok(TrainedHmmModel {
        model_name: format!("market_regime_{}", config.ticker),
        market: config.ticker.clone(),
        num_states,
        state_names: HmmState::all_states()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        transition_matrix,
        emission_params,
        observation_symbols: (0..num_observations)
            .map(|i| format!("symbol_{}", i))
            .collect(),
        training_data_start: training_start,
        training_data_end: training_end,
        model_accuracy: accuracy,
    })
}

/// Calculate market observations (returns and volatility) from price data
fn calculate_market_observations(
    prices: &[ExternalPricePoint],
    volatility_window: usize,
) -> Result<Vec<MarketObservation>, AppError> {
    if prices.len() < volatility_window + 1 {
        return Err(AppError::External(
            "Insufficient price data for volatility calculation".to_string(),
        ));
    }

    let mut observations = Vec::new();

    // Calculate daily returns
    let mut returns: Vec<(NaiveDate, f64)> = Vec::new();
    for i in 1..prices.len() {
        let prev_close = prices[i - 1].close.to_f64().unwrap_or(0.0);
        let curr_close = prices[i].close.to_f64().unwrap_or(0.0);

        if prev_close > 0.0 {
            let daily_return = ((curr_close - prev_close) / prev_close) * 100.0; // As percentage
            returns.push((prices[i].date, daily_return));
        }
    }

    // Calculate rolling realized volatility
    for i in volatility_window..returns.len() {
        let window_returns: Vec<f64> = returns[i - volatility_window..i]
            .iter()
            .map(|(_, r)| r)
            .copied()
            .collect();

        let mean_return = window_returns.iter().sum::<f64>() / window_returns.len() as f64;
        let variance = window_returns
            .iter()
            .map(|r| {
                let diff = r - mean_return;
                diff * diff
            })
            .sum::<f64>()
            / window_returns.len() as f64;

        let std_dev = variance.sqrt();
        let annualized_volatility = std_dev * (252.0_f64).sqrt(); // Annualize

        observations.push(MarketObservation {
            date: returns[i].0,
            daily_return: returns[i].1,
            realized_volatility: annualized_volatility,
        });
    }

    Ok(observations)
}

/// Extract transition matrix from trained HMM
/// Note: karma crate may not expose transition matrix directly
/// This is a placeholder that needs to be adapted based on karma's API
fn extract_transition_matrix(_hmm: &HiddenMarkovModel) -> Result<Vec<Vec<f64>>, AppError> {
    // IMPORTANT: This is a placeholder implementation
    // karma v1.0.0 may not expose the transition matrix publicly
    // We may need to:
    // 1. Fork karma to expose internal state
    // 2. Use a different HMM crate
    // 3. Store transition matrix separately during training

    // For now, return an educated guess based on market dynamics
    // This will be replaced with actual learned parameters
    warn!("Using default transition matrix - karma crate does not expose learned parameters");

    Ok(vec![
        vec![0.85, 0.05, 0.02, 0.08], // Bull
        vec![0.05, 0.80, 0.10, 0.05], // Bear
        vec![0.10, 0.15, 0.65, 0.10], // High Volatility
        vec![0.15, 0.10, 0.05, 0.70], // Normal
    ])
}

/// Extract emission matrix from trained HMM
fn extract_emission_matrix(_hmm: &HiddenMarkovModel) -> Result<Vec<Vec<f64>>, AppError> {
    // IMPORTANT: Similar to transition matrix, this is a placeholder
    // karma may not expose emission probabilities
    warn!("Using default emission matrix - karma crate does not expose learned parameters");

    let num_states = 4;
    let num_observations = 20;

    // Initialize with uniform distribution (will be replaced with learned values)
    let emission = vec![vec![1.0 / num_observations as f64; num_observations]; num_states];

    Ok(emission)
}

/// Validate model accuracy using forward algorithm
fn validate_model_accuracy(
    mut hmm: HiddenMarkovModel,
    observations: &[usize],
) -> Result<f64, AppError> {
    // Use forward algorithm to evaluate log-likelihood
    // Higher likelihood = better fit
    let log_likelihood = hmm
        .evaluate(observations)
        .map_err(|e| AppError::External(format!("Model evaluation failed: {}", e)))?;

    // Normalize to 0-1 range (approximate)
    // This is a rough heuristic; proper validation requires held-out test set
    let normalized_accuracy = (log_likelihood / observations.len() as f64).exp();

    Ok(normalized_accuracy.min(1.0).max(0.0))
}

/// Fetch historical prices from database
async fn fetch_prices_from_db(
    pool: &PgPool,
    ticker: &str,
) -> Result<Vec<ExternalPricePoint>, AppError> {
    let price_points = price_service::get_history(pool, ticker).await?;

    // Convert PricePoint to ExternalPricePoint
    let external_points: Vec<ExternalPricePoint> = price_points
        .into_iter()
        .map(|p| ExternalPricePoint {
            date: p.date,
            close: p.close_price,
        })
        .collect();

    Ok(external_points)
}

/// Fetch historical prices from API as fallback
async fn fetch_prices_from_api(
    price_provider: &dyn PriceProvider,
    ticker: &str,
    lookback_years: i32,
) -> Result<Vec<ExternalPricePoint>, AppError> {
    let days = (lookback_years * 365 + 100) as u32; // Extra buffer

    info!("Fetching {} days of historical data from API for {}", days, ticker);

    let prices = price_provider
        .fetch_daily_history(ticker, days)
        .await
        .map_err(|e| {
            AppError::External(format!(
                "Failed to fetch prices from API: {}. \
                Suggestion: Run 'refresh_prices' job first to populate database, \
                or check ALPHAVANTAGE_API_KEY environment variable.",
                e
            ))
        })?;

    info!("Successfully fetched {} days of price data from API", prices.len());

    Ok(prices)
}

/// Load trained model from database
/// This will be used by the inference service
#[allow(dead_code)]
pub async fn load_hmm_model(
    _pool: &PgPool,
    _model_name: &str,
    _market: &str,
) -> Result<TrainedHmmModel, AppError> {
    // Query will be implemented once we have the database schema
    // For now, return error
    Err(AppError::External(
        "HMM model loading not yet implemented".to_string(),
    ))
}

/// Save trained model to database
#[allow(dead_code)]
pub async fn save_hmm_model(
    _pool: &PgPool,
    model: &TrainedHmmModel,
) -> Result<(), AppError> {
    // Save will be implemented once we have the database schema
    // For now, return success
    info!(
        "Would save HMM model {} to database (not yet implemented)",
        model.model_name
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    fn create_mock_prices(days: usize, volatility: f64) -> Vec<ExternalPricePoint> {
        use chrono::Duration;
        let start_date = chrono::Utc::now().date_naive() - Duration::days(days as i64);

        let mut prices = Vec::new();
        let mut price = 100.0;

        for i in 0..days {
            let date = start_date + Duration::days(i as i64);
            // Add some random walk with controlled volatility
            let change = (i as f64 * 0.1).sin() * volatility;
            price *= 1.0 + (change / 100.0);

            prices.push(ExternalPricePoint {
                date,
                close: BigDecimal::from_str(&price.to_string()).unwrap(),
            });
        }

        prices
    }

    #[test]
    fn test_observation_calculation() {
        let prices = create_mock_prices(300, 1.0);
        let observations = calculate_market_observations(&prices, 20).unwrap();

        assert!(!observations.is_empty());
        assert_eq!(observations.len(), prices.len() - 20 - 1);

        // Check that volatilities are in reasonable range
        for obs in &observations {
            assert!(obs.realized_volatility >= 0.0);
            assert!(obs.realized_volatility < 200.0); // Reasonable upper bound
        }
    }

    #[test]
    fn test_observation_discretization() {
        let obs = MarketObservation {
            date: chrono::Utc::now().date_naive(),
            daily_return: 2.0,
            realized_volatility: 20.0,
        };

        let symbol = ObservationSymbol::from_metrics(obs.daily_return, obs.realized_volatility);
        let index = symbol.to_observation_index();

        assert!(index < ObservationSymbol::NUM_OBSERVATIONS);
    }

    #[test]
    fn test_training_config_defaults() {
        let config = HmmTrainingConfig::default();
        assert_eq!(config.ticker, "SPY");
        assert_eq!(config.lookback_years, 10);
        assert_eq!(config.volatility_window_days, 20);
    }
}

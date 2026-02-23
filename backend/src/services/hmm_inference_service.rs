use chrono::NaiveDate;
use sqlx::PgPool;
use tracing::info;

use crate::db::hmm_queries;
use crate::errors::AppError;
use crate::models::{HmmState, ObservationSymbol, RegimeForecast, StateProbabilities};

/// HMM inference service for regime detection and forecasting
///
/// This service provides three main capabilities:
/// 1. State probability distribution (Forward algorithm)
/// 2. Most likely state (Viterbi-like using Forward + argmax)
/// 3. Regime forecasting (Transition matrix multiplication)

/// Get current state probabilities using Forward algorithm
///
/// Returns probability distribution across all HMM states
/// based on recent market observations
#[allow(dead_code)]
pub async fn get_state_probabilities(
    pool: &PgPool,
    recent_observations: &[ObservationSymbol],
) -> Result<StateProbabilities, AppError> {
    // Load latest trained model
    let model_record = hmm_queries::load_latest_hmm_model(pool, "SPY").await?;

    // For now, use a simplified approach since karma doesn't expose Forward algorithm results
    // In production, we would:
    // 1. Reconstruct HMM from stored parameters
    // 2. Run Forward algorithm on recent observations
    // 3. Return state probabilities

    // Placeholder implementation: Use emission probabilities as approximation
    let state_probs = estimate_state_probabilities_from_observations(
        recent_observations,
        &model_record.emission_params.0,
    )?;

    Ok(state_probs)
}

/// Estimate state probabilities from recent observations
///
/// This is a simplified approach that uses emission probabilities
/// In full HMM, this would be the Forward algorithm result
#[allow(dead_code)]
fn estimate_state_probabilities_from_observations(
    observations: &[ObservationSymbol],
    emission_matrix: &[Vec<f64>],
) -> Result<StateProbabilities, AppError> {
    if observations.is_empty() {
        return Err(AppError::External(
            "No observations provided for state estimation".to_string(),
        ));
    }

    // Use the most recent observation
    let latest_obs = observations.last().unwrap();
    let obs_index = latest_obs.to_observation_index();

    // Get emission probabilities for this observation across all states
    let mut state_likelihoods: Vec<f64> = emission_matrix
        .iter()
        .map(|emissions| emissions.get(obs_index).copied().unwrap_or(0.0001))
        .collect();

    // Normalize to probabilities
    let sum: f64 = state_likelihoods.iter().sum();
    if sum > 0.0 {
        state_likelihoods.iter_mut().for_each(|p| *p /= sum);
    } else {
        // Fallback to uniform distribution
        let uniform = 1.0 / state_likelihoods.len() as f64;
        state_likelihoods.iter_mut().for_each(|p| *p = uniform);
    }

    // Ensure we have exactly 4 states
    if state_likelihoods.len() != 4 {
        return Err(AppError::External(format!(
            "Expected 4 states, got {}",
            state_likelihoods.len()
        )));
    }

    StateProbabilities::from_vec(state_likelihoods)
        .map_err(|e| AppError::External(format!("Invalid state probabilities: {}", e)))
}

/// Get most likely regime using HMM
///
/// Returns the regime with highest probability
#[allow(dead_code)]
pub async fn get_most_likely_regime(
    pool: &PgPool,
    recent_observations: &[ObservationSymbol],
) -> Result<HmmState, AppError> {
    let probabilities = get_state_probabilities(pool, recent_observations).await?;
    Ok(probabilities.most_likely_state())
}

/// Forecast regime N days ahead using transition matrix
///
/// Algorithm:
/// 1. Get current state distribution
/// 2. Multiply by transition matrix^N
/// 3. Return predicted distribution
#[allow(dead_code)]
pub async fn forecast_regime(
    pool: &PgPool,
    current_observations: &[ObservationSymbol],
    horizon_days: i32,
) -> Result<RegimeForecast, AppError> {
    if horizon_days < 1 || horizon_days > 30 {
        return Err(AppError::External(
            "Forecast horizon must be between 1 and 30 days".to_string(),
        ));
    }

    info!("Forecasting regime {} days ahead", horizon_days);

    // Get current state distribution
    let current_probs = get_state_probabilities(pool, current_observations).await?;

    // Load transition matrix
    let model_record = hmm_queries::load_latest_hmm_model(pool, "SPY").await?;
    let transition_matrix = &model_record.transition_matrix.0;

    // Forecast by multiplying transition matrix N times
    let forecast_probs =
        forecast_state_distribution(&current_probs, transition_matrix, horizon_days as usize)?;

    // Calculate transition probability (likelihood of regime change)
    let transition_prob = RegimeForecast::calculate_transition_prob(&forecast_probs);

    // Determine confidence level
    let confidence = RegimeForecast::calculate_confidence(&forecast_probs);

    // Get predicted regime
    let predicted_state = forecast_probs.most_likely_state();

    Ok(RegimeForecast {
        forecast_horizon_days: horizon_days,
        predicted_regime: predicted_state.to_string(),
        regime_probabilities: forecast_probs,
        transition_probability: transition_prob,
        confidence,
    })
}

/// Forecast state distribution N steps ahead
///
/// Formula: P(S_t+n) = P(S_t) × T^n
/// where T is the transition matrix
#[allow(dead_code)]
fn forecast_state_distribution(
    current_probs: &StateProbabilities,
    transition_matrix: &[Vec<f64>],
    steps: usize,
) -> Result<StateProbabilities, AppError> {
    if transition_matrix.len() != 4 {
        return Err(AppError::External(format!(
            "Expected 4x4 transition matrix, got {}x?",
            transition_matrix.len()
        )));
    }

    // Start with current probabilities
    let mut probs = current_probs.to_vec();

    // Multiply by transition matrix N times
    for _ in 0..steps {
        probs = multiply_vector_matrix(&probs, transition_matrix)?;
    }

    StateProbabilities::from_vec(probs)
        .map_err(|e| AppError::External(format!("Invalid forecast probabilities: {}", e)))
}

/// Multiply probability vector by transition matrix
///
/// Result[j] = Sum_i(P[i] * T[i][j])
#[allow(dead_code)]
fn multiply_vector_matrix(
    probs: &[f64],
    matrix: &[Vec<f64>],
) -> Result<Vec<f64>, AppError> {
    if probs.len() != matrix.len() {
        return Err(AppError::External(format!(
            "Probability vector length {} doesn't match matrix size {}",
            probs.len(),
            matrix.len()
        )));
    }

    let num_states = probs.len();
    let mut result = vec![0.0; num_states];

    for j in 0..num_states {
        for i in 0..num_states {
            let transition_prob = matrix[i]
                .get(j)
                .ok_or_else(|| {
                    AppError::External(format!("Transition matrix missing element [{},{}]", i, j))
                })?;
            result[j] += probs[i] * transition_prob;
        }
    }

    // Normalize to ensure probabilities sum to 1.0
    let sum: f64 = result.iter().sum();
    if sum > 0.0 {
        result.iter_mut().for_each(|p| *p /= sum);
    }

    Ok(result)
}

/// Update market regime with HMM data
///
/// This should be called after running HMM inference to store results
#[allow(dead_code)]
pub async fn update_regime_with_hmm_data(
    pool: &PgPool,
    date: NaiveDate,
    recent_observations: &[ObservationSymbol],
) -> Result<(), AppError> {
    // Get HMM probabilities
    let hmm_probs = get_state_probabilities(pool, recent_observations).await?;

    // Get 5-day forecast
    let forecast = forecast_regime(pool, recent_observations, 5).await?;

    // Update database
    hmm_queries::update_regime_with_hmm(
        pool,
        date,
        &hmm_probs,
        Some(&forecast.predicted_regime),
        Some(forecast.transition_probability),
    )
    .await?;

    info!(
        "Updated regime for {} with HMM data: most likely = {}, transition prob = {:.2}",
        date,
        hmm_probs.most_likely_state().to_string(),
        forecast.transition_probability
    );

    Ok(())
}

/// Generate and save regime forecasts for multiple horizons
///
/// Typically called daily to generate 5, 10, and 30 day forecasts
#[allow(dead_code)]
pub async fn generate_regime_forecasts(
    pool: &PgPool,
    forecast_date: NaiveDate,
    recent_observations: &[ObservationSymbol],
) -> Result<Vec<RegimeForecast>, AppError> {
    let horizons = vec![5, 10, 30];
    let mut forecasts = Vec::new();

    // Get latest model ID for reference
    let model_record = hmm_queries::load_latest_hmm_model(pool, "SPY").await?;

    for horizon in horizons {
        let forecast = forecast_regime(pool, recent_observations, horizon).await?;

        // Save to database
        hmm_queries::save_regime_forecast(
            pool,
            forecast_date,
            horizon,
            &forecast.predicted_regime,
            &forecast.regime_probabilities,
            forecast.transition_probability,
            &forecast.confidence,
            Some(model_record.id),
        )
        .await?;

        info!(
            "Generated {}-day forecast: {} ({} confidence)",
            horizon, forecast.predicted_regime, forecast.confidence
        );

        forecasts.push(forecast);
    }

    Ok(forecasts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_vector_matrix() {
        let probs = vec![0.7, 0.2, 0.05, 0.05];
        let matrix = vec![
            vec![0.85, 0.05, 0.02, 0.08],
            vec![0.05, 0.80, 0.10, 0.05],
            vec![0.10, 0.15, 0.65, 0.10],
            vec![0.15, 0.10, 0.05, 0.70],
        ];

        let result = multiply_vector_matrix(&probs, &matrix).unwrap();

        // Check probabilities sum to 1.0
        let sum: f64 = result.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "Probabilities should sum to 1.0");

        // Result should be a valid probability distribution
        assert_eq!(result.len(), 4);
        for p in &result {
            assert!(*p >= 0.0 && *p <= 1.0, "Each probability should be in [0,1]");
        }
    }

    #[test]
    fn test_forecast_state_distribution_convergence() {
        let initial_probs = StateProbabilities {
            bull: 1.0,
            bear: 0.0,
            high_volatility: 0.0,
            normal: 0.0,
        };

        let transition_matrix = vec![
            vec![0.85, 0.05, 0.02, 0.08],
            vec![0.05, 0.80, 0.10, 0.05],
            vec![0.10, 0.15, 0.65, 0.10],
            vec![0.15, 0.10, 0.05, 0.70],
        ];

        // Short-term forecast (5 days)
        let forecast_5 = forecast_state_distribution(&initial_probs, &transition_matrix, 5).unwrap();

        // Should still be mostly bull, but spreading out
        assert!(forecast_5.bull > 0.5);

        // Long-term forecast (100 days) should converge to stationary distribution
        let forecast_100 =
            forecast_state_distribution(&initial_probs, &transition_matrix, 100).unwrap();

        // Check it's a valid probability distribution
        let sum = forecast_100.bull
            + forecast_100.bear
            + forecast_100.high_volatility
            + forecast_100.normal;
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_state_probability_estimation() {
        let observations = vec![ObservationSymbol::from_metrics(2.0, 18.0)];

        // Simple emission matrix (4 states × 20 observations)
        let emission_matrix = vec![
            vec![0.05; 20], // Bull: uniform for simplicity
            vec![0.05; 20], // Bear
            vec![0.05; 20], // High Vol
            vec![0.05; 20], // Normal
        ];

        let result = estimate_state_probabilities_from_observations(&observations, &emission_matrix);
        assert!(result.is_ok());

        let probs = result.unwrap();
        let sum = probs.bull + probs.bear + probs.high_volatility + probs.normal;
        assert!((sum - 1.0).abs() < 0.01, "Probabilities should sum to 1.0");
    }
}

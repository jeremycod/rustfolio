use serde::{Deserialize, Serialize};

/// Hidden Markov Model state for market regime detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HmmState {
    Bull,
    Bear,
    HighVolatility,
    Normal,
}

impl HmmState {
    /// Convert HMM state to index (0-3)
    #[allow(dead_code)]
    pub fn to_index(&self) -> usize {
        match self {
            HmmState::Bull => 0,
            HmmState::Bear => 1,
            HmmState::HighVolatility => 2,
            HmmState::Normal => 3,
        }
    }

    /// Convert index to HMM state
    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(HmmState::Bull),
            1 => Some(HmmState::Bear),
            2 => Some(HmmState::HighVolatility),
            3 => Some(HmmState::Normal),
            _ => None,
        }
    }

    /// Get all possible states in order
    pub fn all_states() -> Vec<HmmState> {
        vec![
            HmmState::Bull,
            HmmState::Bear,
            HmmState::HighVolatility,
            HmmState::Normal,
        ]
    }

    /// Convert to string for database storage
    pub fn to_string(&self) -> String {
        match self {
            HmmState::Bull => "bull".to_string(),
            HmmState::Bear => "bear".to_string(),
            HmmState::HighVolatility => "high_volatility".to_string(),
            HmmState::Normal => "normal".to_string(),
        }
    }

    /// Parse from string
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bull" => Some(HmmState::Bull),
            "bear" => Some(HmmState::Bear),
            "high_volatility" => Some(HmmState::HighVolatility),
            "normal" => Some(HmmState::Normal),
            _ => None,
        }
    }
}

/// Discretized return bins
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReturnBin {
    StrongNegative, // < -2%
    Negative,       // -2% to 0%
    Flat,           // 0% to 1%
    Positive,       // 1% to 3%
    StrongPositive, // > 3%
}

impl ReturnBin {
    /// Discretize a return percentage into a bin
    pub fn from_return(return_pct: f64) -> Self {
        if return_pct < -2.0 {
            ReturnBin::StrongNegative
        } else if return_pct < 0.0 {
            ReturnBin::Negative
        } else if return_pct < 1.0 {
            ReturnBin::Flat
        } else if return_pct < 3.0 {
            ReturnBin::Positive
        } else {
            ReturnBin::StrongPositive
        }
    }

    /// Get the bin index (0-4)
    pub fn to_index(&self) -> usize {
        match self {
            ReturnBin::StrongNegative => 0,
            ReturnBin::Negative => 1,
            ReturnBin::Flat => 2,
            ReturnBin::Positive => 3,
            ReturnBin::StrongPositive => 4,
        }
    }
}

/// Discretized volatility bins
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolatilityBin {
    Low,      // < 15%
    Normal,   // 15-25%
    Elevated, // 25-35%
    High,     // > 35%
}

impl VolatilityBin {
    /// Discretize volatility percentage into a bin
    pub fn from_volatility(vol_pct: f64) -> Self {
        if vol_pct < 15.0 {
            VolatilityBin::Low
        } else if vol_pct < 25.0 {
            VolatilityBin::Normal
        } else if vol_pct < 35.0 {
            VolatilityBin::Elevated
        } else {
            VolatilityBin::High
        }
    }

    /// Get the bin index (0-3)
    pub fn to_index(&self) -> usize {
        match self {
            VolatilityBin::Low => 0,
            VolatilityBin::Normal => 1,
            VolatilityBin::Elevated => 2,
            VolatilityBin::High => 3,
        }
    }
}

/// Observation symbol combining return and volatility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationSymbol {
    pub return_bin: ReturnBin,
    pub volatility_bin: VolatilityBin,
}

impl ObservationSymbol {
    /// Create observation from return and volatility percentages
    pub fn from_metrics(return_pct: f64, volatility_pct: f64) -> Self {
        Self {
            return_bin: ReturnBin::from_return(return_pct),
            volatility_bin: VolatilityBin::from_volatility(volatility_pct),
        }
    }

    /// Convert to unique observation index (0-19)
    /// Formula: return_index * 4 + volatility_index
    pub fn to_observation_index(&self) -> usize {
        self.return_bin.to_index() * 4 + self.volatility_bin.to_index()
    }

    /// Total number of possible observation symbols
    pub const NUM_OBSERVATIONS: usize = 20; // 5 return bins × 4 volatility bins
}

/// HMM state probabilities at a given time
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProbabilities {
    pub bull: f64,
    pub bear: f64,
    pub high_volatility: f64,
    pub normal: f64,
}

impl StateProbabilities {
    /// Create from a vector of probabilities (must be length 4)
    #[allow(dead_code)]
    pub fn from_vec(probs: Vec<f64>) -> Result<Self, String> {
        if probs.len() != 4 {
            return Err(format!("Expected 4 probabilities, got {}", probs.len()));
        }

        // Validate probabilities sum to approximately 1.0
        let sum: f64 = probs.iter().sum();
        if (sum - 1.0).abs() > 0.01 {
            return Err(format!("Probabilities must sum to 1.0, got {}", sum));
        }

        Ok(Self {
            bull: probs[0],
            bear: probs[1],
            high_volatility: probs[2],
            normal: probs[3],
        })
    }

    /// Convert to vector for matrix operations
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.bull, self.bear, self.high_volatility, self.normal]
    }

    /// Get the most likely state
    #[allow(dead_code)]
    pub fn most_likely_state(&self) -> HmmState {
        let probs = [
            (HmmState::Bull, self.bull),
            (HmmState::Bear, self.bear),
            (HmmState::HighVolatility, self.high_volatility),
            (HmmState::Normal, self.normal),
        ];

        probs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(state, _)| state.clone())
            .unwrap_or(HmmState::Normal)
    }

    /// Get probability for a specific state
    #[allow(dead_code)]
    pub fn get_probability(&self, state: &HmmState) -> f64 {
        match state {
            HmmState::Bull => self.bull,
            HmmState::Bear => self.bear,
            HmmState::HighVolatility => self.high_volatility,
            HmmState::Normal => self.normal,
        }
    }

    /// Get confidence (maximum probability)
    #[allow(dead_code)]
    pub fn confidence(&self) -> f64 {
        self.to_vec()
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

/// Regime forecast for N days ahead
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeForecast {
    pub forecast_horizon_days: i32,
    pub predicted_regime: String,
    pub regime_probabilities: StateProbabilities,
    pub transition_probability: f64,
    pub confidence: String, // "high", "medium", "low"
}

impl RegimeForecast {
    /// Determine confidence level based on probability distribution
    #[allow(dead_code)]
    pub fn calculate_confidence(probs: &StateProbabilities) -> String {
        let max_prob = probs.confidence();
        if max_prob > 0.7 {
            "high".to_string()
        } else if max_prob > 0.5 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Calculate probability of regime change
    /// (1 - probability of staying in most likely state)
    #[allow(dead_code)]
    pub fn calculate_transition_prob(current_probs: &StateProbabilities) -> f64 {
        let max_prob = current_probs.confidence();
        1.0 - max_prob
    }
}

/// Enhanced market regime response with HMM data
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedRegimeResponse {
    pub regime_type: String,
    pub confidence: f64,
    pub volatility_based: String,
    pub hmm_most_likely: String,
    pub hmm_probabilities: StateProbabilities,
    pub ensemble_confidence: f64,
}

/// Query parameters for regime forecast endpoint
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RegimeForecastParams {
    #[serde(default = "default_forecast_days")]
    pub days: i32,
}

#[allow(dead_code)]
fn default_forecast_days() -> i32 {
    5
}

impl RegimeForecastParams {
    /// Validate and clamp forecast days
    #[allow(dead_code)]
    pub fn validated(self) -> Self {
        Self {
            days: self.days.clamp(1, 30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmm_state_conversion() {
        assert_eq!(HmmState::Bull.to_index(), 0);
        assert_eq!(HmmState::from_index(0), Some(HmmState::Bull));
        assert_eq!(HmmState::from_index(4), None);
    }

    #[test]
    fn test_return_binning() {
        assert_eq!(ReturnBin::from_return(-3.0), ReturnBin::StrongNegative);
        assert_eq!(ReturnBin::from_return(-1.0), ReturnBin::Negative);
        assert_eq!(ReturnBin::from_return(0.5), ReturnBin::Flat);
        assert_eq!(ReturnBin::from_return(2.0), ReturnBin::Positive);
        assert_eq!(ReturnBin::from_return(5.0), ReturnBin::StrongPositive);
    }

    #[test]
    fn test_volatility_binning() {
        assert_eq!(VolatilityBin::from_volatility(10.0), VolatilityBin::Low);
        assert_eq!(VolatilityBin::from_volatility(20.0), VolatilityBin::Normal);
        assert_eq!(
            VolatilityBin::from_volatility(30.0),
            VolatilityBin::Elevated
        );
        assert_eq!(VolatilityBin::from_volatility(40.0), VolatilityBin::High);
    }

    #[test]
    fn test_observation_symbol() {
        let obs = ObservationSymbol::from_metrics(2.0, 20.0);
        assert_eq!(obs.return_bin, ReturnBin::Positive);
        assert_eq!(obs.volatility_bin, VolatilityBin::Normal);
        assert_eq!(obs.to_observation_index(), 13); // 3 * 4 + 1
    }

    #[test]
    fn test_observation_index_range() {
        // Test all combinations produce valid indices
        for ret in 0..5 {
            for vol in 0..4 {
                let index = ret * 4 + vol;
                assert!(index < ObservationSymbol::NUM_OBSERVATIONS);
            }
        }
    }

    #[test]
    fn test_state_probabilities() {
        let probs = StateProbabilities::from_vec(vec![0.6, 0.2, 0.1, 0.1]).unwrap();
        assert_eq!(probs.bull, 0.6);
        assert_eq!(probs.most_likely_state(), HmmState::Bull);
        assert_eq!(probs.confidence(), 0.6);
    }

    #[test]
    fn test_state_probabilities_invalid() {
        // Sum not equal to 1.0
        let result = StateProbabilities::from_vec(vec![0.5, 0.2, 0.1, 0.1]);
        assert!(result.is_err());

        // Wrong length
        let result = StateProbabilities::from_vec(vec![0.5, 0.5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_confidence_levels() {
        let high_conf = StateProbabilities::from_vec(vec![0.8, 0.1, 0.05, 0.05]).unwrap();
        assert_eq!(RegimeForecast::calculate_confidence(&high_conf), "high");

        let med_conf = StateProbabilities::from_vec(vec![0.6, 0.2, 0.1, 0.1]).unwrap();
        assert_eq!(RegimeForecast::calculate_confidence(&med_conf), "medium");

        let low_conf = StateProbabilities::from_vec(vec![0.4, 0.3, 0.2, 0.1]).unwrap();
        assert_eq!(RegimeForecast::calculate_confidence(&low_conf), "low");
    }

    #[test]
    fn test_forecast_params_validation() {
        let params = RegimeForecastParams { days: 50 }.validated();
        assert_eq!(params.days, 30); // Clamped to max

        let params = RegimeForecastParams { days: -5 }.validated();
        assert_eq!(params.days, 1); // Clamped to min

        let params = RegimeForecastParams { days: 10 }.validated();
        assert_eq!(params.days, 10); // Within range
    }
}

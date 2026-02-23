-- Add HMM support to market regimes
-- Phase 2: Enhanced Regime Detection with Hidden Markov Models

-- ============================================================================
-- Part 1: Extend market_regimes table with HMM probability distributions
-- ============================================================================

-- Add HMM probability distribution across all states
ALTER TABLE market_regimes
ADD COLUMN IF NOT EXISTS hmm_probabilities JSONB;

-- Add HMM prediction for next regime
ALTER TABLE market_regimes
ADD COLUMN IF NOT EXISTS predicted_regime VARCHAR(50);

-- Add transition probability (likelihood of regime change)
ALTER TABLE market_regimes
ADD COLUMN IF NOT EXISTS transition_probability NUMERIC(5, 4);

-- Add index for querying by HMM predictions
CREATE INDEX IF NOT EXISTS idx_market_regimes_predicted
ON market_regimes(predicted_regime)
WHERE predicted_regime IS NOT NULL;

-- Comment on new columns
COMMENT ON COLUMN market_regimes.hmm_probabilities IS
'Probability distribution across HMM states: {"bull": 0.65, "bear": 0.15, "high_volatility": 0.05, "normal": 0.15}';

COMMENT ON COLUMN market_regimes.predicted_regime IS
'HMM predicted regime for next 5-10 days';

COMMENT ON COLUMN market_regimes.transition_probability IS
'Probability of regime change in near future (0.0-1.0)';

-- ============================================================================
-- Part 2: Create hmm_models table for storing trained HMM parameters
-- ============================================================================

CREATE TABLE IF NOT EXISTS hmm_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Model identification
    model_name VARCHAR(100) NOT NULL,
    market VARCHAR(20) NOT NULL,

    -- Model structure
    num_states INTEGER NOT NULL CHECK (num_states > 0),
    state_names JSONB NOT NULL,
    -- Example: ["Bull", "Bear", "HighVolatility", "Normal"]

    -- Model parameters (learned via Baum-Welch)
    transition_matrix JSONB NOT NULL,
    -- Example: [[0.85, 0.05, 0.02, 0.08], [0.05, 0.80, 0.10, 0.05], ...]
    -- 4x4 matrix for state transitions

    emission_params JSONB NOT NULL,
    -- Example: [[0.45, 0.30, ...], [...], ...]
    -- NxM matrix for observation probabilities

    observation_symbols JSONB NOT NULL,
    -- Example: ["symbol_0", "symbol_1", ..., "symbol_19"]
    -- 20 symbols for discretized returns × volatility

    -- Training metadata
    trained_on_date DATE NOT NULL,
    training_data_start DATE NOT NULL,
    training_data_end DATE NOT NULL,
    model_accuracy NUMERIC(5, 4),
    -- Validation accuracy score (0.0-1.0)

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one model per market per training date
    UNIQUE(model_name, market, trained_on_date)
);

-- Indexes for efficient querying
CREATE INDEX idx_hmm_models_market
ON hmm_models(market, trained_on_date DESC);

CREATE INDEX idx_hmm_models_name
ON hmm_models(model_name, trained_on_date DESC);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_hmm_models_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hmm_models_updated_at
    BEFORE UPDATE ON hmm_models
    FOR EACH ROW
    EXECUTE FUNCTION update_hmm_models_updated_at();

-- Comment on table
COMMENT ON TABLE hmm_models IS
'Stores trained Hidden Markov Models for market regime detection. Models are retrained monthly with rolling historical data.';

COMMENT ON COLUMN hmm_models.transition_matrix IS
'State transition probabilities P(state_j | state_i) as NxN matrix';

COMMENT ON COLUMN hmm_models.emission_params IS
'Observation emission probabilities P(observation_k | state_i) as NxM matrix';

COMMENT ON COLUMN hmm_models.model_accuracy IS
'Validation accuracy on held-out test set (0.0 to 1.0)';

-- ============================================================================
-- Part 3: Create regime forecasts table for storing predictions
-- ============================================================================

CREATE TABLE IF NOT EXISTS regime_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Forecast identification
    forecast_date DATE NOT NULL,
    horizon_days INTEGER NOT NULL CHECK (horizon_days > 0 AND horizon_days <= 30),

    -- Predicted regime
    predicted_regime VARCHAR(50) NOT NULL,

    -- Probability distribution
    regime_probabilities JSONB NOT NULL,
    -- Example: {"bull": 0.55, "bear": 0.18, "high_volatility": 0.10, "normal": 0.17}

    -- Confidence metrics
    transition_probability NUMERIC(5, 4) NOT NULL,
    confidence_level VARCHAR(20) NOT NULL CHECK (confidence_level IN ('high', 'medium', 'low')),

    -- Model used for forecast
    hmm_model_id UUID REFERENCES hmm_models(id) ON DELETE SET NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One forecast per date per horizon
    UNIQUE(forecast_date, horizon_days)
);

-- Indexes for efficient querying
CREATE INDEX idx_regime_forecasts_date
ON regime_forecasts(forecast_date DESC);

CREATE INDEX idx_regime_forecasts_horizon
ON regime_forecasts(horizon_days, forecast_date DESC);

CREATE INDEX idx_regime_forecasts_model
ON regime_forecasts(hmm_model_id, forecast_date DESC);

-- Comment on table
COMMENT ON TABLE regime_forecasts IS
'Stores HMM-based regime forecasts for various time horizons (5, 10, 30 days)';

COMMENT ON COLUMN regime_forecasts.horizon_days IS
'Number of days ahead for the forecast (typically 5, 10, or 30)';

COMMENT ON COLUMN regime_forecasts.transition_probability IS
'Probability of regime change within forecast horizon';

-- ============================================================================
-- Part 4: Create job schedule for monthly HMM retraining
-- ============================================================================

-- Insert HMM retraining job into job config table
INSERT INTO job_config (
    job_name,
    schedule,
    enabled,
    max_duration_minutes
) VALUES (
    'train_hmm_model',
    '0 0 0 * * SUN',  -- At midnight on Sunday (weekly training)
    true,
    45  -- 45 minutes max duration
) ON CONFLICT (job_name) DO UPDATE SET
    schedule = EXCLUDED.schedule,
    enabled = EXCLUDED.enabled,
    max_duration_minutes = EXCLUDED.max_duration_minutes;

COMMENT ON TABLE hmm_models IS
'HMM models are retrained monthly via job_schedules to adapt to evolving market conditions';

-- ============================================================================
-- Part 5: Add validation constraints and helper functions
-- ============================================================================

-- Function to validate JSONB probability distribution
CREATE OR REPLACE FUNCTION validate_probability_distribution(probs JSONB)
RETURNS BOOLEAN AS $$
DECLARE
    prob_sum NUMERIC;
BEGIN
    -- Check that probabilities sum to approximately 1.0
    SELECT SUM((value)::NUMERIC) INTO prob_sum
    FROM jsonb_each_text(probs);

    RETURN (prob_sum >= 0.95 AND prob_sum <= 1.05);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Add check constraint for probability distributions
ALTER TABLE market_regimes
ADD CONSTRAINT check_hmm_probabilities_valid
CHECK (hmm_probabilities IS NULL OR validate_probability_distribution(hmm_probabilities));

ALTER TABLE regime_forecasts
ADD CONSTRAINT check_regime_probabilities_valid
CHECK (validate_probability_distribution(regime_probabilities));

-- ============================================================================
-- Part 6: Sample data and examples
-- ============================================================================

-- Example HMM probabilities for documentation
COMMENT ON CONSTRAINT check_hmm_probabilities_valid ON market_regimes IS
'Ensures HMM probabilities sum to 1.0. Example: {"bull": 0.65, "bear": 0.15, "high_volatility": 0.05, "normal": 0.15}';

-- ============================================================================
-- Part 7: Views for easy querying
-- ============================================================================

-- View for current regime with HMM data
CREATE OR REPLACE VIEW current_regime_enhanced AS
SELECT
    mr.id,
    mr.date,
    mr.regime_type AS volatility_based_regime,
    mr.volatility_level,
    mr.market_return,
    mr.confidence AS rule_based_confidence,
    mr.hmm_probabilities,
    mr.predicted_regime,
    mr.transition_probability,
    mr.benchmark_ticker,
    mr.threshold_multiplier,
    CASE
        WHEN mr.hmm_probabilities IS NOT NULL THEN
            (SELECT key
             FROM jsonb_each_text(mr.hmm_probabilities)
             ORDER BY value::NUMERIC DESC
             LIMIT 1)
        ELSE NULL
    END AS hmm_most_likely_regime,
    CASE
        WHEN mr.hmm_probabilities IS NOT NULL THEN
            (SELECT MAX(value::NUMERIC) FROM jsonb_each_text(mr.hmm_probabilities))
        ELSE NULL
    END AS hmm_confidence,
    mr.created_at,
    mr.updated_at
FROM market_regimes mr
ORDER BY mr.date DESC
LIMIT 1;

COMMENT ON VIEW current_regime_enhanced IS
'Current market regime with both rule-based and HMM detection results';

-- View for recent forecasts
CREATE OR REPLACE VIEW recent_regime_forecasts AS
SELECT
    rf.forecast_date,
    rf.horizon_days,
    rf.predicted_regime,
    rf.regime_probabilities,
    rf.transition_probability,
    rf.confidence_level,
    hm.model_name,
    hm.trained_on_date,
    hm.model_accuracy,
    rf.created_at
FROM regime_forecasts rf
LEFT JOIN hmm_models hm ON rf.hmm_model_id = hm.id
ORDER BY rf.forecast_date DESC, rf.horizon_days ASC
LIMIT 100;

COMMENT ON VIEW recent_regime_forecasts IS
'Recent regime forecasts with model metadata for monitoring and analysis';

-- ============================================================================
-- Migration complete
-- ============================================================================

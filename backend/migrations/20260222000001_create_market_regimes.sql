-- Create market_regimes table
-- Stores historical market regime classifications for adaptive risk thresholds
--
-- Regime Types:
-- - 'bull': Positive returns, low volatility (< 20%)
-- - 'bear': Negative returns, high volatility (> 25%)
-- - 'high_volatility': Extreme volatility (> 35%)
-- - 'normal': Default market conditions

CREATE TABLE market_regimes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Date of the regime detection (typically end of trading day)
    date DATE NOT NULL UNIQUE,

    -- Regime classification
    regime_type VARCHAR(20) NOT NULL CHECK (regime_type IN ('bull', 'bear', 'high_volatility', 'normal')),

    -- Underlying market metrics (annualized volatility %)
    volatility_level NUMERIC(6,3) NOT NULL,

    -- Market return over analysis period (%)
    market_return NUMERIC(7,3),

    -- Confidence score (0-100) in regime classification
    confidence NUMERIC(5,2) NOT NULL DEFAULT 100.0,

    -- Benchmark ticker used for detection (typically SPY)
    benchmark_ticker VARCHAR(10) NOT NULL DEFAULT 'SPY',

    -- Lookback period in days used for calculation
    lookback_days INTEGER NOT NULL DEFAULT 30,

    -- Adaptive threshold multipliers based on regime
    -- These are automatically applied to risk thresholds
    threshold_multiplier NUMERIC(4,2) NOT NULL DEFAULT 1.0,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast date-based lookups (most recent regime)
CREATE INDEX idx_market_regimes_date ON market_regimes(date DESC);

-- Index for regime type queries
CREATE INDEX idx_market_regimes_type ON market_regimes(regime_type);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_market_regimes_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER market_regimes_updated_at
    BEFORE UPDATE ON market_regimes
    FOR EACH ROW
    EXECUTE FUNCTION update_market_regimes_updated_at();

-- Insert historical default regime (normal market)
INSERT INTO market_regimes (date, regime_type, volatility_level, market_return, confidence, threshold_multiplier)
VALUES (CURRENT_DATE, 'normal', 18.0, 0.0, 100.0, 1.0)
ON CONFLICT (date) DO NOTHING;

-- Comment on table and important columns
COMMENT ON TABLE market_regimes IS 'Historical market regime classifications for adaptive risk management';
COMMENT ON COLUMN market_regimes.regime_type IS 'bull, bear, high_volatility, or normal market classification';
COMMENT ON COLUMN market_regimes.volatility_level IS 'Annualized volatility percentage of benchmark';
COMMENT ON COLUMN market_regimes.threshold_multiplier IS 'Multiplier applied to risk thresholds (< 1.0 = stricter, > 1.0 = looser)';

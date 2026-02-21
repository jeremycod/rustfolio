-- Create portfolio optimization cache table
-- This table stores pre-calculated optimization recommendations for portfolios

CREATE TABLE portfolio_optimization_cache (
    portfolio_id UUID NOT NULL PRIMARY KEY REFERENCES portfolios(id) ON DELETE CASCADE,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    -- Optimization results (stored as JSONB)
    recommendations JSONB NOT NULL,
    risk_free_rate DOUBLE PRECISION NOT NULL,

    -- Metadata
    positions_analyzed INTEGER NOT NULL,
    calculation_duration_ms INTEGER,

    CONSTRAINT valid_recommendations CHECK (jsonb_typeof(recommendations) = 'array')
);

-- Indexes
CREATE INDEX idx_optimization_cache_expires_at ON portfolio_optimization_cache(expires_at);
CREATE INDEX idx_optimization_cache_calculated_at ON portfolio_optimization_cache(calculated_at DESC);

-- Comment
COMMENT ON TABLE portfolio_optimization_cache IS 'Caches portfolio optimization recommendations to avoid expensive real-time calculations';

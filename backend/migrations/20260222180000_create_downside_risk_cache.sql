-- Downside risk cache for portfolio tail-risk metrics
-- Stores pre-computed downside deviation, Sortino ratios, and CVaR metrics

CREATE TABLE downside_risk_cache (
    portfolio_id UUID NOT NULL,
    days INTEGER NOT NULL,
    benchmark VARCHAR(10) NOT NULL,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    -- Cached analysis data as JSONB
    risk_data JSONB NOT NULL,

    PRIMARY KEY (portfolio_id, days, benchmark)
);

-- Index for efficient cleanup of expired entries
CREATE INDEX idx_downside_risk_cache_expires_at ON downside_risk_cache(expires_at);

-- Index for portfolio lookups
CREATE INDEX idx_downside_risk_cache_portfolio ON downside_risk_cache(portfolio_id);

-- Comments for documentation
COMMENT ON TABLE downside_risk_cache IS 'Cache for downside risk analysis including CVaR, Sortino ratio, and tail risk metrics. TTL is 6 hours.';
COMMENT ON COLUMN downside_risk_cache.portfolio_id IS 'Portfolio UUID';
COMMENT ON COLUMN downside_risk_cache.days IS 'Analysis window in days (e.g., 90, 180, 365)';
COMMENT ON COLUMN downside_risk_cache.benchmark IS 'Benchmark ticker for MAR calculation';
COMMENT ON COLUMN downside_risk_cache.risk_data IS 'Complete downside risk analysis as JSONB';

-- Create portfolio_risk_cache table
CREATE TABLE IF NOT EXISTS portfolio_risk_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    days INTEGER NOT NULL,
    benchmark VARCHAR(10) NOT NULL,
    risk_data JSONB NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(portfolio_id, days, benchmark)
);

-- Create index on portfolio_id for faster lookups
CREATE INDEX idx_portfolio_risk_cache_portfolio_id ON portfolio_risk_cache(portfolio_id);

-- Create index on expires_at for cleanup queries
CREATE INDEX idx_portfolio_risk_cache_expires_at ON portfolio_risk_cache(expires_at);

-- Add comment
COMMENT ON TABLE portfolio_risk_cache IS 'Caches portfolio risk calculations to avoid expensive recalculations and API calls';
COMMENT ON COLUMN portfolio_risk_cache.risk_data IS 'Serialized PortfolioRiskWithViolations JSON';
COMMENT ON COLUMN portfolio_risk_cache.expires_at IS 'Timestamp when this cache entry should be considered stale (4 hours from calculated_at)';

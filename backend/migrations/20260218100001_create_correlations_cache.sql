-- Portfolio Correlations Cache Table
-- Stores pre-computed correlation matrices for portfolio holdings to avoid expensive recalculations
-- Correlation calculations require fetching and processing historical price data across multiple tickers
-- This cache significantly improves performance for portfolio analytics and risk assessment

CREATE TABLE IF NOT EXISTS portfolio_correlations_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    days INTEGER NOT NULL,
    correlations_data JSONB NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    calculation_status VARCHAR(20) DEFAULT 'fresh',
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(portfolio_id, days)
);

-- Index on portfolio_id for efficient portfolio-based lookups
CREATE INDEX idx_portfolio_correlations_cache_portfolio_id ON portfolio_correlations_cache(portfolio_id);

-- Index on expires_at for efficient cache expiration cleanup queries
CREATE INDEX idx_portfolio_correlations_cache_expires_at ON portfolio_correlations_cache(expires_at);

-- Index on calculation_status for monitoring cache health and identifying stale entries
CREATE INDEX idx_portfolio_correlations_cache_status ON portfolio_correlations_cache(calculation_status);

-- Table and column comments for documentation
COMMENT ON TABLE portfolio_correlations_cache IS 'Caches portfolio correlation matrices to avoid expensive recalculations. TTL typically 24 hours since correlations do not change intraday.';
COMMENT ON COLUMN portfolio_correlations_cache.portfolio_id IS 'Foreign key to portfolios table - cascades deletion when portfolio is removed';
COMMENT ON COLUMN portfolio_correlations_cache.days IS 'Number of days of historical data used for correlation calculation (e.g., 30, 60, 90, 365)';
COMMENT ON COLUMN portfolio_correlations_cache.correlations_data IS 'JSONB containing the correlation matrix, ticker pairs, and related metadata';
COMMENT ON COLUMN portfolio_correlations_cache.calculated_at IS 'Timestamp when the correlation matrix was calculated';
COMMENT ON COLUMN portfolio_correlations_cache.expires_at IS 'Timestamp when this cache entry should be considered stale and eligible for recalculation';
COMMENT ON COLUMN portfolio_correlations_cache.calculation_status IS 'Status indicator: fresh, stale, error, or calculating';
COMMENT ON COLUMN portfolio_correlations_cache.last_error IS 'Error message if calculation failed, NULL if successful';

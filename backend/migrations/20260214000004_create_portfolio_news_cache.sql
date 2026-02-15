-- Create portfolio_news_cache table
CREATE TABLE IF NOT EXISTS portfolio_news_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    news_data JSONB NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(portfolio_id)
);

-- Create index on portfolio_id for faster lookups
CREATE INDEX idx_portfolio_news_cache_portfolio_id ON portfolio_news_cache(portfolio_id);

-- Create index on expires_at for cleanup queries
CREATE INDEX idx_portfolio_news_cache_expires_at ON portfolio_news_cache(expires_at);

-- Add comment
COMMENT ON TABLE portfolio_news_cache IS 'Caches portfolio news analysis to avoid unnecessary API calls';
COMMENT ON COLUMN portfolio_news_cache.news_data IS 'Serialized PortfolioNewsAnalysis JSON';
COMMENT ON COLUMN portfolio_news_cache.expires_at IS 'Timestamp when this cache entry should be considered stale (24 hours from fetched_at)';

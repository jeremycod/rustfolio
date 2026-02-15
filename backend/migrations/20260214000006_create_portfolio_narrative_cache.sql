-- Create portfolio_narrative_cache table
CREATE TABLE IF NOT EXISTS portfolio_narrative_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    time_period VARCHAR(10) NOT NULL,
    narrative_data JSONB NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(portfolio_id, time_period)
);

-- Create index on portfolio_id for faster lookups
CREATE INDEX idx_portfolio_narrative_cache_portfolio_id ON portfolio_narrative_cache(portfolio_id);

-- Create index on expires_at for cleanup queries
CREATE INDEX idx_portfolio_narrative_cache_expires_at ON portfolio_narrative_cache(expires_at);

-- Add narrative_cache_hours to user_preferences table (default 24 hours)
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS narrative_cache_hours INTEGER NOT NULL DEFAULT 24;

-- Add comment
COMMENT ON TABLE portfolio_narrative_cache IS 'Caches AI-generated portfolio narratives to avoid unnecessary LLM API calls';
COMMENT ON COLUMN portfolio_narrative_cache.narrative_data IS 'Serialized PortfolioNarrative JSON';
COMMENT ON COLUMN portfolio_narrative_cache.expires_at IS 'Timestamp when this cache entry should be considered stale (configurable, default 24 hours)';
COMMENT ON COLUMN user_preferences.narrative_cache_hours IS 'How many hours to cache portfolio narratives before requiring refresh (default: 24)';

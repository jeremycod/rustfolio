-- Long-term investment guidance cache
-- Caches quality scores, recommendations, and allocation strategies
-- TTL: 1 hour (configurable at insert time)

CREATE TABLE IF NOT EXISTS long_term_guidance_cache (
    id UUID PRIMARY KEY,
    portfolio_id UUID NOT NULL,
    goal VARCHAR(20) NOT NULL,             -- 'retirement', 'college', 'wealth'
    horizon_years INTEGER NOT NULL,
    risk_tolerance VARCHAR(20) NOT NULL,    -- 'conservative', 'moderate', 'aggressive'
    guidance_data JSONB NOT NULL,           -- Full LongTermGuidanceResponse
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    -- Only one active cache entry per parameter combination
    CONSTRAINT uq_long_term_guidance_params
        UNIQUE (portfolio_id, goal, horizon_years, risk_tolerance)
);

-- Index for cache lookups
CREATE INDEX IF NOT EXISTS idx_ltg_cache_lookup
    ON long_term_guidance_cache (portfolio_id, goal, horizon_years, risk_tolerance, expires_at);

-- Index for cleanup of expired entries
CREATE INDEX IF NOT EXISTS idx_ltg_cache_expiry
    ON long_term_guidance_cache (expires_at);

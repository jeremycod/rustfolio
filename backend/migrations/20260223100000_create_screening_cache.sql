-- Screening cache table for caching multi-factor screening results (15-minute TTL)
CREATE TABLE IF NOT EXISTS screening_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_key VARCHAR(128) NOT NULL UNIQUE,
    results_json JSONB NOT NULL,
    total_screened INTEGER NOT NULL DEFAULT 0,
    total_passed_filters INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '15 minutes')
);

CREATE INDEX IF NOT EXISTS idx_screening_cache_key ON screening_cache(cache_key);
CREATE INDEX IF NOT EXISTS idx_screening_cache_expiry ON screening_cache(expires_at);

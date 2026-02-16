-- Rolling beta cache for tracking beta changes over time
-- Stores pre-computed rolling beta analysis to avoid expensive recalculations

CREATE TABLE rolling_beta_cache (
    ticker VARCHAR(10) NOT NULL,
    benchmark VARCHAR(10) NOT NULL,
    total_days INTEGER NOT NULL,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    -- Cached beta time series as JSONB for efficient storage
    beta_30d JSONB NOT NULL,
    beta_60d JSONB NOT NULL,
    beta_90d JSONB NOT NULL,

    -- Summary statistics
    current_beta DOUBLE PRECISION NOT NULL,
    beta_volatility DOUBLE PRECISION NOT NULL,

    PRIMARY KEY (ticker, benchmark, total_days)
);

-- Index for efficient cleanup of expired entries
CREATE INDEX idx_rolling_beta_cache_expires_at ON rolling_beta_cache(expires_at);

-- Index for looking up by ticker
CREATE INDEX idx_rolling_beta_cache_ticker ON rolling_beta_cache(ticker);

-- Comments for documentation
COMMENT ON TABLE rolling_beta_cache IS 'Cache for rolling beta analysis to avoid expensive recalculations. TTL is 24 hours since beta does not change intraday.';
COMMENT ON COLUMN rolling_beta_cache.ticker IS 'Ticker symbol being analyzed';
COMMENT ON COLUMN rolling_beta_cache.benchmark IS 'Benchmark ticker (e.g., SPY, QQQ, IWM)';
COMMENT ON COLUMN rolling_beta_cache.total_days IS 'Total number of days in the analysis period';
COMMENT ON COLUMN rolling_beta_cache.beta_30d IS 'JSONB array of 30-day rolling beta points';
COMMENT ON COLUMN rolling_beta_cache.beta_60d IS 'JSONB array of 60-day rolling beta points';
COMMENT ON COLUMN rolling_beta_cache.beta_90d IS 'JSONB array of 90-day rolling beta points';
COMMENT ON COLUMN rolling_beta_cache.current_beta IS 'Most recent beta value from 90d window';
COMMENT ON COLUMN rolling_beta_cache.beta_volatility IS 'Standard deviation of 90d beta values';

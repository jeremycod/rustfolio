-- Beta Forecast Cache Table
-- Stores cached beta forecasts for positions with 24-hour TTL
-- Keyed by (ticker, benchmark, days_ahead, method)

CREATE TABLE beta_forecast_cache (
    ticker VARCHAR(10) NOT NULL,
    benchmark VARCHAR(10) NOT NULL,
    days_ahead INTEGER NOT NULL,
    method VARCHAR(30) NOT NULL,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    -- Cached forecast data as JSONB
    forecast_points JSONB NOT NULL,
    regime_changes JSONB NOT NULL,
    warnings JSONB NOT NULL,

    -- Summary statistics
    current_beta DOUBLE PRECISION NOT NULL,
    beta_volatility DOUBLE PRECISION NOT NULL,

    PRIMARY KEY (ticker, benchmark, days_ahead, method)
);

-- Index for cache expiration cleanup
CREATE INDEX idx_beta_forecast_cache_expires_at ON beta_forecast_cache(expires_at);

-- Index for ticker-based lookups
CREATE INDEX idx_beta_forecast_cache_ticker ON beta_forecast_cache(ticker);

-- Comments for documentation
COMMENT ON TABLE beta_forecast_cache IS 'Caches beta forecasts with 24-hour TTL to improve performance';
COMMENT ON COLUMN beta_forecast_cache.forecast_points IS 'Array of {date, predicted_beta, lower_bound, upper_bound, confidence_level}';
COMMENT ON COLUMN beta_forecast_cache.regime_changes IS 'Array of detected regime changes with z-scores';
COMMENT ON COLUMN beta_forecast_cache.warnings IS 'Array of warning messages for low confidence, high volatility, etc.';

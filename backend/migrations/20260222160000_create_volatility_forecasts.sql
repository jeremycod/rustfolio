-- Create volatility_forecasts table for GARCH volatility forecasting
-- This table caches volatility forecasts to avoid recomputing expensive GARCH models

CREATE TABLE IF NOT EXISTS volatility_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticker VARCHAR(10) NOT NULL,
    horizon_days INTEGER NOT NULL, -- forecast horizon (1-90 days)
    confidence_level NUMERIC(3, 2) NOT NULL, -- 0.80 or 0.95
    forecast_data JSONB NOT NULL, -- complete VolatilityForecast struct
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL, -- cache expiry (24 hours)

    -- Denormalized fields for querying
    current_volatility NUMERIC(10, 4), -- annualized current volatility (%)
    avg_forecasted_volatility NUMERIC(10, 4), -- average forecasted volatility
    garch_omega NUMERIC(15, 10), -- GARCH parameter ω
    garch_alpha NUMERIC(10, 8), -- GARCH parameter α
    garch_beta NUMERIC(10, 8), -- GARCH parameter β
    persistence NUMERIC(10, 8), -- α + β (volatility persistence)

    UNIQUE(ticker, horizon_days, confidence_level, generated_at)
);

-- Index for cache lookups (most common query)
CREATE INDEX idx_volatility_forecasts_cache_lookup
ON volatility_forecasts(ticker, horizon_days, confidence_level, expires_at);

-- Index for expiry-based cleanup
CREATE INDEX idx_volatility_forecasts_expires
ON volatility_forecasts(expires_at);

-- Index for analytics queries (finding high volatility stocks)
CREATE INDEX idx_volatility_forecasts_high_vol
ON volatility_forecasts(avg_forecasted_volatility DESC, generated_at DESC);

-- Index for GARCH parameter analysis
CREATE INDEX idx_volatility_forecasts_persistence
ON volatility_forecasts(persistence DESC, ticker);

-- Comments for documentation
COMMENT ON TABLE volatility_forecasts IS 'Cached volatility forecasts using GARCH(1,1) model with 24-hour TTL';
COMMENT ON COLUMN volatility_forecasts.ticker IS 'Stock ticker symbol';
COMMENT ON COLUMN volatility_forecasts.horizon_days IS 'Forecast horizon in days (1-90)';
COMMENT ON COLUMN volatility_forecasts.confidence_level IS 'Confidence level for intervals (0.80 or 0.95)';
COMMENT ON COLUMN volatility_forecasts.forecast_data IS 'Complete VolatilityForecast JSON with all forecast points and parameters';
COMMENT ON COLUMN volatility_forecasts.current_volatility IS 'Current realized volatility (annualized percentage)';
COMMENT ON COLUMN volatility_forecasts.avg_forecasted_volatility IS 'Average forecasted volatility across horizon';
COMMENT ON COLUMN volatility_forecasts.garch_omega IS 'GARCH constant term (long-run variance component)';
COMMENT ON COLUMN volatility_forecasts.garch_alpha IS 'GARCH ARCH coefficient (recent shock weight)';
COMMENT ON COLUMN volatility_forecasts.garch_beta IS 'GARCH coefficient (past variance weight)';
COMMENT ON COLUMN volatility_forecasts.persistence IS 'Volatility persistence (α+β), higher = longer-lasting shocks';

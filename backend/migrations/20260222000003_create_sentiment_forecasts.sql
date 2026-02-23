-- Create table for storing sentiment-augmented forecasts
-- These forecasts combine technical analysis with sentiment factors

CREATE TABLE IF NOT EXISTS sentiment_forecast_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticker VARCHAR(10) NOT NULL,

    -- Forecast metadata
    forecast_days INTEGER NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL, -- 12-hour cache TTL

    -- Sentiment factors (at time of forecast generation)
    news_sentiment NUMERIC(4, 3) NOT NULL,
    sec_filing_sentiment NUMERIC(4, 3),
    insider_sentiment NUMERIC(4, 3) NOT NULL,
    combined_sentiment NUMERIC(4, 3) NOT NULL,
    sentiment_momentum NUMERIC(4, 3) NOT NULL,
    spike_detected BOOLEAN NOT NULL DEFAULT FALSE,
    divergence_detected BOOLEAN NOT NULL DEFAULT FALSE,

    -- Forecast adjustments
    reversal_probability NUMERIC(4, 3) NOT NULL DEFAULT 0.0, -- 0.0 to 1.0
    confidence_adjustment NUMERIC(4, 3) NOT NULL DEFAULT 1.0, -- 0.5 to 1.5

    -- Base forecast (JSON array of ForecastPoint)
    base_forecast JSONB NOT NULL,

    -- Sentiment-adjusted forecast (JSON array of ForecastPoint)
    sentiment_adjusted_forecast JSONB NOT NULL,

    -- Divergence flags and warnings
    divergence_flags JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- Methodology used
    methodology TEXT NOT NULL DEFAULT 'Sentiment-Augmented Linear Regression',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Only store one forecast per ticker at a time (most recent)
    UNIQUE(ticker)
);

-- Index for querying by ticker
CREATE INDEX idx_sentiment_forecast_ticker ON sentiment_forecast_cache(ticker);

-- Index for cleanup of expired cache entries
CREATE INDEX idx_sentiment_forecast_expires ON sentiment_forecast_cache(expires_at);

-- Comments
COMMENT ON TABLE sentiment_forecast_cache IS 'Caches sentiment-aware stock price forecasts (12-hour TTL)';
COMMENT ON COLUMN sentiment_forecast_cache.combined_sentiment IS 'Combined sentiment score from news, SEC filings, and insider activity (-1.0 to 1.0)';
COMMENT ON COLUMN sentiment_forecast_cache.sentiment_momentum IS 'Rate of change in sentiment (7-day change)';
COMMENT ON COLUMN sentiment_forecast_cache.spike_detected IS 'Whether an unusual sentiment spike was detected';
COMMENT ON COLUMN sentiment_forecast_cache.divergence_detected IS 'Whether sentiment-price divergence was detected';
COMMENT ON COLUMN sentiment_forecast_cache.reversal_probability IS 'Probability of price reversal based on divergence (0.0 to 1.0)';
COMMENT ON COLUMN sentiment_forecast_cache.confidence_adjustment IS 'Overall confidence adjustment factor (0.5 to 1.5)';
COMMENT ON COLUMN sentiment_forecast_cache.base_forecast IS 'Technical forecast without sentiment adjustments';
COMMENT ON COLUMN sentiment_forecast_cache.sentiment_adjusted_forecast IS 'Forecast with sentiment-adjusted confidence intervals';
COMMENT ON COLUMN sentiment_forecast_cache.divergence_flags IS 'Human-readable divergence warnings';

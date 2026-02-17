-- Create sentiment signal cache table for Sprint 18
CREATE TABLE sentiment_signal_cache (
    ticker VARCHAR(10) NOT NULL PRIMARY KEY,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    -- Current metrics
    current_sentiment DOUBLE PRECISION NOT NULL,
    sentiment_trend VARCHAR(20) NOT NULL,
    momentum_trend VARCHAR(20) NOT NULL,
    divergence VARCHAR(20) NOT NULL,

    -- Correlation data
    sentiment_price_correlation DOUBLE PRECISION,
    correlation_lag_days INTEGER,

    -- Historical data
    historical_sentiment JSONB NOT NULL,

    -- Metadata
    news_articles_analyzed INTEGER NOT NULL,
    warnings JSONB NOT NULL
);

CREATE INDEX idx_sentiment_cache_expires_at ON sentiment_signal_cache(expires_at);
CREATE INDEX idx_sentiment_cache_divergence ON sentiment_signal_cache(divergence);

COMMENT ON TABLE sentiment_signal_cache IS 'Cached sentiment signals with 6-hour TTL (more frequent than other caches due to news freshness)';

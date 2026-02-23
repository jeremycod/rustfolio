-- Create stock_signals table for storing trading signals
-- This table caches generated trading signals based on technical indicators
-- Signals expire after 4 hours to ensure fresh analysis

CREATE TABLE IF NOT EXISTS stock_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticker VARCHAR(10) NOT NULL,
    signal_type VARCHAR(50) NOT NULL, -- 'momentum', 'mean_reversion', 'trend', 'combined'
    horizon_months INTEGER NOT NULL,
    probability NUMERIC(5, 4) NOT NULL CHECK (probability >= 0.0 AND probability <= 1.0), -- 0.0 to 1.0
    direction VARCHAR(10) NOT NULL, -- 'bullish', 'bearish', 'neutral'
    confidence_level VARCHAR(20) NOT NULL, -- 'high', 'medium', 'low'
    contributing_factors JSONB NOT NULL, -- which indicators contributed
    explanation TEXT NOT NULL, -- human-readable explanation
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- cache expiry (default 4 hours from generation)
    UNIQUE(ticker, signal_type, horizon_months, generated_at)
);

-- Create indices for efficient querying
CREATE INDEX idx_stock_signals_ticker ON stock_signals(ticker, generated_at DESC);
CREATE INDEX idx_stock_signals_expires ON stock_signals(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_stock_signals_ticker_horizon ON stock_signals(ticker, horizon_months, generated_at DESC);
CREATE INDEX idx_stock_signals_ticker_type ON stock_signals(ticker, signal_type, generated_at DESC);

-- Add comment for documentation
COMMENT ON TABLE stock_signals IS 'Trading signals generated from technical indicators. Signals expire after 4 hours to ensure fresh analysis.';
COMMENT ON COLUMN stock_signals.ticker IS 'Stock ticker symbol';
COMMENT ON COLUMN stock_signals.signal_type IS 'Type of signal: momentum, mean_reversion, trend, or combined';
COMMENT ON COLUMN stock_signals.horizon_months IS 'Time horizon in months (1, 3, 6, 12)';
COMMENT ON COLUMN stock_signals.probability IS 'Probability score from 0.0 to 1.0';
COMMENT ON COLUMN stock_signals.direction IS 'Signal direction: bullish, bearish, or neutral';
COMMENT ON COLUMN stock_signals.confidence_level IS 'Confidence level: high (>70%), medium (55-70%), or low (<55%)';
COMMENT ON COLUMN stock_signals.contributing_factors IS 'JSON containing individual factors and their weights';
COMMENT ON COLUMN stock_signals.explanation IS 'Human-readable explanation of the signal';
COMMENT ON COLUMN stock_signals.expires_at IS 'When the cached signal expires (typically 4 hours from generation)';

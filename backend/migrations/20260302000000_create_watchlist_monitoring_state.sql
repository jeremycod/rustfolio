-- Create watchlist_monitoring_state table to track monitoring metrics for each watchlist item
-- This table stores the last known values for various metrics to enable change detection

CREATE TABLE watchlist_monitoring_state (
    watchlist_item_id UUID PRIMARY KEY REFERENCES watchlist_items(id) ON DELETE CASCADE,
    last_price DOUBLE PRECISION,
    last_rsi DOUBLE PRECISION,
    last_volume_ratio DOUBLE PRECISION,
    last_volatility DOUBLE PRECISION,
    last_sentiment_score DOUBLE PRECISION,
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient lookups
CREATE INDEX idx_watchlist_monitoring_state_item ON watchlist_monitoring_state(watchlist_item_id);
CREATE INDEX idx_watchlist_monitoring_state_checked ON watchlist_monitoring_state(last_checked_at);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_watchlist_monitoring_state_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER watchlist_monitoring_state_updated_at
    BEFORE UPDATE ON watchlist_monitoring_state
    FOR EACH ROW
    EXECUTE FUNCTION update_watchlist_monitoring_state_updated_at();

-- Comments
COMMENT ON TABLE watchlist_monitoring_state IS 'Tracks last known metric values for watchlist items to detect changes';
COMMENT ON COLUMN watchlist_monitoring_state.last_price IS 'Last recorded price for the ticker';
COMMENT ON COLUMN watchlist_monitoring_state.last_rsi IS 'Last calculated RSI value';
COMMENT ON COLUMN watchlist_monitoring_state.last_volume_ratio IS 'Last calculated volume ratio';
COMMENT ON COLUMN watchlist_monitoring_state.last_volatility IS 'Last calculated volatility';
COMMENT ON COLUMN watchlist_monitoring_state.last_sentiment_score IS 'Last calculated sentiment score';
COMMENT ON COLUMN watchlist_monitoring_state.last_checked_at IS 'When the monitoring last ran for this item';

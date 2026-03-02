-- Create watchlist_thresholds table to store individual threshold rules
-- This allows for more granular threshold management than JSONB

CREATE TABLE watchlist_thresholds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    watchlist_item_id UUID NOT NULL REFERENCES watchlist_items(id) ON DELETE CASCADE,
    threshold_type VARCHAR(50) NOT NULL,
    comparison VARCHAR(10) NOT NULL,
    value DECIMAL(20,6) NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure only one threshold per type per item
    CONSTRAINT unique_item_threshold_type UNIQUE(watchlist_item_id, threshold_type),

    -- Validate threshold types
    CONSTRAINT valid_threshold_type CHECK (
        threshold_type IN (
            'price_above',
            'price_below',
            'price_change_pct',
            'volatility',
            'volume_spike',
            'rsi_overbought',
            'rsi_oversold'
        )
    ),

    -- Validate comparison operators
    CONSTRAINT valid_comparison CHECK (
        comparison IN ('gt', 'gte', 'lt', 'lte')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_watchlist_thresholds_item ON watchlist_thresholds(watchlist_item_id);
CREATE INDEX idx_watchlist_thresholds_enabled ON watchlist_thresholds(watchlist_item_id, enabled)
    WHERE enabled = TRUE;

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_watchlist_thresholds_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER watchlist_thresholds_updated_at
    BEFORE UPDATE ON watchlist_thresholds
    FOR EACH ROW
    EXECUTE FUNCTION update_watchlist_thresholds_updated_at();

-- Comments
COMMENT ON TABLE watchlist_thresholds IS 'Individual threshold rules for watchlist items';
COMMENT ON COLUMN watchlist_thresholds.threshold_type IS 'Type of threshold (price_above, price_below, volatility, etc.)';
COMMENT ON COLUMN watchlist_thresholds.comparison IS 'Comparison operator (gt, gte, lt, lte)';
COMMENT ON COLUMN watchlist_thresholds.value IS 'Threshold value to compare against';
COMMENT ON COLUMN watchlist_thresholds.enabled IS 'Whether the threshold is active';

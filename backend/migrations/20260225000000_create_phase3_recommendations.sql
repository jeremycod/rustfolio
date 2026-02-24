-- Phase 3: AI-Powered Stock Recommendations
-- Migration: Create tables for recommendations, watchlists, watchlist items,
--            watchlist alerts, and recommendation explanations

-- ==============================================================================
-- TABLE: recommendations
-- Stores generated stock recommendations with multi-factor scores
-- ==============================================================================
CREATE TABLE recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL,
    score DECIMAL(5,2) NOT NULL,
    factors JSONB NOT NULL DEFAULT '{}',
    recommendation_type VARCHAR(50) NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    CONSTRAINT recommendations_score_range CHECK (score >= 0 AND score <= 100),
    CONSTRAINT recommendations_type_valid CHECK (
        recommendation_type IN ('screen', 'long-term', 'factor')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_recommendations_user_generated ON recommendations(user_id, generated_at DESC);
CREATE INDEX idx_recommendations_symbol ON recommendations(symbol);
CREATE INDEX idx_recommendations_score ON recommendations(score DESC);
CREATE INDEX idx_recommendations_type ON recommendations(recommendation_type);
CREATE INDEX idx_recommendations_expires ON recommendations(expires_at)
    WHERE expires_at IS NOT NULL;

COMMENT ON TABLE recommendations IS 'Phase 3: AI-powered stock recommendations with multi-factor scores';
COMMENT ON COLUMN recommendations.factors IS 'JSON object containing individual factor scores (fundamental, technical, sentiment, momentum)';
COMMENT ON COLUMN recommendations.recommendation_type IS 'Type of recommendation: screen, long-term, or factor';
COMMENT ON COLUMN recommendations.score IS 'Composite recommendation score from 0-100';

-- ==============================================================================
-- TABLE: watchlists
-- User-created watchlists for monitoring stocks
-- ==============================================================================
CREATE TABLE watchlists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_watchlist_name UNIQUE(user_id, name)
);

-- Indexes for efficient querying
CREATE INDEX idx_watchlists_user ON watchlists(user_id);
CREATE INDEX idx_watchlists_user_default ON watchlists(user_id, is_default)
    WHERE is_default = TRUE;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_watchlists_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER watchlists_updated_at
    BEFORE UPDATE ON watchlists
    FOR EACH ROW
    EXECUTE FUNCTION update_watchlists_updated_at();

COMMENT ON TABLE watchlists IS 'Phase 3: User-created watchlists for monitoring stocks';
COMMENT ON COLUMN watchlists.is_default IS 'Whether this is the user default watchlist (one per user)';

-- ==============================================================================
-- TABLE: watchlist_items
-- Individual stocks within a watchlist, with optional custom thresholds
-- ==============================================================================
CREATE TABLE watchlist_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    watchlist_id UUID NOT NULL REFERENCES watchlists(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    custom_thresholds JSONB,
    notes TEXT,

    CONSTRAINT unique_watchlist_symbol UNIQUE(watchlist_id, symbol)
);

-- Indexes for efficient querying
CREATE INDEX idx_watchlist_items_watchlist ON watchlist_items(watchlist_id);
CREATE INDEX idx_watchlist_items_symbol ON watchlist_items(symbol);

COMMENT ON TABLE watchlist_items IS 'Phase 3: Individual stocks within a watchlist';
COMMENT ON COLUMN watchlist_items.custom_thresholds IS 'User-defined alert thresholds JSON (e.g., price_target, volatility_max, rsi_overbought)';

-- ==============================================================================
-- TABLE: watchlist_alerts
-- Triggered alerts for stocks in watchlists
-- ==============================================================================
CREATE TABLE watchlist_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    watchlist_id UUID NOT NULL REFERENCES watchlists(id) ON DELETE CASCADE,
    watchlist_item_id UUID REFERENCES watchlist_items(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL,
    alert_type VARCHAR(50) NOT NULL,
    threshold_value DECIMAL(20,6),
    actual_value DECIMAL(20,6),
    message TEXT NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged BOOLEAN DEFAULT FALSE,
    acknowledged_at TIMESTAMPTZ,

    CONSTRAINT watchlist_alerts_type_valid CHECK (
        alert_type IN ('price_target', 'volatility', 'sentiment', 'technical', 'volume', 'rsi')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_watchlist_alerts_watchlist ON watchlist_alerts(watchlist_id, triggered_at DESC);
CREATE INDEX idx_watchlist_alerts_symbol ON watchlist_alerts(symbol);
CREATE INDEX idx_watchlist_alerts_item ON watchlist_alerts(watchlist_item_id)
    WHERE watchlist_item_id IS NOT NULL;
CREATE INDEX idx_watchlist_alerts_unacknowledged ON watchlist_alerts(watchlist_id)
    WHERE acknowledged = FALSE;
CREATE INDEX idx_watchlist_alerts_type ON watchlist_alerts(alert_type);
CREATE INDEX idx_watchlist_alerts_triggered ON watchlist_alerts(triggered_at DESC);

-- Trigger to set acknowledged_at when acknowledged
CREATE OR REPLACE FUNCTION set_watchlist_alert_acknowledged_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.acknowledged = TRUE AND OLD.acknowledged = FALSE THEN
        NEW.acknowledged_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER watchlist_alert_acknowledged_at
    BEFORE UPDATE ON watchlist_alerts
    FOR EACH ROW
    EXECUTE FUNCTION set_watchlist_alert_acknowledged_at();

COMMENT ON TABLE watchlist_alerts IS 'Phase 3: Triggered alerts for stocks in watchlists';
COMMENT ON COLUMN watchlist_alerts.alert_type IS 'Type of alert: price_target, volatility, sentiment, technical, volume, rsi';
COMMENT ON COLUMN watchlist_alerts.threshold_value IS 'The threshold value that was configured';
COMMENT ON COLUMN watchlist_alerts.actual_value IS 'The actual value that triggered the alert';

-- ==============================================================================
-- TABLE: recommendation_explanations
-- Cached LLM-generated explanations for recommendations
-- ==============================================================================
CREATE TABLE recommendation_explanations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(10) NOT NULL,
    recommendation_id UUID REFERENCES recommendations(id) ON DELETE SET NULL,
    explanation TEXT NOT NULL,
    factors_snapshot JSONB NOT NULL DEFAULT '{}',
    model_used VARCHAR(100),
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT unique_symbol_explanation UNIQUE(symbol, generated_at)
);

-- Indexes for efficient querying
CREATE INDEX idx_recommendation_explanations_symbol ON recommendation_explanations(symbol, generated_at DESC);
CREATE INDEX idx_recommendation_explanations_expiry ON recommendation_explanations(expires_at);
CREATE INDEX idx_recommendation_explanations_recommendation ON recommendation_explanations(recommendation_id)
    WHERE recommendation_id IS NOT NULL;

-- Composite index for fast cache lookups by symbol with expiry ordering
CREATE INDEX idx_recommendation_explanations_active ON recommendation_explanations(symbol, expires_at DESC);

COMMENT ON TABLE recommendation_explanations IS 'Phase 3: Cached LLM-generated explanations for stock recommendations';
COMMENT ON COLUMN recommendation_explanations.factors_snapshot IS 'Snapshot of factor data at the time the explanation was generated';
COMMENT ON COLUMN recommendation_explanations.model_used IS 'LLM model identifier used to generate the explanation';
COMMENT ON COLUMN recommendation_explanations.expires_at IS 'Expiration time for cache invalidation (typically 1 hour TTL)';

-- ==============================================================================
-- VIEWS: Convenience views for common queries
-- ==============================================================================

-- Active (non-expired) recommendations
CREATE OR REPLACE VIEW active_recommendations AS
SELECT *
FROM recommendations
WHERE expires_at IS NULL OR expires_at > NOW()
ORDER BY score DESC;

-- Unacknowledged watchlist alerts
CREATE OR REPLACE VIEW unacknowledged_watchlist_alerts AS
SELECT wa.*, w.user_id, w.name AS watchlist_name
FROM watchlist_alerts wa
JOIN watchlists w ON wa.watchlist_id = w.id
WHERE wa.acknowledged = FALSE
ORDER BY wa.triggered_at DESC;

-- Watchlist overview with item counts
CREATE OR REPLACE VIEW watchlist_overview AS
SELECT
    w.id,
    w.user_id,
    w.name,
    w.description,
    w.is_default,
    w.created_at,
    w.updated_at,
    COUNT(wi.id) AS item_count,
    COUNT(wa.id) FILTER (WHERE wa.acknowledged = FALSE) AS unacknowledged_alert_count
FROM watchlists w
LEFT JOIN watchlist_items wi ON w.id = wi.watchlist_id
LEFT JOIN watchlist_alerts wa ON w.id = wa.watchlist_id
GROUP BY w.id;

-- ==============================================================================
-- JOB CONFIG: Add watchlist monitoring job
-- ==============================================================================
INSERT INTO job_config (job_name, schedule, max_duration_minutes)
VALUES
    ('monitor_watchlists', '0 */5 * * * *', 10),
    ('refresh_recommendations', '0 0 5 * * *', 60),
    ('cleanup_expired_explanations', '0 0 4 * * SUN', 15)
ON CONFLICT (job_name) DO UPDATE SET schedule = EXCLUDED.schedule;

-- ==============================================================================
-- CLEANUP: Function to delete expired recommendation explanations
-- ==============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_recommendation_explanations()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM recommendation_explanations
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_recommendation_explanations IS 'Deletes expired recommendation explanations from cache';

-- ==============================================================================
-- CLEANUP: Function to delete old watchlist alerts
-- ==============================================================================
CREATE OR REPLACE FUNCTION cleanup_old_watchlist_alerts(days_to_keep INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM watchlist_alerts
    WHERE triggered_at < NOW() - (days_to_keep || ' days')::INTERVAL
      AND acknowledged = TRUE;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_watchlist_alerts IS 'Deletes acknowledged watchlist alerts older than specified days (default 90)';

-- ==============================================================================
-- CLEANUP: Function to delete old recommendations
-- ==============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_recommendations(days_to_keep INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM recommendations
    WHERE expires_at IS NOT NULL
      AND expires_at < NOW() - (days_to_keep || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_recommendations IS 'Deletes expired recommendations older than specified days (default 30)';

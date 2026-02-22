-- Sprint 20: Alerts & Notifications System
-- Migration: Create tables for alert rules, alert history, notifications, and preferences

-- ==============================================================================
-- TABLE: users (minimal implementation for alert ownership)
-- NOTE: This is a placeholder until proper authentication is implemented
-- ==============================================================================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add user_id to portfolios if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name='portfolios' AND column_name='user_id') THEN
        ALTER TABLE portfolios ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;
        CREATE INDEX idx_portfolios_user_id ON portfolios(user_id);
    END IF;
END $$;

-- Create a default user for existing portfolios
INSERT INTO users (id, email, name)
VALUES ('00000000-0000-0000-0000-000000000001'::UUID, 'default@rustfolio.local', 'Default User')
ON CONFLICT (email) DO NOTHING;

-- Assign existing portfolios to default user
UPDATE portfolios SET user_id = '00000000-0000-0000-0000-000000000001'::UUID WHERE user_id IS NULL;

-- ==============================================================================
-- TABLE: alert_rules
-- Stores user-defined alert rules for monitoring portfolios and positions
-- ==============================================================================
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    portfolio_id UUID REFERENCES portfolios(id) ON DELETE CASCADE,
    ticker VARCHAR(10),  -- NULL for portfolio-wide alerts

    -- Rule configuration
    rule_type VARCHAR(50) NOT NULL,  -- 'price_change', 'volatility_spike', 'drawdown', etc.
    threshold DECIMAL(10, 4) NOT NULL,
    comparison VARCHAR(10) NOT NULL,  -- 'gt', 'lt', 'gte', 'lte', 'eq'
    enabled BOOLEAN DEFAULT TRUE,

    -- User-facing information
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Notification settings
    notification_channels TEXT[] DEFAULT ARRAY['email', 'in_app'],
    cooldown_hours INTEGER DEFAULT 24,  -- Prevent alert spam
    last_triggered_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_portfolio_or_ticker CHECK (
        (portfolio_id IS NOT NULL AND ticker IS NULL) OR
        (portfolio_id IS NULL AND ticker IS NOT NULL) OR
        (portfolio_id IS NOT NULL AND ticker IS NOT NULL)
    ),
    CONSTRAINT check_comparison_valid CHECK (
        comparison IN ('gt', 'lt', 'gte', 'lte', 'eq')
    ),
    CONSTRAINT check_cooldown_positive CHECK (cooldown_hours > 0)
);

-- Indexes for efficient querying
CREATE INDEX idx_alert_rules_user_id ON alert_rules(user_id);
CREATE INDEX idx_alert_rules_enabled ON alert_rules(enabled) WHERE enabled = TRUE;
CREATE INDEX idx_alert_rules_portfolio_id ON alert_rules(portfolio_id) WHERE portfolio_id IS NOT NULL;
CREATE INDEX idx_alert_rules_ticker ON alert_rules(ticker) WHERE ticker IS NOT NULL;
CREATE INDEX idx_alert_rules_user_enabled ON alert_rules(user_id, enabled) WHERE enabled = TRUE;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_alert_rules_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER alert_rules_updated_at
    BEFORE UPDATE ON alert_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_alert_rules_updated_at();

-- ==============================================================================
-- TABLE: alert_history
-- Stores all triggered alerts for audit and history tracking
-- ==============================================================================
CREATE TABLE alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    portfolio_id UUID REFERENCES portfolios(id) ON DELETE SET NULL,
    ticker VARCHAR(10),

    -- Alert details
    rule_type VARCHAR(50) NOT NULL,
    threshold DECIMAL(10, 4) NOT NULL,
    actual_value DECIMAL(10, 4) NOT NULL,
    message TEXT NOT NULL,
    severity VARCHAR(20) DEFAULT 'medium',  -- 'low', 'medium', 'high', 'critical'

    -- Timestamps
    triggered_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Additional context (JSON)
    metadata JSONB DEFAULT '{}',

    -- Constraints
    CONSTRAINT check_severity_valid CHECK (
        severity IN ('low', 'medium', 'high', 'critical')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_alert_history_user_id ON alert_history(user_id);
CREATE INDEX idx_alert_history_triggered_at ON alert_history(triggered_at DESC);
CREATE INDEX idx_alert_history_rule_id ON alert_history(alert_rule_id);
CREATE INDEX idx_alert_history_portfolio_id ON alert_history(portfolio_id) WHERE portfolio_id IS NOT NULL;
CREATE INDEX idx_alert_history_ticker ON alert_history(ticker) WHERE ticker IS NOT NULL;
CREATE INDEX idx_alert_history_user_triggered ON alert_history(user_id, triggered_at DESC);

-- ==============================================================================
-- TABLE: notifications
-- Stores in-app notifications for users
-- ==============================================================================
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    alert_history_id UUID REFERENCES alert_history(id) ON DELETE CASCADE,

    -- Notification content
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL,  -- 'alert', 'info', 'warning', 'error', 'success'

    -- Read status
    read BOOLEAN DEFAULT FALSE,
    read_at TIMESTAMPTZ,

    -- Navigation
    link VARCHAR(500),  -- Deep link to relevant page

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,  -- Optional expiry for time-sensitive notifications

    -- Constraints
    CONSTRAINT check_notification_type_valid CHECK (
        notification_type IN ('alert', 'info', 'warning', 'error', 'success')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read) WHERE read = FALSE;
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);

-- Trigger to set read_at when marking as read
CREATE OR REPLACE FUNCTION set_notification_read_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.read = TRUE AND OLD.read = FALSE THEN
        NEW.read_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER notification_read_at
    BEFORE UPDATE ON notifications
    FOR EACH ROW
    EXECUTE FUNCTION set_notification_read_at();

-- ==============================================================================
-- TABLE: notification_preferences
-- Stores user preferences for notification delivery
-- ==============================================================================
CREATE TABLE notification_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- Channel preferences
    email_enabled BOOLEAN DEFAULT TRUE,
    in_app_enabled BOOLEAN DEFAULT TRUE,
    webhook_enabled BOOLEAN DEFAULT FALSE,
    webhook_url VARCHAR(500),

    -- Quiet hours (time only, applies to user timezone)
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR(50) DEFAULT 'UTC',

    -- Rate limiting
    max_daily_emails INTEGER DEFAULT 10,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_max_daily_emails_positive CHECK (max_daily_emails > 0),
    CONSTRAINT check_max_daily_emails_reasonable CHECK (max_daily_emails <= 100)
);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_notification_preferences_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER notification_preferences_updated_at
    BEFORE UPDATE ON notification_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_preferences_updated_at();

-- ==============================================================================
-- TABLE: daily_email_counts
-- Tracks daily email counts per user for rate limiting
-- ==============================================================================
CREATE TABLE daily_email_counts (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    email_count INTEGER DEFAULT 0,

    PRIMARY KEY (user_id, date),

    CONSTRAINT check_email_count_nonnegative CHECK (email_count >= 0)
);

-- Index for efficient querying
CREATE INDEX idx_daily_email_counts_date ON daily_email_counts(date);

-- Function to increment daily email count
CREATE OR REPLACE FUNCTION increment_daily_email_count(p_user_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    INSERT INTO daily_email_counts (user_id, date, email_count)
    VALUES (p_user_id, CURRENT_DATE, 1)
    ON CONFLICT (user_id, date)
    DO UPDATE SET email_count = daily_email_counts.email_count + 1
    RETURNING email_count INTO v_count;

    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get daily email count
CREATE OR REPLACE FUNCTION get_daily_email_count(p_user_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    SELECT COALESCE(email_count, 0) INTO v_count
    FROM daily_email_counts
    WHERE user_id = p_user_id AND date = CURRENT_DATE;

    RETURN COALESCE(v_count, 0);
END;
$$ LANGUAGE plpgsql;

-- ==============================================================================
-- VIEWS: Convenience views for common queries
-- ==============================================================================

-- Active alerts view (excludes disabled rules)
CREATE OR REPLACE VIEW active_alert_rules AS
SELECT *
FROM alert_rules
WHERE enabled = TRUE;

-- Recent alerts view (last 7 days)
CREATE OR REPLACE VIEW recent_alerts AS
SELECT *
FROM alert_history
WHERE triggered_at >= NOW() - INTERVAL '7 days'
ORDER BY triggered_at DESC;

-- Unread notifications view
CREATE OR REPLACE VIEW unread_notifications AS
SELECT *
FROM notifications
WHERE read = FALSE
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY created_at DESC;

-- ==============================================================================
-- COMMENTS: Documentation for tables and columns
-- ==============================================================================

COMMENT ON TABLE alert_rules IS 'User-defined alert rules for monitoring portfolios and positions';
COMMENT ON TABLE alert_history IS 'Historical record of all triggered alerts';
COMMENT ON TABLE notifications IS 'In-app notifications for users';
COMMENT ON TABLE notification_preferences IS 'User preferences for notification delivery';
COMMENT ON TABLE daily_email_counts IS 'Tracks daily email counts for rate limiting';

COMMENT ON COLUMN alert_rules.rule_type IS 'Type of alert: price_change, volatility_spike, drawdown_exceeded, risk_threshold, sentiment_change, divergence';
COMMENT ON COLUMN alert_rules.comparison IS 'Comparison operator: gt (>), lt (<), gte (>=), lte (<=), eq (=)';
COMMENT ON COLUMN alert_rules.cooldown_hours IS 'Minimum hours between alert triggers to prevent spam';
COMMENT ON COLUMN alert_rules.notification_channels IS 'Array of channels: email, in_app, webhook';

COMMENT ON COLUMN alert_history.severity IS 'Alert severity: low, medium, high, critical';
COMMENT ON COLUMN alert_history.metadata IS 'Additional context about the alert in JSON format';

COMMENT ON COLUMN notifications.link IS 'Deep link to relevant page (e.g., /portfolios/:id)';
COMMENT ON COLUMN notifications.expires_at IS 'Optional expiration timestamp for time-sensitive notifications';

COMMENT ON COLUMN notification_preferences.quiet_hours_start IS 'Start of quiet hours (no notifications) in user timezone';
COMMENT ON COLUMN notification_preferences.quiet_hours_end IS 'End of quiet hours in user timezone';
COMMENT ON COLUMN notification_preferences.max_daily_emails IS 'Maximum number of email notifications per day';

-- ==============================================================================
-- SAMPLE DATA (for development/testing)
-- ==============================================================================

-- Note: Sample data will be inserted via application code or separate seed script
-- to avoid hardcoded UUIDs and maintain referential integrity

-- ==============================================================================
-- CLEANUP: Function to delete old alert history (optional, for maintenance)
-- ==============================================================================

CREATE OR REPLACE FUNCTION cleanup_old_alert_history(days_to_keep INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM alert_history
    WHERE triggered_at < NOW() - (days_to_keep || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_alert_history IS 'Deletes alert history older than specified days (default 90)';

-- ==============================================================================
-- CLEANUP: Function to delete old notifications (optional, for maintenance)
-- ==============================================================================

CREATE OR REPLACE FUNCTION cleanup_old_notifications(days_to_keep INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM notifications
    WHERE created_at < NOW() - (days_to_keep || ' days')::INTERVAL
      AND read = TRUE;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_notifications IS 'Deletes read notifications older than specified days (default 30)';

-- ==============================================================================
-- GRANTS: Set appropriate permissions (adjust as needed for your setup)
-- ==============================================================================

-- Note: Adjust these grants based on your database user setup
-- GRANT SELECT, INSERT, UPDATE, DELETE ON alert_rules TO app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON alert_history TO app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON notifications TO app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON notification_preferences TO app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON daily_email_counts TO app_user;

-- Add risk preference fields to user_preferences table
-- This migration extends the existing user_preferences table with risk appetite settings

-- Add new columns for risk preferences
ALTER TABLE user_preferences
    ADD COLUMN IF NOT EXISTS risk_appetite VARCHAR(20) NOT NULL DEFAULT 'Balanced',
    ADD COLUMN IF NOT EXISTS forecast_horizon_preference INTEGER NOT NULL DEFAULT 6,
    ADD COLUMN IF NOT EXISTS signal_sensitivity VARCHAR(20) NOT NULL DEFAULT 'Medium',
    ADD COLUMN IF NOT EXISTS sentiment_weight NUMERIC(3, 2) NOT NULL DEFAULT 0.30,
    ADD COLUMN IF NOT EXISTS technical_weight NUMERIC(3, 2) NOT NULL DEFAULT 0.40,
    ADD COLUMN IF NOT EXISTS fundamental_weight NUMERIC(3, 2) NOT NULL DEFAULT 0.30,
    ADD COLUMN IF NOT EXISTS custom_settings JSONB;

-- Add check constraints for data validation
ALTER TABLE user_preferences
    ADD CONSTRAINT check_risk_appetite
        CHECK (risk_appetite IN ('Conservative', 'Balanced', 'Aggressive')),
    ADD CONSTRAINT check_signal_sensitivity
        CHECK (signal_sensitivity IN ('Low', 'Medium', 'High')),
    ADD CONSTRAINT check_forecast_horizon
        CHECK (forecast_horizon_preference BETWEEN 1 AND 24),
    ADD CONSTRAINT check_sentiment_weight
        CHECK (sentiment_weight BETWEEN 0.0 AND 1.0),
    ADD CONSTRAINT check_technical_weight
        CHECK (technical_weight BETWEEN 0.0 AND 1.0),
    ADD CONSTRAINT check_fundamental_weight
        CHECK (fundamental_weight BETWEEN 0.0 AND 1.0);

-- Create index for efficient risk appetite queries
CREATE INDEX IF NOT EXISTS idx_user_preferences_risk_appetite
    ON user_preferences(risk_appetite);

-- Add comments for documentation
COMMENT ON COLUMN user_preferences.risk_appetite IS
    'User risk tolerance level: Conservative (long-term, low risk), Balanced (medium risk/return), Aggressive (short-term, high risk)';
COMMENT ON COLUMN user_preferences.forecast_horizon_preference IS
    'Preferred forecast horizon in months (1-24), affects default time range for predictions';
COMMENT ON COLUMN user_preferences.signal_sensitivity IS
    'Signal generation sensitivity: Low (high confidence only), Medium (balanced), High (more signals, lower threshold)';
COMMENT ON COLUMN user_preferences.sentiment_weight IS
    'Weight for sentiment analysis in signal generation (0.0-1.0)';
COMMENT ON COLUMN user_preferences.technical_weight IS
    'Weight for technical indicators in signal generation (0.0-1.0)';
COMMENT ON COLUMN user_preferences.fundamental_weight IS
    'Weight for fundamental analysis in signal generation (0.0-1.0)';
COMMENT ON COLUMN user_preferences.custom_settings IS
    'Extensible JSON field for additional user-specific preferences';

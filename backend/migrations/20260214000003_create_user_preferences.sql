-- Create user_preferences table for tracking AI feature consent and settings
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL,
    llm_enabled BOOLEAN NOT NULL DEFAULT false,
    consent_given_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add index for efficient lookups by user_id
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

-- Add comments
COMMENT ON TABLE user_preferences IS 'User preferences and consent for AI-powered features';
COMMENT ON COLUMN user_preferences.user_id IS 'Foreign key to users table (unique constraint ensures one preference record per user)';
COMMENT ON COLUMN user_preferences.llm_enabled IS 'Whether the user has enabled AI features';
COMMENT ON COLUMN user_preferences.consent_given_at IS 'Timestamp when user gave consent for AI features (NULL if not yet consented)';
COMMENT ON COLUMN user_preferences.updated_at IS 'Last time preferences were modified';

-- Create llm_usage table for tracking AI feature costs and usage
CREATE TABLE IF NOT EXISTS llm_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    portfolio_id UUID,
    model VARCHAR(50) NOT NULL,
    prompt_tokens INT NOT NULL,
    completion_tokens INT NOT NULL,
    total_cost NUMERIC(10, 6) NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes for efficient queries
CREATE INDEX idx_llm_usage_user_id ON llm_usage(user_id);
CREATE INDEX idx_llm_usage_portfolio_id ON llm_usage(portfolio_id);
CREATE INDEX idx_llm_usage_created_at ON llm_usage(created_at);
CREATE INDEX idx_llm_usage_user_created ON llm_usage(user_id, created_at);

-- Add comments
COMMENT ON TABLE llm_usage IS 'Tracks LLM API usage for cost monitoring and analytics';
COMMENT ON COLUMN llm_usage.user_id IS 'User who triggered the LLM request (nullable for system requests)';
COMMENT ON COLUMN llm_usage.portfolio_id IS 'Portfolio associated with the request (if applicable)';
COMMENT ON COLUMN llm_usage.model IS 'LLM model used (e.g., gpt-4o-mini)';
COMMENT ON COLUMN llm_usage.prompt_tokens IS 'Number of tokens in the prompt';
COMMENT ON COLUMN llm_usage.completion_tokens IS 'Number of tokens in the completion';
COMMENT ON COLUMN llm_usage.total_cost IS 'Total cost in USD for this request';

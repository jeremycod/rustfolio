-- Create ticker fetch failures table for tracking failed API attempts
-- This prevents repeated API calls to external services for tickers that are known to fail

CREATE TABLE IF NOT EXISTS ticker_fetch_failures (
    ticker TEXT PRIMARY KEY,
    last_attempt_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    failure_type TEXT NOT NULL, -- 'not_found', 'rate_limited', 'api_error'
    retry_after TIMESTAMP NOT NULL, -- When we can retry this ticker
    consecutive_failures INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for efficient cleanup of expired entries
CREATE INDEX IF NOT EXISTS idx_ticker_fetch_failures_retry_after
    ON ticker_fetch_failures(retry_after);

-- Index for looking up by ticker and checking if retry is allowed
CREATE INDEX IF NOT EXISTS idx_ticker_fetch_failures_ticker_retry
    ON ticker_fetch_failures(ticker, retry_after);

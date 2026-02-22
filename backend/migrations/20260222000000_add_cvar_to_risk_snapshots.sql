-- Add CVaR (Expected Shortfall) columns to risk_snapshots table
-- CVaR provides a more comprehensive tail-risk measure than VaR alone

-- Add VaR and CVaR columns for both 95% and 99% confidence levels
ALTER TABLE risk_snapshots
ADD COLUMN IF NOT EXISTS var_95 NUMERIC(10, 4),
ADD COLUMN IF NOT EXISTS var_99 NUMERIC(10, 4),
ADD COLUMN IF NOT EXISTS expected_shortfall_95 NUMERIC(10, 4),
ADD COLUMN IF NOT EXISTS expected_shortfall_99 NUMERIC(10, 4);

-- Add comments to document the metrics
COMMENT ON COLUMN risk_snapshots.var_95 IS 'Value at Risk at 95% confidence level (1-day horizon), as a negative percentage';
COMMENT ON COLUMN risk_snapshots.var_99 IS 'Value at Risk at 99% confidence level (1-day horizon), as a negative percentage';
COMMENT ON COLUMN risk_snapshots.expected_shortfall_95 IS 'Expected Shortfall (CVaR) at 95% confidence level - average loss beyond VaR threshold';
COMMENT ON COLUMN risk_snapshots.expected_shortfall_99 IS 'Expected Shortfall (CVaR) at 99% confidence level - average loss beyond VaR threshold';

-- Create index for querying positions with high tail risk
CREATE INDEX IF NOT EXISTS idx_risk_snapshots_expected_shortfall
ON risk_snapshots(portfolio_id, expected_shortfall_99 DESC NULLS LAST)
WHERE expected_shortfall_99 IS NOT NULL;

-- Note: The existing value_at_risk column is kept for backward compatibility
-- New applications should use var_95 and var_99 for clarity

-- Create risk threshold settings table
-- Stores customizable risk thresholds per portfolio

CREATE TABLE risk_threshold_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,

    -- Volatility thresholds (as percentage, e.g., 30.0 for 30%)
    volatility_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT 30.0,
    volatility_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT 50.0,

    -- Drawdown thresholds (as negative percentage, e.g., -20.0 for -20%)
    drawdown_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT -20.0,
    drawdown_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT -35.0,

    -- Beta thresholds (relative to benchmark)
    beta_warning_threshold NUMERIC(4,2) NOT NULL DEFAULT 1.5,
    beta_critical_threshold NUMERIC(4,2) NOT NULL DEFAULT 2.0,

    -- Risk score thresholds (0-100 scale)
    risk_score_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT 60.0,
    risk_score_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT 80.0,

    -- VaR thresholds (as negative percentage, e.g., -5.0 for -5%)
    var_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT -5.0,
    var_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT -10.0,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one settings record per portfolio
    UNIQUE(portfolio_id)
);

-- Index for fast lookups by portfolio
CREATE INDEX idx_risk_thresholds_portfolio ON risk_threshold_settings(portfolio_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_risk_threshold_settings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER risk_threshold_settings_updated_at
    BEFORE UPDATE ON risk_threshold_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_risk_threshold_settings_updated_at();

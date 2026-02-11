-- Create risk_snapshots table for historical risk tracking
CREATE TABLE risk_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    ticker TEXT,  -- NULL for portfolio-level snapshots
    snapshot_date DATE NOT NULL,
    snapshot_type TEXT NOT NULL CHECK (snapshot_type IN ('portfolio', 'position')),

    -- Risk metrics (same as PositionRisk/PortfolioRisk)
    volatility NUMERIC(10, 4) NOT NULL,
    max_drawdown NUMERIC(10, 4) NOT NULL,
    beta NUMERIC(10, 4),
    sharpe NUMERIC(10, 4),
    value_at_risk NUMERIC(10, 4),
    risk_score NUMERIC(5, 2) NOT NULL,
    risk_level TEXT NOT NULL,

    -- Context
    total_value NUMERIC(15, 2),  -- For portfolio snapshots
    market_value NUMERIC(15, 2),  -- For position snapshots

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(portfolio_id, ticker, snapshot_date, snapshot_type)
);

-- Create indexes for efficient querying
CREATE INDEX idx_risk_snapshots_portfolio_date
    ON risk_snapshots(portfolio_id, snapshot_date DESC);

CREATE INDEX idx_risk_snapshots_ticker_date
    ON risk_snapshots(portfolio_id, ticker, snapshot_date DESC)
    WHERE ticker IS NOT NULL;

-- Create index for snapshot type filtering
CREATE INDEX idx_risk_snapshots_type
    ON risk_snapshots(portfolio_id, snapshot_type);

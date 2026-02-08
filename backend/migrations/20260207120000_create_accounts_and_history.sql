-- Create accounts table to track different accounts within a portfolio
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    account_number TEXT NOT NULL,
    account_nickname TEXT NOT NULL,
    client_id TEXT,
    client_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(portfolio_id, account_number)
);

CREATE INDEX idx_accounts_portfolio ON accounts(portfolio_id);

-- Create holdings_snapshots table to track historical holdings data
CREATE TABLE holdings_snapshots (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL,
    ticker TEXT NOT NULL,
    holding_name TEXT,
    asset_category TEXT,
    industry TEXT,
    quantity NUMERIC NOT NULL,
    price NUMERIC NOT NULL,
    average_cost NUMERIC NOT NULL,
    book_value NUMERIC NOT NULL,
    market_value NUMERIC NOT NULL,
    fund TEXT,
    accrued_interest NUMERIC,
    gain_loss NUMERIC,
    gain_loss_pct NUMERIC,
    percentage_of_assets NUMERIC,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(account_id, snapshot_date, ticker)
);

CREATE INDEX idx_holdings_snapshots_account_date ON holdings_snapshots(account_id, snapshot_date DESC);
CREATE INDEX idx_holdings_snapshots_ticker ON holdings_snapshots(ticker);

-- Add account_id to positions table (optional, for linking current positions to accounts)
ALTER TABLE positions ADD COLUMN account_id UUID REFERENCES accounts(id) ON DELETE SET NULL;
CREATE INDEX idx_positions_account ON positions(account_id);

-- Create a view for the latest holdings per account
CREATE VIEW latest_account_holdings AS
SELECT DISTINCT ON (h.account_id, h.ticker)
    h.id,
    h.account_id,
    a.account_nickname,
    a.account_number,
    h.ticker,
    h.holding_name,
    h.asset_category,
    h.quantity,
    h.price,
    h.market_value,
    h.gain_loss,
    h.gain_loss_pct,
    h.snapshot_date
FROM holdings_snapshots h
JOIN accounts a ON h.account_id = a.id
ORDER BY h.account_id, h.ticker, h.snapshot_date DESC;

-- Create a view for account value history over time
CREATE VIEW account_value_history AS
SELECT
    account_id,
    snapshot_date,
    SUM(market_value) as total_value,
    SUM(book_value) as total_cost,
    SUM(gain_loss) as total_gain_loss,
    CASE
        WHEN SUM(book_value) > 0 THEN (SUM(gain_loss) / SUM(book_value)) * 100
        ELSE 0
    END as total_gain_loss_pct
FROM holdings_snapshots
GROUP BY account_id, snapshot_date
ORDER BY account_id, snapshot_date;

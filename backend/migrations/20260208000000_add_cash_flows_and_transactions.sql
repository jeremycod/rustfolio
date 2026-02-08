-- Track deposits and withdrawals to accounts
CREATE TABLE cash_flows (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    flow_type TEXT NOT NULL CHECK (flow_type IN ('DEPOSIT', 'WITHDRAWAL')),
    amount NUMERIC NOT NULL CHECK (amount > 0),
    flow_date DATE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cash_flows_account_date ON cash_flows(account_id, flow_date DESC);

-- Track detected transactions between snapshots
CREATE TABLE detected_transactions (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('BUY', 'SELL', 'DIVIDEND', 'SPLIT', 'OTHER')),
    ticker TEXT NOT NULL,
    quantity NUMERIC,
    price NUMERIC,
    amount NUMERIC,
    transaction_date DATE NOT NULL,
    from_snapshot_date DATE,
    to_snapshot_date DATE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_detected_transactions_account ON detected_transactions(account_id, transaction_date DESC);
CREATE INDEX idx_detected_transactions_ticker ON detected_transactions(ticker);

-- Add cumulative deposits/withdrawals tracking to accounts
ALTER TABLE accounts ADD COLUMN total_deposits NUMERIC DEFAULT 0;
ALTER TABLE accounts ADD COLUMN total_withdrawals NUMERIC DEFAULT 0;

-- View to calculate true performance (gains excluding deposits/withdrawals)
CREATE VIEW account_true_performance AS
SELECT
    a.id as account_id,
    a.account_nickname,
    a.account_number,
    a.total_deposits,
    a.total_withdrawals,
    COALESCE(latest.total_value, 0) as current_value,
    COALESCE(latest.total_cost, 0) as book_value,
    -- True gain/loss = Current Value - (Total Deposits - Total Withdrawals)
    COALESCE(latest.total_value, 0) - (COALESCE(a.total_deposits, 0) - COALESCE(a.total_withdrawals, 0)) as true_gain_loss,
    CASE
        WHEN (COALESCE(a.total_deposits, 0) - COALESCE(a.total_withdrawals, 0)) > 0
        THEN ((COALESCE(latest.total_value, 0) - (COALESCE(a.total_deposits, 0) - COALESCE(a.total_withdrawals, 0))) / (COALESCE(a.total_deposits, 0) - COALESCE(a.total_withdrawals, 0))) * 100
        ELSE 0
    END as true_gain_loss_pct,
    latest.snapshot_date as as_of_date
FROM accounts a
LEFT JOIN LATERAL (
    SELECT
        avh.total_value,
        avh.total_cost,
        avh.snapshot_date
    FROM account_value_history avh
    WHERE avh.account_id = a.id
    ORDER BY avh.snapshot_date DESC
    LIMIT 1
) latest ON true;

-- View for transaction history with cash flows
CREATE VIEW account_activity AS
SELECT
    account_id,
    'TRANSACTION' as activity_type,
    transaction_type as type_detail,
    ticker,
    quantity,
    amount,
    transaction_date as activity_date,
    description
FROM detected_transactions
UNION ALL
SELECT
    account_id,
    'CASH_FLOW' as activity_type,
    flow_type as type_detail,
    NULL as ticker,
    NULL as quantity,
    amount,
    flow_date as activity_date,
    description
FROM cash_flows
ORDER BY activity_date DESC;

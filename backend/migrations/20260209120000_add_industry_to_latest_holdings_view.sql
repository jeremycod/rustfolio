-- Add industry field to latest_account_holdings view
-- This helps correctly identify mutual funds vs regular equities

-- Drop the existing view
DROP VIEW IF EXISTS latest_account_holdings;

-- Recreate the view with industry field
CREATE VIEW latest_account_holdings AS
SELECT DISTINCT ON (h.account_id, h.ticker)
    h.id,
    h.account_id,
    a.account_nickname,
    a.account_number,
    h.ticker,
    h.holding_name,
    h.asset_category,
    h.industry,
    h.quantity,
    h.price,
    h.market_value,
    h.gain_loss,
    h.gain_loss_pct,
    h.snapshot_date
FROM holdings_snapshots h
JOIN accounts a ON h.account_id = a.id
ORDER BY h.account_id, h.ticker, h.snapshot_date DESC;

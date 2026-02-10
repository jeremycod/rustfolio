-- Fix initial deposits for existing accounts
-- This migration recalculates deposits by creating initial deposits based on first snapshot values

-- Step 1: Delete all existing cash flows and reset account totals
DELETE FROM cash_flows;
UPDATE accounts SET total_deposits = 0, total_withdrawals = 0;

-- Step 2: For each account, create an initial deposit based on first snapshot value
INSERT INTO cash_flows (id, account_id, flow_type, amount, flow_date, description)
SELECT
    gen_random_uuid() as id,
    account_id,
    'DEPOSIT' as flow_type,
    total_value as amount,
    snapshot_date as flow_date,
    'Initial account value: $' || ROUND(total_value::numeric, 2)::text as description
FROM (
    SELECT
        hs.account_id,
        hs.snapshot_date,
        SUM(hs.quantity * hs.price) as total_value,
        ROW_NUMBER() OVER (PARTITION BY hs.account_id ORDER BY hs.snapshot_date ASC) as rn
    FROM holdings_snapshots hs
    GROUP BY hs.account_id, hs.snapshot_date
) first_snapshots
WHERE rn = 1 AND total_value > 1;

-- Step 3: Update account totals
UPDATE accounts
SET total_deposits = COALESCE((
    SELECT SUM(amount) FROM cash_flows
    WHERE account_id = accounts.id AND flow_type = 'DEPOSIT'
), 0),
total_withdrawals = COALESCE((
    SELECT SUM(amount) FROM cash_flows
    WHERE account_id = accounts.id AND flow_type = 'WITHDRAWAL'
), 0);

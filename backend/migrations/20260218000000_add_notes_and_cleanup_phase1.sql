-- Migration: Add notes column and Phase 1 cleanup
-- Created: 2026-02-18
-- Purpose: Support mutual fund tracking and clean up invalid tickers

-- Add notes column to holdings_snapshots
ALTER TABLE holdings_snapshots ADD COLUMN IF NOT EXISTS notes TEXT;

-- Add notes column to positions
ALTER TABLE positions ADD COLUMN IF NOT EXISTS notes TEXT;

-- Mark mutual fund holdings in holdings_snapshots
-- These are Canadian mutual fund codes that don't have public ticker symbols
-- and require manual price entry
UPDATE holdings_snapshots
SET notes = '[MUTUAL_FUND - Manual pricing needed]'
WHERE ticker IN (
    -- Fidelity funds
    'FID5494', 'FID5982', 'FID7648', 'FID631',
    -- Dynamic Funds
    'DYN3361', 'DYN3128',
    -- Edge funds
    'EDG5001',
    -- Brookfield funds
    'BIP502',
    -- Lysander funds
    'LYZ801F', 'LYZ932F',
    -- RBC funds
    'RBF1684',
    -- AGF funds
    'AGF9110',
    -- Manulife funds
    'MFC5505',
    -- Other fund codes
    'RPD210', 'MMF8621', 'NWT595'
)
AND (notes IS NULL OR notes NOT LIKE '%MUTUAL_FUND%');

-- Mark mutual fund positions
UPDATE positions
SET notes = '[MUTUAL_FUND - Manual pricing needed]'
WHERE ticker IN (
    'FID5494', 'FID5982', 'FID7648', 'FID631',
    'DYN3361', 'DYN3128', 'EDG5001', 'BIP502',
    'LYZ801F', 'LYZ932F', 'RBF1684', 'AGF9110',
    'MFC5505', 'RPD210', 'MMF8621', 'NWT595'
)
AND (notes IS NULL OR notes NOT LIKE '%MUTUAL_FUND%');

-- Clear rate-limited entries from ticker_fetch_failures
-- These can be safely retried after implementing rate limiting in Phase 2
DELETE FROM ticker_fetch_failures
WHERE failure_type = 'rate_limited';

-- Clean up empty/invalid tickers from holdings_snapshots
DELETE FROM holdings_snapshots
WHERE ticker IS NULL OR TRIM(ticker) = '';

-- Clean up empty/invalid tickers from positions
DELETE FROM positions
WHERE ticker IS NULL OR TRIM(ticker) = '';

-- Create an index on the notes column for faster filtering
CREATE INDEX IF NOT EXISTS idx_holdings_snapshots_notes ON holdings_snapshots(notes)
WHERE notes IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_positions_notes ON positions(notes)
WHERE notes IS NOT NULL;

-- Fix watchlist_items table schema to match backend code expectations
-- This migration adds missing columns that the watchlist_queries.rs code requires

-- Add missing columns to watchlist_items
ALTER TABLE watchlist_items
  ADD COLUMN IF NOT EXISTS sort_order INTEGER DEFAULT 0,
  ADD COLUMN IF NOT EXISTS added_price DECIMAL(12,2),
  ADD COLUMN IF NOT EXISTS target_price DECIMAL(12,2);

-- Rename symbol to ticker for consistency with backend code
ALTER TABLE watchlist_items
  RENAME COLUMN symbol TO ticker;

-- Create index on sort_order for efficient ordering
CREATE INDEX IF NOT EXISTS idx_watchlist_items_sort_order
  ON watchlist_items(watchlist_id, sort_order);

-- Update existing rows to have sequential sort_order
WITH ranked_items AS (
  SELECT id, ROW_NUMBER() OVER (PARTITION BY watchlist_id ORDER BY added_at) - 1 AS new_order
  FROM watchlist_items
)
UPDATE watchlist_items wi
SET sort_order = ri.new_order
FROM ranked_items ri
WHERE wi.id = ri.id;

-- Update watchlist_alerts to use ticker instead of symbol
ALTER TABLE watchlist_alerts
  RENAME COLUMN symbol TO ticker;

-- Add comments
COMMENT ON COLUMN watchlist_items.sort_order IS 'Display order of items within the watchlist (0-indexed)';
COMMENT ON COLUMN watchlist_items.added_price IS 'Price when the stock was added to the watchlist';
COMMENT ON COLUMN watchlist_items.target_price IS 'User-defined target price for alerts';
COMMENT ON COLUMN watchlist_items.ticker IS 'Stock ticker symbol (renamed from symbol for consistency)';

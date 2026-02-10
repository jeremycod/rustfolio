-- Add unique constraint on (ticker, date) for price_points table
-- This is required for ON CONFLICT upserts to work

-- Drop the existing index (will be replaced by unique constraint)
DROP INDEX IF EXISTS idx_price_ticker_date;

-- Add unique constraint
ALTER TABLE price_points
ADD CONSTRAINT price_points_ticker_date_unique UNIQUE (ticker, date);

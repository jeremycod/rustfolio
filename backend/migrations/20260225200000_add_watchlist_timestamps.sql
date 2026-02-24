-- Add missing timestamp columns to watchlist_items
-- The code references created_at and updated_at but they don't exist

-- Add created_at and updated_at columns
ALTER TABLE watchlist_items
  ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Set created_at to added_at for existing rows (so they match)
UPDATE watchlist_items
SET created_at = added_at
WHERE created_at IS NULL OR created_at = '1970-01-01 00:00:00+00';

-- Create trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_watchlist_items_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER watchlist_items_updated_at
    BEFORE UPDATE ON watchlist_items
    FOR EACH ROW
    EXECUTE FUNCTION update_watchlist_items_updated_at();

-- Add comments
COMMENT ON COLUMN watchlist_items.created_at IS 'Timestamp when the item was created (same as added_at)';
COMMENT ON COLUMN watchlist_items.updated_at IS 'Timestamp when the item was last updated';

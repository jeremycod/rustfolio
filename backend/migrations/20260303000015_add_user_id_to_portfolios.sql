-- Add user_id FK to portfolios for multi-tenancy
-- Uses IF NOT EXISTS to be idempotent (column may already exist from a previous partial run)
ALTER TABLE portfolios ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE CASCADE;

-- Assign all existing portfolios to the default user (only those with NULL user_id)
UPDATE portfolios SET user_id = '00000000-0000-0000-0000-000000000001' WHERE user_id IS NULL;

-- Make user_id mandatory
ALTER TABLE portfolios ALTER COLUMN user_id SET NOT NULL;

-- Fix FK constraint to CASCADE if it was previously SET NULL
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.referential_constraints
        WHERE constraint_name = 'portfolios_user_id_fkey'
          AND delete_rule = 'SET NULL'
    ) THEN
        ALTER TABLE portfolios DROP CONSTRAINT portfolios_user_id_fkey;
        ALTER TABLE portfolios ADD CONSTRAINT portfolios_user_id_fkey
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_portfolios_user ON portfolios(user_id);

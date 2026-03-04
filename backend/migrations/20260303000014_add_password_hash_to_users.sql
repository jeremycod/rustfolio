-- Add password_hash column to users table for local authentication
-- Nullable: existing default user has no password (login disabled for it)
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT;

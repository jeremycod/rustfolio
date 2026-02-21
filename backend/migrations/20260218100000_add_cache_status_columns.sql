-- Migration: Add status tracking columns to portfolio_risk_cache
-- Created: 2026-02-18
-- Purpose: Enable background job tracking for portfolio risk calculations
--
-- This migration adds status tracking capabilities to the portfolio_risk_cache table
-- to support asynchronous cache warming and error handling.
--
-- Status values:
--   - 'fresh': Cache entry is current and valid
--   - 'stale': Cache entry has expired and needs recalculation
--   - 'calculating': Background job is currently computing this cache entry
--   - 'error': Calculation failed, see last_error for details

-- Add calculation_status column
-- Tracks the current state of the cache entry for background job coordination
ALTER TABLE portfolio_risk_cache
ADD COLUMN IF NOT EXISTS calculation_status VARCHAR(20) DEFAULT 'stale';

-- Add last_error column
-- Stores error messages when calculations fail, for debugging and monitoring
ALTER TABLE portfolio_risk_cache
ADD COLUMN IF NOT EXISTS last_error TEXT;

-- Add retry_count column
-- Tracks number of failed calculation attempts, used for exponential backoff
ALTER TABLE portfolio_risk_cache
ADD COLUMN IF NOT EXISTS retry_count INTEGER DEFAULT 0;

-- Create index on calculation_status for efficient filtering
-- Used by background jobs to find entries that need processing
CREATE INDEX IF NOT EXISTS idx_portfolio_risk_cache_status
ON portfolio_risk_cache(calculation_status);

-- Add table and column comments for maintainability
COMMENT ON TABLE portfolio_risk_cache IS
'Caches portfolio risk calculations to avoid expensive recalculations and API calls. Supports background job processing with status tracking.';

COMMENT ON COLUMN portfolio_risk_cache.calculation_status IS
'Current calculation state: fresh (valid), stale (expired), calculating (in progress), error (failed). Used by background jobs for coordination.';

COMMENT ON COLUMN portfolio_risk_cache.last_error IS
'Error message from the most recent failed calculation attempt. NULL if last calculation succeeded.';

COMMENT ON COLUMN portfolio_risk_cache.retry_count IS
'Number of consecutive failed calculation attempts. Reset to 0 on successful calculation.';

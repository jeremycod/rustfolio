-- Cache Invalidation Triggers for Portfolio Risk and Correlations
-- Created: 2026-02-18
-- Purpose: Automatically invalidate portfolio risk and correlation caches when holdings change
--
-- WHY THIS TRIGGER IS NEEDED:
-- When holdings_snapshots are inserted, updated, or deleted, the portfolio's risk metrics
-- and correlations become stale because they depend on the portfolio composition.
-- Without this trigger, stale cache entries would be served until they expire naturally,
-- leading to incorrect risk calculations being displayed to users.
--
-- WHAT IT DOES:
-- This trigger automatically marks cache entries as 'stale' whenever holdings change,
-- ensuring that the background jobs will recalculate risk metrics on the next run.
-- This maintains cache freshness while avoiding expensive inline recalculations.
--
-- HOW IT MAINTAINS CACHE FRESHNESS:
-- 1. Detects any INSERT, UPDATE, or DELETE on holdings_snapshots table
-- 2. Extracts the account_id from the affected row (NEW or OLD)
-- 3. Joins through accounts table to find the associated portfolio_id
-- 4. Marks all portfolio_risk_cache entries for that portfolio as 'stale'
-- 5. Marks all portfolio_correlations_cache entries for that portfolio as 'stale'
-- 6. Background jobs will detect stale entries and recalculate on their schedule

-- Create the trigger function
-- This function invalidates cache entries when holdings change
CREATE OR REPLACE FUNCTION invalidate_portfolio_risk_cache()
RETURNS TRIGGER AS $$
DECLARE
    affected_account_id UUID;
    affected_portfolio_id UUID;
BEGIN
    -- Determine which account_id was affected
    -- For INSERT or UPDATE, use NEW; for DELETE, use OLD
    affected_account_id := COALESCE(NEW.account_id, OLD.account_id);

    -- Find the portfolio_id by joining through accounts table
    SELECT portfolio_id INTO affected_portfolio_id
    FROM accounts
    WHERE id = affected_account_id;

    -- If we found a portfolio_id, invalidate its caches
    IF affected_portfolio_id IS NOT NULL THEN
        -- Invalidate portfolio_risk_cache entries
        -- Set status to 'stale' and update the timestamp
        UPDATE portfolio_risk_cache
        SET calculation_status = 'stale',
            updated_at = NOW()
        WHERE portfolio_id = affected_portfolio_id
          AND calculation_status != 'stale';  -- Only update if not already stale

        -- Invalidate portfolio_correlations_cache entries
        -- Set status to 'stale' and update the timestamp
        UPDATE portfolio_correlations_cache
        SET calculation_status = 'stale',
            updated_at = NOW()
        WHERE portfolio_id = affected_portfolio_id
          AND calculation_status != 'stale';  -- Only update if not already stale
    END IF;

    -- Return the appropriate row for the trigger
    -- For INSERT/UPDATE return NEW, for DELETE return OLD
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Add function comment for documentation
COMMENT ON FUNCTION invalidate_portfolio_risk_cache() IS
'Trigger function that invalidates portfolio risk and correlation caches when holdings change. '
'Called automatically on INSERT, UPDATE, or DELETE of holdings_snapshots. '
'Marks cache entries as stale so background jobs will recalculate on next run.';

-- Drop the trigger if it already exists (for idempotent migrations)
DROP TRIGGER IF EXISTS trigger_invalidate_risk_on_holdings_change ON holdings_snapshots;

-- Create the trigger on holdings_snapshots table
-- AFTER ensures the holding change is committed before cache invalidation
-- FOR EACH ROW ensures every affected holding triggers cache invalidation
CREATE TRIGGER trigger_invalidate_risk_on_holdings_change
    AFTER INSERT OR UPDATE OR DELETE ON holdings_snapshots
    FOR EACH ROW
    EXECUTE FUNCTION invalidate_portfolio_risk_cache();

-- Add trigger comment for documentation
COMMENT ON TRIGGER trigger_invalidate_risk_on_holdings_change ON holdings_snapshots IS
'Automatically invalidates portfolio risk and correlation caches when holdings change. '
'Ensures cache entries are marked stale so background jobs recalculate on next scheduled run. '
'Fires AFTER INSERT, UPDATE, or DELETE to ensure data consistency.';

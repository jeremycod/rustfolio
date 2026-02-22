-- Fix threshold column types from DECIMAL to DOUBLE PRECISION
-- This ensures compatibility with Rust f64 type (FLOAT8 in SQL)

-- Drop views that reference the columns
DROP VIEW IF EXISTS active_alert_rules;
DROP VIEW IF EXISTS recent_alerts;

-- Alter column types
ALTER TABLE alert_rules ALTER COLUMN threshold TYPE DOUBLE PRECISION;
ALTER TABLE alert_history ALTER COLUMN threshold TYPE DOUBLE PRECISION;
ALTER TABLE alert_history ALTER COLUMN actual_value TYPE DOUBLE PRECISION;

-- Recreate views
CREATE OR REPLACE VIEW active_alert_rules AS
SELECT *
FROM alert_rules
WHERE enabled = TRUE;

CREATE OR REPLACE VIEW recent_alerts AS
SELECT *
FROM alert_history
WHERE triggered_at >= NOW() - INTERVAL '7 days'
ORDER BY triggered_at DESC;

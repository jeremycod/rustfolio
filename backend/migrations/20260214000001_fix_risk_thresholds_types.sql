-- Fix risk threshold column types from NUMERIC to DOUBLE PRECISION
-- NUMERIC doesn't map directly to f64 in Rust, causing decoding errors

ALTER TABLE risk_threshold_settings
    ALTER COLUMN volatility_warning_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN volatility_critical_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN drawdown_warning_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN drawdown_critical_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN beta_warning_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN beta_critical_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN risk_score_warning_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN risk_score_critical_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN var_warning_threshold TYPE DOUBLE PRECISION,
    ALTER COLUMN var_critical_threshold TYPE DOUBLE PRECISION;

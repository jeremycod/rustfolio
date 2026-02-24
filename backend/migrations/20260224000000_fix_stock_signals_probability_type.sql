-- Fix probability column type mismatch
-- Change from NUMERIC(5,4) to DOUBLE PRECISION to match Rust f64 type
-- This fixes the cache decoding error: "Rust type `f64` (as SQL type `FLOAT8`) is not compatible with SQL type `NUMERIC`"

ALTER TABLE stock_signals
    ALTER COLUMN probability TYPE DOUBLE PRECISION;

-- Add check constraint for valid probability range (0.0 to 1.0)
ALTER TABLE stock_signals
    DROP CONSTRAINT IF EXISTS stock_signals_probability_check,
    ADD CONSTRAINT stock_signals_probability_check CHECK (probability >= 0.0 AND probability <= 1.0);

-- Add comment explaining the fix
COMMENT ON COLUMN stock_signals.probability IS 'Probability score from 0.0 to 1.0 (DOUBLE PRECISION for Rust f64 compatibility)';

-- Add Payment Frequency Support for Liabilities
-- Migration: Adds payment_frequency column to support bi-weekly, weekly, and monthly payments

-- Add payment_frequency column with default 'monthly' for existing records
ALTER TABLE survey_liabilities
ADD COLUMN payment_frequency VARCHAR(20) DEFAULT 'monthly';

-- Add CHECK constraint for valid payment frequencies
ALTER TABLE survey_liabilities
ADD CONSTRAINT survey_liabilities_payment_frequency_valid CHECK (
    payment_frequency IN ('monthly', 'bi_weekly', 'weekly')
);

-- Update comment
COMMENT ON COLUMN survey_liabilities.payment_frequency IS 'Payment frequency: monthly (default), bi_weekly, weekly';

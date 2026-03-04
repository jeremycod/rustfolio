-- Add effective tax rate fields to income info for realistic net cash flow calculations
ALTER TABLE survey_income_info
ADD COLUMN IF NOT EXISTS effective_tax_rate DECIMAL(5,2),        -- primary person's effective rate (%)
ADD COLUMN IF NOT EXISTS spouse_effective_tax_rate DECIMAL(5,2); -- spouse effective rate (%)

-- Separate tax rate for investment income (dividends, interest)
-- which is taxed differently from employment income in most jurisdictions.
ALTER TABLE survey_income_info
ADD COLUMN IF NOT EXISTS investment_income_tax_rate DECIMAL(5,2),
ADD COLUMN IF NOT EXISTS spouse_investment_income_tax_rate DECIMAL(5,2);

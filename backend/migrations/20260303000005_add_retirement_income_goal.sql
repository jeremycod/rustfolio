-- Add desired retirement income fields to income_info table
ALTER TABLE survey_income_info
ADD COLUMN desired_annual_retirement_income DECIMAL(12,2),
ADD COLUMN retirement_income_needs_notes TEXT;

COMMENT ON COLUMN survey_income_info.desired_annual_retirement_income IS 'Desired annual income in retirement for gap analysis';
COMMENT ON COLUMN survey_income_info.retirement_income_needs_notes IS 'Notes about retirement income needs and goals';

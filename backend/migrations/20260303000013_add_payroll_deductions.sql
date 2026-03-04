-- Monthly payroll deductions (CPP/EI/pension contributions etc.) separate from income tax
ALTER TABLE survey_income_info
ADD COLUMN IF NOT EXISTS monthly_deductions DECIMAL(10,2) DEFAULT 0,
ADD COLUMN IF NOT EXISTS spouse_monthly_deductions DECIMAL(10,2) DEFAULT 0;

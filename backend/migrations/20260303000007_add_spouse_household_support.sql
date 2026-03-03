-- Spouse & Household Financial Extension
-- Extends the financial planning module to support couples

-- 1. Extend survey_personal_info with spouse/partner fields
ALTER TABLE survey_personal_info
ADD COLUMN IF NOT EXISTS has_spouse BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS spouse_name VARCHAR(255),
ADD COLUMN IF NOT EXISTS spouse_birth_year INT,
ADD COLUMN IF NOT EXISTS spouse_employment_status VARCHAR(50);

-- 2. Extend survey_income_info with spouse income fields
ALTER TABLE survey_income_info
ADD COLUMN IF NOT EXISTS spouse_gross_annual_income DECIMAL(15,2),
ADD COLUMN IF NOT EXISTS spouse_pay_frequency VARCHAR(20),
ADD COLUMN IF NOT EXISTS spouse_retirement_contribution_rate DECIMAL(5,2),
ADD COLUMN IF NOT EXISTS spouse_employer_match_rate DECIMAL(5,2);

-- 3. Extend survey_assets with ownership attribution
ALTER TABLE survey_assets
ADD COLUMN IF NOT EXISTS ownership VARCHAR(20) NOT NULL DEFAULT 'mine',
ADD COLUMN IF NOT EXISTS joint_split_percentage DECIMAL(5,2) DEFAULT 50.00;

-- 4. Extend survey_liabilities with ownership attribution
ALTER TABLE survey_liabilities
ADD COLUMN IF NOT EXISTS ownership VARCHAR(20) NOT NULL DEFAULT 'mine',
ADD COLUMN IF NOT EXISTS joint_split_percentage DECIMAL(5,2) DEFAULT 50.00;

-- 5. Create household expenses table for shared/attributed expenses
CREATE TABLE IF NOT EXISTS survey_household_expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    expense_category VARCHAR(50) NOT NULL,
    expense_type VARCHAR(20) NOT NULL DEFAULT 'shared', -- 'shared', 'mine', 'spouse'
    monthly_amount DECIMAL(10,2) NOT NULL,
    description VARCHAR(255),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_household_expenses_survey ON survey_household_expenses(survey_id);

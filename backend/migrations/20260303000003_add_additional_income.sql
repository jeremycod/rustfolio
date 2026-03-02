-- Add Additional Income Sources Table
-- Supports dividends, rental income, side income, etc.

CREATE TABLE survey_additional_income (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    income_type VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    monthly_amount DECIMAL(10,2) NOT NULL,
    is_recurring BOOLEAN DEFAULT true,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT survey_additional_income_type_valid CHECK (
        income_type IN (
            'dividends',
            'interest',
            'rental_income',
            'side_business',
            'pension',
            'social_security',
            'disability',
            'child_support',
            'alimony',
            'other'
        )
    )
);

-- Indexes
CREATE INDEX idx_survey_additional_income_survey ON survey_additional_income(survey_id);
CREATE INDEX idx_survey_additional_income_type ON survey_additional_income(survey_id, income_type);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_additional_income_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_additional_income_updated_at
    BEFORE UPDATE ON survey_additional_income
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_additional_income_updated_at();

COMMENT ON TABLE survey_additional_income IS 'Financial Planning: Additional income sources beyond employment';
COMMENT ON COLUMN survey_additional_income.income_type IS 'Type: dividends, interest, rental_income, side_business, pension, social_security, disability, child_support, alimony, other';
COMMENT ON COLUMN survey_additional_income.monthly_amount IS 'Monthly income amount (enter as monthly regardless of actual frequency)';
COMMENT ON COLUMN survey_additional_income.is_recurring IS 'Whether this is recurring income (true) or one-time (false)';

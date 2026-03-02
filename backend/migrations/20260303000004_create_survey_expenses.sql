-- Add Survey Expenses Table
-- Replace 70% expense estimate with actual expense tracking

CREATE TABLE survey_expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    expense_category VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    monthly_amount DECIMAL(10,2) NOT NULL,
    is_recurring BOOLEAN DEFAULT true,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT survey_expenses_category_valid CHECK (
        expense_category IN (
            'housing',           -- Rent/Mortgage (separate from debt)
            'utilities',         -- Electric, gas, water, internet
            'groceries',
            'transportation',    -- Gas, public transit, parking
            'insurance',         -- Health, life, home, auto
            'healthcare',        -- Medical, dental, prescriptions
            'childcare',
            'education',
            'entertainment',
            'dining_out',
            'subscriptions',     -- Netflix, Spotify, gym, etc.
            'personal_care',
            'clothing',
            'gifts_donations',
            'pet_care',
            'home_maintenance',
            'savings_investments', -- Non-retirement savings
            'miscellaneous',
            'other'
        )
    )
);

-- Indexes
CREATE INDEX idx_survey_expenses_survey ON survey_expenses(survey_id);
CREATE INDEX idx_survey_expenses_category ON survey_expenses(survey_id, expense_category);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_expenses_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_expenses_updated_at
    BEFORE UPDATE ON survey_expenses
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_expenses_updated_at();

COMMENT ON TABLE survey_expenses IS 'Financial Planning: Monthly expense tracking by category';
COMMENT ON COLUMN survey_expenses.expense_category IS 'Category: housing, utilities, groceries, transportation, insurance, healthcare, childcare, education, entertainment, dining_out, subscriptions, personal_care, clothing, gifts_donations, pet_care, home_maintenance, savings_investments, miscellaneous, other';
COMMENT ON COLUMN survey_expenses.monthly_amount IS 'Monthly expense amount';
COMMENT ON COLUMN survey_expenses.is_recurring IS 'Whether this is recurring expense (true) or one-time (false)';

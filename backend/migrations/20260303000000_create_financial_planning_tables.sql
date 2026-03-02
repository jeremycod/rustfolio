-- Financial Planning Module: Survey-Based Financial Planning Tables
-- Migration: Create tables for financial surveys, personal info, income,
--            assets, liabilities, goals, risk profile, and snapshot cache

-- ==============================================================================
-- TABLE: financial_surveys
-- Main entity tracking the overall survey status per user
-- ==============================================================================
CREATE TABLE financial_surveys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version INT NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,

    CONSTRAINT financial_surveys_status_valid CHECK (
        status IN ('draft', 'completed')
    )
);

-- Indexes for efficient querying
CREATE INDEX idx_financial_surveys_user ON financial_surveys(user_id);
CREATE INDEX idx_financial_surveys_user_status ON financial_surveys(user_id, status);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_financial_surveys_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER financial_surveys_updated_at
    BEFORE UPDATE ON financial_surveys
    FOR EACH ROW
    EXECUTE FUNCTION update_financial_surveys_updated_at();

COMMENT ON TABLE financial_surveys IS 'Financial Planning: Main survey entity tracking status per user';
COMMENT ON COLUMN financial_surveys.status IS 'Survey status: draft or completed';
COMMENT ON COLUMN financial_surveys.version IS 'Schema version for forward compatibility';

-- ==============================================================================
-- TABLE: survey_personal_info
-- Demographics and personal details
-- ==============================================================================
CREATE TABLE survey_personal_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    full_name VARCHAR(255),
    birth_year INT,
    marital_status VARCHAR(20),
    employment_status VARCHAR(50),
    dependents INT DEFAULT 0,
    contact_email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_survey_personal_info UNIQUE(survey_id)
);

-- Indexes
CREATE INDEX idx_survey_personal_info_survey ON survey_personal_info(survey_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_personal_info_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_personal_info_updated_at
    BEFORE UPDATE ON survey_personal_info
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_personal_info_updated_at();

COMMENT ON TABLE survey_personal_info IS 'Financial Planning: Personal demographic information';
COMMENT ON COLUMN survey_personal_info.marital_status IS 'Marital status: single, married, divorced, widowed';
COMMENT ON COLUMN survey_personal_info.employment_status IS 'Employment: employed, self_employed, unemployed, retired, student';

-- ==============================================================================
-- TABLE: survey_income_info
-- Income and retirement contribution details
-- ==============================================================================
CREATE TABLE survey_income_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    gross_annual_income DECIMAL(15,2),
    pay_frequency VARCHAR(20),
    retirement_contribution_rate DECIMAL(5,2),
    employer_match_rate DECIMAL(5,2),
    planned_retirement_age INT,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_survey_income_info UNIQUE(survey_id)
);

-- Indexes
CREATE INDEX idx_survey_income_info_survey ON survey_income_info(survey_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_income_info_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_income_info_updated_at
    BEFORE UPDATE ON survey_income_info
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_income_info_updated_at();

COMMENT ON TABLE survey_income_info IS 'Financial Planning: Income and retirement contribution details';
COMMENT ON COLUMN survey_income_info.pay_frequency IS 'Pay frequency: annual, monthly, bi_weekly, weekly';
COMMENT ON COLUMN survey_income_info.retirement_contribution_rate IS 'Percentage of income contributed to retirement (0-100)';
COMMENT ON COLUMN survey_income_info.employer_match_rate IS 'Employer match percentage (0-100)';

-- ==============================================================================
-- TABLE: survey_assets
-- Asset inventory with type categorization
-- ==============================================================================
CREATE TABLE survey_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    asset_type VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    current_value DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT survey_assets_type_valid CHECK (
        asset_type IN ('liquid', 'investment', 'retirement', 'real_estate', 'other')
    )
);

-- Indexes
CREATE INDEX idx_survey_assets_survey ON survey_assets(survey_id);
CREATE INDEX idx_survey_assets_type ON survey_assets(survey_id, asset_type);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_assets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_assets_updated_at
    BEFORE UPDATE ON survey_assets
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_assets_updated_at();

COMMENT ON TABLE survey_assets IS 'Financial Planning: Asset inventory with categorization';
COMMENT ON COLUMN survey_assets.asset_type IS 'Asset type: liquid, investment, retirement, real_estate, other';

-- ==============================================================================
-- TABLE: survey_liabilities
-- Debt and liability tracking
-- ==============================================================================
CREATE TABLE survey_liabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    liability_type VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    balance DECIMAL(15,2) NOT NULL,
    interest_rate DECIMAL(5,2),
    monthly_payment DECIMAL(10,2),
    linked_asset_id UUID REFERENCES survey_assets(id) ON DELETE SET NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT survey_liabilities_type_valid CHECK (
        liability_type IN ('mortgage', 'student_loan', 'auto_loan', 'credit_card', 'other')
    )
);

-- Indexes
CREATE INDEX idx_survey_liabilities_survey ON survey_liabilities(survey_id);
CREATE INDEX idx_survey_liabilities_type ON survey_liabilities(survey_id, liability_type);
CREATE INDEX idx_survey_liabilities_linked_asset ON survey_liabilities(linked_asset_id)
    WHERE linked_asset_id IS NOT NULL;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_liabilities_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_liabilities_updated_at
    BEFORE UPDATE ON survey_liabilities
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_liabilities_updated_at();

COMMENT ON TABLE survey_liabilities IS 'Financial Planning: Debt and liability tracking';
COMMENT ON COLUMN survey_liabilities.liability_type IS 'Liability type: mortgage, student_loan, auto_loan, credit_card, other';
COMMENT ON COLUMN survey_liabilities.linked_asset_id IS 'Optional link to an asset (e.g., mortgage linked to real estate)';

-- ==============================================================================
-- TABLE: survey_goals
-- Financial goals with progress tracking
-- ==============================================================================
CREATE TABLE survey_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    goal_type VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    target_amount DECIMAL(15,2),
    current_savings DECIMAL(15,2) DEFAULT 0,
    target_date DATE,
    priority VARCHAR(20),
    currency VARCHAR(3) DEFAULT 'USD',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT survey_goals_type_valid CHECK (
        goal_type IN ('retirement', 'home_purchase', 'education', 'travel', 'other')
    ),
    CONSTRAINT survey_goals_priority_valid CHECK (
        priority IS NULL OR priority IN ('high', 'medium', 'low')
    )
);

-- Indexes
CREATE INDEX idx_survey_goals_survey ON survey_goals(survey_id);
CREATE INDEX idx_survey_goals_priority ON survey_goals(survey_id, priority);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_goals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_goals_updated_at
    BEFORE UPDATE ON survey_goals
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_goals_updated_at();

COMMENT ON TABLE survey_goals IS 'Financial Planning: Financial goals with progress tracking';
COMMENT ON COLUMN survey_goals.goal_type IS 'Goal type: retirement, home_purchase, education, travel, other';
COMMENT ON COLUMN survey_goals.priority IS 'Priority level: high, medium, low';

-- ==============================================================================
-- TABLE: survey_risk_profile
-- Risk tolerance assessment
-- ==============================================================================
CREATE TABLE survey_risk_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    risk_tolerance VARCHAR(20),
    investment_experience VARCHAR(20),
    time_horizon_years INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_survey_risk_profile UNIQUE(survey_id),
    CONSTRAINT survey_risk_tolerance_valid CHECK (
        risk_tolerance IS NULL OR risk_tolerance IN ('conservative', 'moderate', 'aggressive')
    ),
    CONSTRAINT survey_investment_experience_valid CHECK (
        investment_experience IS NULL OR investment_experience IN ('beginner', 'intermediate', 'advanced')
    )
);

-- Indexes
CREATE INDEX idx_survey_risk_profile_survey ON survey_risk_profile(survey_id);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_survey_risk_profile_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER survey_risk_profile_updated_at
    BEFORE UPDATE ON survey_risk_profile
    FOR EACH ROW
    EXECUTE FUNCTION update_survey_risk_profile_updated_at();

COMMENT ON TABLE survey_risk_profile IS 'Financial Planning: Risk tolerance assessment';
COMMENT ON COLUMN survey_risk_profile.risk_tolerance IS 'Risk tolerance: conservative, moderate, aggressive';
COMMENT ON COLUMN survey_risk_profile.investment_experience IS 'Investment experience: beginner, intermediate, advanced';

-- ==============================================================================
-- TABLE: financial_snapshots
-- Cached calculation results for the financial summary
-- ==============================================================================
CREATE TABLE financial_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    survey_id UUID NOT NULL REFERENCES financial_surveys(id) ON DELETE CASCADE,
    net_worth DECIMAL(15,2),
    total_assets DECIMAL(15,2),
    total_liabilities DECIMAL(15,2),
    monthly_cash_flow DECIMAL(10,2),
    projected_retirement_income DECIMAL(10,2),
    snapshot_data JSONB,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_financial_snapshots_survey ON financial_snapshots(survey_id);
CREATE INDEX idx_financial_snapshots_generated ON financial_snapshots(survey_id, generated_at DESC);

COMMENT ON TABLE financial_snapshots IS 'Financial Planning: Cached calculation results for financial summary';
COMMENT ON COLUMN financial_snapshots.snapshot_data IS 'Full calculated snapshot as JSON (goal progress, projections, recommendations)';
COMMENT ON COLUMN financial_snapshots.net_worth IS 'Total assets minus total liabilities';
COMMENT ON COLUMN financial_snapshots.monthly_cash_flow IS 'Estimated monthly disposable income';
COMMENT ON COLUMN financial_snapshots.projected_retirement_income IS 'Projected monthly income at retirement';

-- Add owner field to additional income to support spouse/household attribution
ALTER TABLE survey_additional_income
ADD COLUMN IF NOT EXISTS owner VARCHAR(20) NOT NULL DEFAULT 'mine'; -- 'mine' or 'spouse'

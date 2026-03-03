-- Add owner field to goals to support spouse/household attribution
ALTER TABLE survey_goals
ADD COLUMN IF NOT EXISTS owner VARCHAR(20) NOT NULL DEFAULT 'mine'; -- 'mine', 'spouse', 'joint'

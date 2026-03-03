-- Link survey assets to portfolio accounts for automatic value refresh
ALTER TABLE survey_assets
ADD COLUMN IF NOT EXISTS linked_account_id UUID REFERENCES accounts(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_survey_assets_linked_account ON survey_assets(linked_account_id)
WHERE linked_account_id IS NOT NULL;

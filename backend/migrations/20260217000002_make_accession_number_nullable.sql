-- Make accession_number nullable since we're creating placeholder transactions
-- without always having the parent sec_filing record
ALTER TABLE insider_transactions
ALTER COLUMN accession_number DROP NOT NULL;

ALTER TABLE material_events
ALTER COLUMN accession_number DROP NOT NULL;

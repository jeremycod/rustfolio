-- Add unique constraint for insider transactions to support ON CONFLICT
-- A unique transaction is identified by ticker, date, and reporting person
ALTER TABLE insider_transactions
ADD CONSTRAINT insider_transactions_unique
UNIQUE (ticker, transaction_date, reporting_person);

-- Also add unique constraint for material events to support ON CONFLICT
ALTER TABLE material_events
ADD CONSTRAINT material_events_unique
UNIQUE (ticker, event_date, event_type);

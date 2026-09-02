-- Add name column for ledger entries
-- Add total column representing gross total (bank_balance + income)
ALTER TABLE ledgers
    ADD COLUMN name VARCHAR(255),
    ADD COLUMN total NUMERIC GENERATED ALWAYS AS (bank_balance + income) STORED;

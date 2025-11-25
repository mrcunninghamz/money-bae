-- Drop triggers
DROP TRIGGER IF EXISTS trigger_incomes_totals ON incomes;
DROP TRIGGER IF EXISTS trigger_ledger_bills_totals ON ledger_bills;

-- Drop function
DROP FUNCTION IF EXISTS recalculate_ledger_totals();

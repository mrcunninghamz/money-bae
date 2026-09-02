-- Function to recalculate ledger totals
CREATE OR REPLACE FUNCTION recalculate_ledger_totals()
RETURNS TRIGGER AS $$
DECLARE
    target_ledger_id INTEGER;
BEGIN
    -- Determine which ledger(s) to update
    IF TG_OP = 'DELETE' THEN
        target_ledger_id := OLD.ledger_id;
    ELSE
        target_ledger_id := NEW.ledger_id;
    END IF;

    -- Update the ledger totals
    UPDATE ledgers
    SET
        income = COALESCE((
            SELECT SUM(amount)
            FROM incomes
            WHERE ledger_id = target_ledger_id
        ), 0),
        expenses = COALESCE((
            SELECT SUM(amount)
            FROM ledger_bills
            WHERE ledger_id = target_ledger_id
        ), 0)
    WHERE id = target_ledger_id;

    -- If UPDATE changed ledger_id, recalculate old ledger too
    IF TG_OP = 'UPDATE' AND OLD.ledger_id IS DISTINCT FROM NEW.ledger_id THEN
        UPDATE ledgers
        SET
            income = COALESCE((
                SELECT SUM(amount)
                FROM incomes
                WHERE ledger_id = OLD.ledger_id
            ), 0),
            expenses = COALESCE((
                SELECT SUM(amount)
                FROM ledger_bills
                WHERE ledger_id = OLD.ledger_id
            ), 0)
        WHERE id = OLD.ledger_id;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Triggers for incomes table
CREATE TRIGGER trigger_incomes_totals
AFTER INSERT OR UPDATE OR DELETE ON incomes
FOR EACH ROW
EXECUTE FUNCTION recalculate_ledger_totals();

-- Triggers for ledger_bills table
CREATE TRIGGER trigger_ledger_bills_totals
AFTER INSERT OR UPDATE OR DELETE ON ledger_bills
FOR EACH ROW
EXECUTE FUNCTION recalculate_ledger_totals();

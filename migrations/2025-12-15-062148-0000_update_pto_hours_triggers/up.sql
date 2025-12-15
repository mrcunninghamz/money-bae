-- Function to update PTO hours aggregates
CREATE OR REPLACE FUNCTION update_pto_hours() RETURNS TRIGGER AS $$
BEGIN
    -- Update hours_planned and hours_remaining for the affected PTO
    UPDATE ptos
    SET 
        hours_planned = (
            SELECT COALESCE(SUM(hours), 0)
            FROM pto_plan
            WHERE pto_id = COALESCE(NEW.pto_id, OLD.pto_id)
            AND status IN ('Planned', 'Requested', 'Approved')
        ),
        hours_used = (
            SELECT COALESCE(SUM(hours), 0)
            FROM pto_plan
            WHERE pto_id = COALESCE(NEW.pto_id, OLD.pto_id)
            AND status = 'Completed'
        ),
        hours_remaining = available_hours + prev_year_hours - (
            SELECT COALESCE(SUM(hours), 0)
            FROM pto_plan
            WHERE pto_id = COALESCE(NEW.pto_id, OLD.pto_id)
            AND status = 'Completed'
        )
    WHERE id = COALESCE(NEW.pto_id, OLD.pto_id);
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on INSERT
CREATE TRIGGER pto_plan_after_insert
AFTER INSERT ON pto_plan
FOR EACH ROW
EXECUTE FUNCTION update_pto_hours();

-- Trigger on UPDATE
CREATE TRIGGER pto_plan_after_update
AFTER UPDATE ON pto_plan
FOR EACH ROW
EXECUTE FUNCTION update_pto_hours();

-- Trigger on DELETE
CREATE TRIGGER pto_plan_after_delete
AFTER DELETE ON pto_plan
FOR EACH ROW
EXECUTE FUNCTION update_pto_hours();

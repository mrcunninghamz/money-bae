-- This file should undo anything in `up.sql`
ALTER TABLE ledgers
    DROP COLUMN total,
    DROP COLUMN name;


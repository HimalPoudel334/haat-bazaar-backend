-- This file should undo anything in `up.sql`
ALTER TABLE payments DROP COLUMN tendered;
ALTER TABLE payments DROP COLUMN change;
ALTER TABLE payments DROP COLUMN discount;

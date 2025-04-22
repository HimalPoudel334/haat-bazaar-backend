-- Your SQL goes here
ALTER TABLE orders ADD COLUMN status TEXT NOT NULL DEFAULT 'Pending';

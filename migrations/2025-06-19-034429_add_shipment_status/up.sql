-- Your SQL goes here
ALTER TABLE shipments ADD COLUMN status TEXT NOT NULL DEFAULT 'Pending';

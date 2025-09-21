-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS product_ratings;
DROP TRIGGER IF EXISTS update_product_ratings_updated_at;

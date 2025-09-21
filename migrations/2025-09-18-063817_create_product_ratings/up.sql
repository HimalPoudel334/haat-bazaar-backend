-- Your SQL goes here
CREATE TABLE product_ratings (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    product_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    rating DOUBLE NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Product Ratings Indexes
CREATE UNIQUE INDEX idx_product_ratings_uuid ON product_ratings(uuid);
CREATE INDEX idx_product_ratings_product_id ON product_ratings(product_id);
CREATE INDEX idx_product_ratings_user_id ON product_ratings(user_id);
CREATE INDEX idx_product_ratings_rating ON product_ratings(rating);
CREATE INDEX idx_product_ratings_created_at ON product_ratings(created_at);
CREATE INDEX idx_product_ratings_updated_at ON product_ratings(updated_at);
CREATE UNIQUE INDEX idx_product_ratings_product_user ON product_ratings(product_id, user_id);
CREATE INDEX idx_product_ratings_product_rating ON product_ratings(product_id, rating);

CREATE TRIGGER update_product_ratings_updated_at
AFTER UPDATE ON product_ratings
FOR EACH ROW
BEGIN
  UPDATE product_ratings
  SET updated_at = CURRENT_TIMESTAMP
  WHERE id = OLD.id;
END;

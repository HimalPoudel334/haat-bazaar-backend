-- Your SQL goes here
CREATE TABLE IF NOT EXISTS product_images (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  image_name TEXT NOT NULL,
  product_id INTEGER NOT NULL,

  FOREIGN KEY(product_id) REFERENCES products(id)
);

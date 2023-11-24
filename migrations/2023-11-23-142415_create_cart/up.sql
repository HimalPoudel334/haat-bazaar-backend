-- Your SQL goes here
CREATE TABLE IF NOT EXISTS carts (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  product_id INTEGER NOT NULL,
  customer_id INTEGER NOT NULL,

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(customer_id) REFERENCES customers(id)
)

-- Your SQL goes here
CREATE TABLE IF NOT EXISTS order_items (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  product_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL,
  quantity DOUBLE NOT NULL,
  price DOUBLE NOT NULL,

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);

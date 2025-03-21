-- Your SQL goes here
CREATE TABLE IF NOT EXISTS shipments (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  ship_date TEXT NOT NULL,
  address TEXT NOT NULL,
  city TEXT NOT NULL,
  state TEXT NOT NULL,
  country TEXT NOT NULL,
  zip_code TEXT NOT NULL,
  order_id INTEGER NOT NULL,
  
  FOREIGN KEY(order_id) REFERENCES orders(id)
);

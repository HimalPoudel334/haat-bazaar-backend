-- Your SQL goes here
CREATE TABLE IF NOT EXISTS shipments (
  id UNSIGNED INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  ship_date TEXT NOT NULL,
  address TEXT NOT NULL,
  city TEXT NOT NULL,
  state TEXT NOT NULL,
  country TEXT NOT NULL,
  zip_code TEXT NOT NULL,
  customer_id UNSIGNED INTEGER NOT NULL,
  order_id UNSIGNED INTEGER NOT NULL,
  
  FOREIGN KEY(customer_id) REFERENCES customers(id),
  FOREIGN KEY(order_id) REFERENCES orders(id),
);

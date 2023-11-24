-- Your SQL goes here
CREATE TABLE IF NOT EXISTS payments (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  pay_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  amount DOUBLE NOT NULL,
  payment_method TEXT NOT NULL,
  customer_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL,

  FOREIGN KEY(customer_id) REFERENCES customers(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);

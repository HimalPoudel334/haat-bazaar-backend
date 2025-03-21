-- Your SQL goes here
CREATE TABLE IF NOT EXISTS invoices (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  invoice_number INTEGER NOT NULL DEFAULT 1,
  invoice_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  sub_total DOUBLE NOT NULL,
  vat_percent DOUBLE NOT NULL,
  vat_amount DOUBLE NOT NULL DEFAULT 0,
  net_amount DOUBLE NOT NULL,
  order_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,
  payment_id INTEGER NOT NULL,

  FOREIGN KEY(order_id) REFERENCES orders(id),
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(payment_id) REFERENCES payments(id)
);

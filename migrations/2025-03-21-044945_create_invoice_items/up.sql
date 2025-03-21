-- Your SQL goes here
CREATE TABLE IF NOT EXISTS invoice_items (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  quantity DOUBLE NOT NULL,
  unit_price DOUBLE NOT NULL,
  discount_percent DOUBLE NOT NULL DEFAULT 0,
  discount_amount DOUBLE NOT NULL,
  total DOUBLE NOT NULL,
  product_id INTEGER NOT NULL,
  invoice_id INTEGER NOT NULL,

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(invoice_id) REFERENCES invoices(id)

);

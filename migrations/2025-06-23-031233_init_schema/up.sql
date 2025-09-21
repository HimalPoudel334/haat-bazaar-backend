CREATE TABLE categories (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
CREATE TABLE sqlite_sequence(name,seq);
CREATE TABLE products (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  image TEXT NOT NULL,
  price DOUBLE NOT NULL,
  previous_price DOUBLE NOT NULL,
  unit TEXT NOT NULL,
  unit_change DOUBLE NOT NULL,
  stock DOUBLE NOT NULL,
  category_id INTEGER NOT NULL,

  FOREIGN KEY(category_id) REFERENCES categories(id)
);
CREATE TABLE users (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  phone_number TEXT NOT NULL,
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL, 
  user_type TEXT NOT NULL
, location TEXT, nearest_landmark TEXT);
CREATE TABLE carts (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  product_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,
  quantity DOUBLE NOT NULL,
  sku TEXT NOT NULL,
  created_on TEXT NOT NULL, discount double not null default 0.0,

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(user_id) REFERENCES users(id)
);
CREATE TABLE payments (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  pay_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  amount DOUBLE NOT NULL,
  payment_method TEXT NOT NULL,
  user_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL, transaction_id TEXT NOT NULL, tendered DOUBLE NOT NULL DEFAULT 0, change DOUBLE NOT NULL DEFAULT 0, discount DOUBLE NOT NULL DEFAULT 0, status text not null default 'Pending', service_charge DOUBLE NOT NULL DEFAULT 0.0, refunded BOOLEAN NOT NULL DEFAULT 0,

  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE invoices (
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
CREATE TABLE invoice_items (
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
CREATE TABLE product_images (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  image_name TEXT NOT NULL,
  product_id INTEGER NOT NULL,

  FOREIGN KEY(product_id) REFERENCES products(id)
);
CREATE TABLE IF NOT EXISTS "order_items" (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  product_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL,
  quantity DOUBLE NOT NULL,
  price DOUBLE NOT NULL,
  discount DOUBLE NOT NULL DEFAULT 0.0,
  amount DOUBLE NOT NULL,  -- ✅ new column

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE __diesel_schema_migrations (
       version VARCHAR(50) PRIMARY KEY NOT NULL,
       run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS "orders" (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  created_on TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  fulfilled_on TEXT NOT NULL,
  delivery_charge DOUBLE NOT NULL,
  delivery_location TEXT NOT NULL,
  delivery_status TEXT NOT NULL,
  total_price DOUBLE NOT NULL,
  user_id INTEGER NOT NULL,
  quantity DOUBLE NOT NULL,
  status TEXT NOT NULL DEFAULT 'Pending',
  discount DOUBLE NOT NULL DEFAULT 0.0,
  amount DOUBLE NOT NULL,

  FOREIGN KEY(user_id) REFERENCES users(id)
);
CREATE TABLE IF NOT EXISTS "shipments" (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL,
    ship_date TEXT NOT NULL,
    address TEXT NOT NULL,
    city TEXT NOT NULL,
    state TEXT NOT NULL,
    country TEXT NOT NULL,
    zip_code TEXT NOT NULL,
    order_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    assigned_to INT,
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (assigned_to) REFERENCES users(id)
);


-- Indexes
-- UUID Indexes (Critical for lookups by UUID)
CREATE UNIQUE INDEX idx_categories_uuid ON categories(uuid);
CREATE UNIQUE INDEX idx_products_uuid ON products(uuid);
CREATE UNIQUE INDEX idx_users_uuid ON users(uuid);
CREATE UNIQUE INDEX idx_carts_uuid ON carts(uuid);
CREATE UNIQUE INDEX idx_payments_uuid ON payments(uuid);
CREATE UNIQUE INDEX idx_invoices_uuid ON invoices(uuid);
CREATE UNIQUE INDEX idx_invoice_items_uuid ON invoice_items(uuid);
CREATE UNIQUE INDEX idx_product_images_uuid ON product_images(uuid);
CREATE UNIQUE INDEX idx_order_items_uuid ON order_items(uuid);
CREATE UNIQUE INDEX idx_orders_uuid ON orders(uuid);
CREATE UNIQUE INDEX idx_shipments_uuid ON shipments(uuid);

-- Foreign Key Indexes (Critical for JOIN performance)
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_carts_product_id ON carts(product_id);
CREATE INDEX idx_carts_user_id ON carts(user_id);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_order_id ON payments(order_id);
CREATE INDEX idx_invoices_order_id ON invoices(order_id);
CREATE INDEX idx_invoices_user_id ON invoices(user_id);
CREATE INDEX idx_invoices_payment_id ON invoices(payment_id);
CREATE INDEX idx_invoice_items_product_id ON invoice_items(product_id);
CREATE INDEX idx_invoice_items_invoice_id ON invoice_items(invoice_id);
CREATE INDEX idx_product_images_product_id ON product_images(product_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_shipments_order_id ON shipments(order_id);
CREATE INDEX idx_shipments_assigned_to ON shipments(assigned_to);

-- Composite Indexes for Common Query Patterns
CREATE UNIQUE INDEX idx_carts_user_product ON carts(user_id, product_id); -- Prevent duplicate cart items
CREATE INDEX idx_orders_user_status ON orders(user_id, status); -- User's orders by status
CREATE INDEX idx_orders_status_created ON orders(status, created_on); -- Orders by status and date
CREATE INDEX idx_payments_status_date ON payments(status, pay_date); -- Payments by status and date
CREATE INDEX idx_shipments_status_date ON shipments(status, ship_date); -- Shipments by status and date

-- Email Index for User Lookups (already unique, but explicit index for performance)
CREATE INDEX idx_users_email ON users(email);

-- Transaction ID for Payment Lookups
CREATE INDEX idx_payments_transaction_id ON payments(transaction_id);

-- Invoice Number for Invoice Lookups
CREATE INDEX idx_invoices_invoice_number ON invoices(invoice_number);

-- Date-based Indexes for Reporting
CREATE INDEX idx_orders_created_on ON orders(created_on);
CREATE INDEX idx_orders_fulfilled_on ON orders(fulfilled_on);
CREATE INDEX idx_payments_pay_date ON payments(pay_date);
CREATE INDEX idx_invoices_invoice_date ON invoices(invoice_date);

-- Status Indexes for Filtering
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_delivery_status ON orders(delivery_status);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_shipments_status ON shipments(status);

-- Product Search and Filtering
CREATE INDEX idx_products_name ON products(name); -- For product name searches
CREATE INDEX idx_products_price ON products(price); -- For price filtering
CREATE INDEX idx_products_stock ON products(stock); -- For stock availability

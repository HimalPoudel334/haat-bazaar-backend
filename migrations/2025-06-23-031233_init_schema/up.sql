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
  created_on TEXT NOT NULL,
  discount double NOT NULL DEFAULT 0.0,

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
  order_id INTEGER NOT NULL, transaction_id TEXT NOT NULL, tendered DOUBLE NOT NULL DEFAULT 0, change DOUBLE NOT NULL DEFAULT 0, discount DOUBLE NOT NULL DEFAULT 0, status text not null default 'Pending',

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
CREATE TABLE shipments (
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
CREATE TABLE orders (
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
  amount DOUBLE  NOT NULL, 

  FOREIGN KEY(user_id) REFERENCES users(id)
);
CREATE TABLE order_items (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  product_id INTEGER NOT NULL,
  order_id INTEGER NOT NULL,
  quantity DOUBLE NOT NULL,
  price DOUBLE NOT NULL,
  discount DOUBLE NOT NULL DEFAULT 0.0,
  amount DOUBLE NOT NULL, 

  FOREIGN KEY(product_id) REFERENCES products(id),
  FOREIGN KEY(order_id) REFERENCES orders(id)
);

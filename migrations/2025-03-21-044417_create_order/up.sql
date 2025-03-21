-- Your SQL goes here
CREATE TABLE IF NOT EXISTS orders (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  created_on TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  fulfilled_on TEXT NOT NULL,
  delivery_charge DOUBLE NOT NULL,
  delivery_location TEXT NOT NULL,
  delivery_status TEXT NOT NULL,
  total_price DOUBLE NOT NULL,
  user_id INTEGER NOT NULL,

  FOREIGN KEY(user_id) REFERENCES users(id)
);

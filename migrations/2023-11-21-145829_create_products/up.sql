-- Your SQL goes here
CREATE TABLE IF NOT EXISTS products (
  id UNSIGNED INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  image TEXT NOT NULL,
  price REAL NOT NULL,
  previous_price REAL NOT NULL,
  unit TEXT NOT NULL,
  unit_change REAL NOT NULL,
  stock REAL NOT NULL,
  category_id INTEGER NOT NULL,

  FOREIGN KEY(category_id) REFERENCES categories(id)
);

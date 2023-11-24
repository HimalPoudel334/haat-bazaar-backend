-- Your SQL goes here
CREATE TABLE IF NOT EXISTS products (
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

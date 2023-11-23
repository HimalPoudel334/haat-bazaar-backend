-- Your SQL goes here
CREATE TABLE IF NOT EXISTS customers (
  id UNSIGNED INT NOT NULL PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  phone_number TEXT NOT NULL,
  password TEXT NOT NULL,
);

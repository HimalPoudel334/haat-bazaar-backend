-- Your SQL goes here
CREATE TABLE admin_devices (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  uuid TEXT NOT NULL UNIQUE,
  user_id INTEGER NOT NULL UNIQUE,
  fcm_token TEXT NOT NULL UNIQUE,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add trigger for updated_at (SQLite specific)
CREATE TRIGGER update_admin_devices_updated_at
AFTER UPDATE ON admin_devices
FOR EACH ROW
BEGIN
  UPDATE admin_devices SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

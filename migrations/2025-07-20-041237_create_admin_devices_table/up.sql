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

-- Indexes for admin_devices table
CREATE UNIQUE INDEX idx_admin_devices_uuid ON admin_devices(uuid);
CREATE UNIQUE INDEX idx_admin_devices_user_id ON admin_devices(user_id);
CREATE UNIQUE INDEX idx_admin_devices_fcm_token ON admin_devices(fcm_token);
CREATE INDEX idx_admin_devices_created_at ON admin_devices(created_at);
CREATE INDEX idx_admin_devices_updated_at ON admin_devices(updated_at);

-- Add trigger for updated_at (SQLite specific)
CREATE TRIGGER update_admin_devices_updated_at
AFTER UPDATE ON admin_devices
FOR EACH ROW
BEGIN
  UPDATE admin_devices SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

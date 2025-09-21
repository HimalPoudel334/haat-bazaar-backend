-- Your SQL goes here
CREATE TABLE refresh_tokens (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
  uuid TEXT NOT NULL UNIQUE,
  token TEXT NOT NULL,
  user_id INTEGER NOT NULL,
  expires_on TEXT NOT NULL,

  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- Refresh Tokens Indexes

-- UUID Index (for API lookups by UUID)
CREATE UNIQUE INDEX idx_refresh_tokens_uuid ON refresh_tokens(uuid);

-- Token Index (CRITICAL - for token validation/lookup)
CREATE UNIQUE INDEX idx_refresh_tokens_token ON refresh_tokens(token);

-- Foreign Key Index (for JOINs with users table)
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Expiration Index (for cleanup operations)
CREATE INDEX idx_refresh_tokens_expires_on ON refresh_tokens(expires_on);

-- Composite Index (user + expiration - for getting user's active tokens)
CREATE INDEX idx_refresh_tokens_user_expires ON refresh_tokens(user_id, expires_on);

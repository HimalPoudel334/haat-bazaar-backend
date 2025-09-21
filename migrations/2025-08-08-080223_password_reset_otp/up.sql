-- Your SQL goes here

CREATE TABLE password_reset_otps (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    otp_code TEXT NOT NULL,
    expires_at TEXT NOT NULL, -- Store ISO 8601 timestamp (UTC)
    is_used BOOLEAN NOT NULL DEFAULT 0,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Indexes for better performance
CREATE INDEX idx_password_reset_otps_user_id ON password_reset_otps(user_id);
CREATE INDEX idx_password_reset_otps_otp_code ON password_reset_otps(otp_code);
CREATE INDEX idx_password_reset_otps_expires_at ON password_reset_otps(expires_at);
CREATE INDEX idx_password_reset_otps_is_used ON password_reset_otps(is_used);
CREATE INDEX idx_password_reset_otps_created_at ON password_reset_otps(created_at);
CREATE INDEX idx_password_reset_otps_user_expires ON password_reset_otps(user_id, expires_at);
CREATE INDEX idx_password_reset_otps_otp_used ON password_reset_otps(otp_code, is_used);

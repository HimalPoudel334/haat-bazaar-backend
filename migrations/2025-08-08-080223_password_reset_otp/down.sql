-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS idx_password_reset_otps_expires_at;
DROP INDEX IF EXISTS idx_password_reset_otps_otp_code;
DROP INDEX IF EXISTS idx_password_reset_otps_user_id;
DROP TABLE IF EXISTS password_reset_otps;

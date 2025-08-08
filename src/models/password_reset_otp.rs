use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;

use crate::models::user::User;

#[derive(Queryable, Identifiable, Serialize, Debug, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::password_reset_otps)]
pub struct PasswordResetOtp {
    pub id: i32,
    pub user_id: i32,
    pub otp_code: String,
    pub expires_at: String,
    pub is_used: bool,
    pub attempts: i32,
    pub created_at: String,
}

impl PasswordResetOtp {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }

    pub fn get_otp_code(&self) -> &str {
        &self.otp_code
    }

    pub fn get_expiration(&self) -> DateTime<Utc> {
        self.expires_at
            .parse::<DateTime<Utc>>()
            .expect("Failed to parse expires_at as RFC3339 datetime")
    }
    pub fn is_used(&self) -> bool {
        self.is_used
    }

    pub fn get_attempts(&self) -> i32 {
        self.attempts
    }

    pub fn get_created_at(&self) -> DateTime<Utc> {
        self.created_at
            .parse::<DateTime<Utc>>()
            .expect("Failed to parse created_at as RFC3339 datetime")
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::password_reset_otps)]
pub struct NewPasswordResetOtp {
    pub user_id: i32,
    pub otp_code: String,
    pub expires_at: String,
    pub is_used: bool,
    pub attempts: i32,
}

impl NewPasswordResetOtp {
    pub fn new(user_id: i32, otp_code: String, expiry: i32) -> Self {
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expiry as i64);
        let expires_at = expires_at.to_rfc3339();
        Self {
            user_id,
            otp_code,
            expires_at,
            is_used: false,
            attempts: 0,
        }
    }
}

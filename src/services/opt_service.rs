use chrono::Utc;
use diesel::prelude::*;
use rand::Rng;

use crate::{
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        password_reset_otp::{NewPasswordResetOtp, PasswordResetOtp},
        user::User,
    },
    utils::password_helper,
};

#[derive(Debug)]
pub enum OtpError {
    DatabaseError(diesel::result::Error),
    UserNotFound,
    InvalidOtp,
    AttemptsExceeded,
    OtpExpired,
}

impl From<diesel::result::Error> for OtpError {
    fn from(err: diesel::result::Error) -> Self {
        OtpError::DatabaseError(err)
    }
}

pub struct OtpService {
    pool: SqliteConnectionPool,
}

impl OtpService {
    pub fn new(pool: &SqliteConnectionPool) -> Self {
        Self {
            pool: pool.to_owned(),
        }
    }

    fn generate_otp() -> (String, String) {
        let mut rng = rand::rng();
        let otp = format!("{:06}", rng.random_range(100000..999999));
        let otp_hash = password_helper::hash_otp(&otp).unwrap();

        (otp, otp_hash)
    }

    pub fn find_user_by_email(&self, email: &str) -> Result<Option<User>, diesel::result::Error> {
        use crate::schema::users;

        let conn = &mut get_conn(&self.pool);

        users::table
            .filter(users::email.eq(email))
            // .filter(users::is_verified.eq(true))
            .first::<User>(conn)
            .optional()
    }

    pub fn create_password_reset_otp(
        &self,
        user_id: i32,
        expiry_seconds: i32,
    ) -> Result<PasswordResetOtp, OtpError> {
        use crate::schema::password_reset_otps;

        let conn = &mut get_conn(&self.pool);

        conn.transaction::<_, OtpError, _>(|con| {
            diesel::update(password_reset_otps::table)
                .filter(password_reset_otps::user_id.eq(user_id))
                .filter(password_reset_otps::is_used.eq(false))
                .set(password_reset_otps::is_used.eq(true))
                .execute(con)?;

            let otp_code_str = Self::generate_otp();

            let new_otp = NewPasswordResetOtp::new(user_id, otp_code_str.1, expiry_seconds);

            let mut otp = diesel::insert_into(password_reset_otps::table)
                .values(&new_otp)
                .get_result::<PasswordResetOtp>(con)?;

            otp.otp_code = otp_code_str.0;
            Ok(otp)
        })
    }

    pub fn verify_otp(&self, uid: i32, otp_code_str: &str) -> Result<bool, OtpError> {
        use crate::schema::password_reset_otps;

        let conn = &mut get_conn(&self.pool);

        conn.transaction::<_, OtpError, _>(|con| {
            let otp_record = password_reset_otps::table
                .filter(password_reset_otps::user_id.eq(uid))
                .filter(password_reset_otps::is_used.eq(false))
                .filter(password_reset_otps::expires_at.gt(Utc::now().to_rfc3339()))
                .order(password_reset_otps::created_at.desc())
                .first::<PasswordResetOtp>(con)
                .optional()?;

            let otp_record = match otp_record {
                Some(record) => record,
                None => return Ok(false),
            };

            if otp_record.attempts >= 2 {
                diesel::update(password_reset_otps::table)
                    .filter(password_reset_otps::id.eq(otp_record.id))
                    .set(password_reset_otps::is_used.eq(true))
                    .execute(con)?;
                return Err(OtpError::AttemptsExceeded);
            }

            diesel::update(password_reset_otps::table)
                .filter(password_reset_otps::id.eq(otp_record.id))
                .set(password_reset_otps::attempts.eq(otp_record.attempts + 1))
                .execute(con)?;

            if Utc::now().to_rfc3339() > otp_record.expires_at {
                return Err(OtpError::OtpExpired);
            }

            if password_helper::verify_otp_hash(&otp_record.otp_code, otp_code_str) {
                return Ok(true);
            }

            Ok(false)
        })
    }

    pub fn mark_otp_as_used(
        &self,
        uid: i32,
        otp_code_str: &str,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::password_reset_otps;

        let conn = &mut get_conn(&self.pool);

        diesel::update(password_reset_otps::table)
            .filter(password_reset_otps::user_id.eq(uid))
            .filter(password_reset_otps::expires_at.gt(Utc::now().to_rfc3339()))
            .filter(password_reset_otps::otp_code.eq(otp_code_str))
            .filter(password_reset_otps::is_used.eq(false))
            .set(password_reset_otps::is_used.eq(true))
            .execute(conn)?;

        Ok(())
    }

    pub fn cleanup_expired_otps(&self) -> Result<usize, diesel::result::Error> {
        use crate::schema::password_reset_otps;

        let conn = &mut get_conn(&self.pool);

        let deleted_count = diesel::delete(password_reset_otps::table)
            .filter(password_reset_otps::expires_at.lt(Utc::now().to_rfc3339()))
            .execute(conn)?;

        Ok(deleted_count)
    }

    pub fn has_valid_otp(&self, user_id: i32) -> Result<bool, diesel::result::Error> {
        use crate::schema::password_reset_otps;

        let conn = &mut get_conn(&self.pool);

        let count = password_reset_otps::table
            .filter(password_reset_otps::user_id.eq(user_id))
            .filter(password_reset_otps::is_used.eq(false))
            .filter(password_reset_otps::expires_at.gt(Utc::now().to_rfc3339()))
            .count()
            .get_result::<i64>(conn)?;

        Ok(count > 0)
    }

    pub fn update_user_password(
        &self,
        user_id: i32,
        new_password_hash: &str,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::{password_reset_otps, users};

        let conn = &mut get_conn(&self.pool);

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::update(users::table)
                .filter(users::id.eq(user_id))
                .set(users::password.eq(new_password_hash))
                .execute(conn)?;

            diesel::update(password_reset_otps::table)
                .filter(password_reset_otps::user_id.eq(user_id))
                .set(password_reset_otps::is_used.eq(true))
                .execute(conn)?;

            Ok(())
        })
    }
}

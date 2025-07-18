use std::collections::HashSet;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
    iat: usize,
    pub roles: HashSet<String>,
}

pub async fn create_jwt_token(
    user_id: String,
    role: String,
    max_age: i32,
    jwt_secret: String,
) -> Result<String, Error> {
    let claims = Claims {
        sub: user_id,
        exp: get_expiration(max_age as i64).0,
        iat: get_current_datetime(),
        roles: {
            let mut roles_set = HashSet::new();
            roles_set.insert(role);
            roles_set
        },
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

pub async fn create_refresh_token(
    user_id: String,
    refresh_secret: &String,
    refresh_expiry: usize,
) -> Result<String, Error> {
    let claims = Claims {
        sub: user_id,
        exp: refresh_expiry,
        iat: get_current_datetime(),
        roles: HashSet::new(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(refresh_secret.as_bytes()),
    )?;

    Ok(token)
}

pub async fn verify_jwt_with_validation(
    token: &str,
    jwt_secret: &[u8],
    validate_expiration: bool,
) -> Result<Claims, Error> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = validate_expiration;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret), &validation)?;

    Ok(token_data.claims)
}

// Wrapper with default validate_expiration = true
pub async fn verify_jwt(token: &str, jwt_secret: &[u8]) -> Result<Claims, Error> {
    verify_jwt_with_validation(token, jwt_secret, true).await
}

/// Returns both expiration timestamp (UTC) and formatted datetime in +05:45.
pub fn get_expiration(exp_in_minutes: i64) -> (usize, String) {
    let timezone = get_timezone();

    let expiration_utc: chrono::DateTime<Utc> = Utc::now()
        .checked_add_signed(Duration::minutes(exp_in_minutes))
        .expect("Failed to add expiration time");

    let expiration_str = expiration_utc
        .with_timezone(&timezone)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    (expiration_utc.timestamp() as usize, expiration_str)
}

fn get_timezone() -> chrono::FixedOffset {
    chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap() // +05:45
}

fn get_current_datetime() -> usize {
    Utc::now().with_timezone(&get_timezone()).timestamp() as usize
}

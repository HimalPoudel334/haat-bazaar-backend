use std::collections::HashSet;

use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
    iat: usize,
    pub roles: HashSet<String>,
}

pub async fn create_jwt_token(user_id: String, role: String, max_age: i32, jwt_secret: String) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(max_age as i64))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
        roles: {
            let mut roles_set = HashSet::new();
            roles_set.insert(role);
            roles_set
        },
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes())
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, jwt_secret: &[u8]) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret),
        &Validation::default()
    );

    Ok(token_data?.claims)
}

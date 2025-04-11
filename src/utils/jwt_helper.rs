use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize, // expiration time
    iat: usize, // issued at
}

pub async fn create_jwt_token(user_id: String, max_age: i32, jwt_secret: String) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(max_age as i64))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes())
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, jwt_secret: String) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default()
    );

    Ok(token_data?.claims)
}

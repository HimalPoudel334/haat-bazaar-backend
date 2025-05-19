use argon2::{
    password_hash::{Error, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand_core::OsRng;

/// Hashes a password securely using Argon2 and returns it in PHC string format.
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a plaintext password against a hashed password string.
/// Returns true if they match, false otherwise.
pub fn verify_password_hash(hashed_password: &str, input_password: &str) -> bool {
    match PasswordHash::new(hashed_password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(input_password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

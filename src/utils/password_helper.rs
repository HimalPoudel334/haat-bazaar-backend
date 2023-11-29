use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn hash_password(password: &String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string()
}

///Verifies is the hashes are same
pub fn verify_hashes(original: String, new: String) -> bool {
    let password_hash = PasswordHash::new(&original).unwrap();
    match Argon2::default().verify_password(new.as_bytes(), &password_hash) {
        Ok(_) => {
            println!("Password match");
            true
        }
        Err(_) => {
            println!("Password didn't match");
            false
        }
    }
}

// V2
// src/infrastructure/security/password.rs
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::error::Error;

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.to_string())?;
        Ok(password_hash.to_string())
    }

    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| e.to_string())?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

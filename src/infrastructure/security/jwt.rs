// V2
// src/infrastructure/security/jwt.rs
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, TokenData};
use std::error::Error;
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtService {
    access_secret: String,
    refresh_secret: String,
}

impl JwtService {
    pub fn new(access_secret: String, refresh_secret: String) -> Self {
        Self { access_secret, refresh_secret }
    }

    pub fn generate_token(&self, user_id: Uuid) -> Result<String, Box<dyn Error + Send + Sync>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(60))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.access_secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, Box<dyn Error + Send + Sync>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.refresh_secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, Box<dyn Error + Send + Sync>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.access_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<TokenData<Claims>, Box<dyn Error + Send + Sync>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.refresh_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data)
    }
}

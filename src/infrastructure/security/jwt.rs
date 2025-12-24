// V1
// src/infrastructure/security/jwt.rs
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::domain::UserId;
use crate::error::AppError;
use crate::presentation::{AccessTokenClaims, RefreshTokenClaims};

/// Trait para abstraer el servicio JWT
/// Permite testing con mocks y cambio de implementación
#[async_trait::async_trait]
pub trait JwtService: Send + Sync {
    /// Genera un access token para un usuario
    fn generate_access_token(&self, user_id: UserId, username: String) -> Result<String, AppError>;

    /// Genera un refresh token para un usuario
    fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AppError>;

    /// Valida y decodifica un access token
    fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError>;

    /// Valida y decodifica un refresh token
    fn validate_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AppError>;

    /// Obtiene el tiempo de expiración del access token en segundos
    fn get_access_token_expiry(&self) -> i64;

    /// Obtiene el tiempo de expiración del refresh token en segundos
    fn get_refresh_token_expiry(&self) -> i64;
}

/// Implementación del servicio JWT usando jsonwebtoken
#[derive(Clone)]
pub struct JwtServiceImpl {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtServiceImpl {
    /// Crea un nuevo servicio JWT
    ///
    /// # Parámetros
    /// - `secret`: Secret para firmar tokens (mínimo 32 caracteres recomendado)
    /// - `access_token_expiry`: Tiempo de vida del access token en segundos
    /// - `refresh_token_expiry`: Tiempo de vida del refresh token en segundos
    pub fn new(secret: String, access_token_expiry: i64, refresh_token_expiry: i64) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        Self {
            encoding_key,
            decoding_key,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    /// Genera un token JWT genérico
    fn generate_token<T: Serialize>(&self, claims: &T) -> Result<String, AppError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|_| AppError::JwtGenerationError)
    }

    /// Valida y decodifica un token JWT genérico
    fn decode_token<T: for<'de> Deserialize<'de>>(&self, token: &str) -> Result<T, AppError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.leeway = 0; // Sin margen de error en expiración

        let token_data = decode::<T>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}

#[async_trait::async_trait]
impl JwtService for JwtServiceImpl {
    fn generate_access_token(&self, user_id: UserId, username: String) -> Result<String, AppError> {
        let claims = AccessTokenClaims::new(user_id, username, self.access_token_expiry);
        self.generate_token(&claims)
    }

    fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AppError> {
        let claims = RefreshTokenClaims::new(user_id, self.refresh_token_expiry);
        self.generate_token(&claims)
    }

    fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError> {
        self.decode_token(token)
    }

    fn validate_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AppError> {
        let claims: RefreshTokenClaims = self.decode_token(token)?;

        // Validación adicional: verificar que es un refresh token
        if claims.token_type != "refresh" {
            return Err(AppError::InvalidRefreshToken);
        }

        Ok(claims)
    }

    fn get_access_token_expiry(&self) -> i64 {
        self.access_token_expiry
    }

    fn get_refresh_token_expiry(&self) -> i64 {
        self.refresh_token_expiry
    }
}
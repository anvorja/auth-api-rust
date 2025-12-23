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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtServiceImpl {
        JwtServiceImpl::new(
            "test-secret-key-minimum-32-characters-long".to_string(),
            900,    // 15 minutos
            604800, // 7 días
        )
    }

    #[test]
    fn test_generate_access_token() {
        let service = create_test_service();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let token = service.generate_access_token(user_id, username.clone()).unwrap();

        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ")); // JWT siempre empieza con "eyJ"
    }

    #[test]
    fn test_generate_refresh_token() {
        let service = create_test_service();
        let user_id = UserId::new();

        let token = service.generate_refresh_token(user_id).unwrap();

        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ"));
    }

    #[test]
    fn test_validate_access_token() {
        let service = create_test_service();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let token = service.generate_access_token(user_id, username.clone()).unwrap();
        let claims = service.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_validate_refresh_token() {
        let service = create_test_service();
        let user_id = UserId::new();

        let token = service.generate_refresh_token(user_id).unwrap();
        let claims = service.validate_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_invalid_token() {
        let service = create_test_service();
        let invalid_token = "invalid.jwt.token";

        let result = service.validate_access_token(invalid_token);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidToken | AppError::JwtDecodingError));
    }

    #[test]
    fn test_expired_token() {
        // Crear un servicio con expiración de -1 segundo (ya expirado)
        let service = JwtServiceImpl::new(
            "test-secret-key-minimum-32-characters-long".to_string(),
            -1, // Ya expirado
            604800,
        );

        let user_id = UserId::new();
        let username = "testuser".to_string();

        let token = service.generate_access_token(user_id, username).unwrap();
        let result = service.validate_access_token(&token);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidToken));
    }

    #[test]
    fn test_wrong_secret() {
        let service1 = JwtServiceImpl::new(
            "secret-key-one-minimum-32-characters-long".to_string(),
            900,
            604800,
        );

        let service2 = JwtServiceImpl::new(
            "secret-key-two-minimum-32-characters-long".to_string(),
            900,
            604800,
        );

        let user_id = UserId::new();
        let username = "testuser".to_string();

        // Generar token con service1
        let token = service1.generate_access_token(user_id, username).unwrap();

        // Intentar validar con service2 (diferente secret)
        let result = service2.validate_access_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_as_refresh_token() {
        let service = create_test_service();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        // Generar access token
        let access_token = service.generate_access_token(user_id, username).unwrap();

        // Intentar validarlo como refresh token (debe fallar)
        let result = service.validate_refresh_token(&access_token);

        // El token es válido como JWT pero no como refresh token
        // Dependiendo de la implementación, podría fallar en el decode o en la validación del tipo
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiry_values() {
        let service = create_test_service();

        assert_eq!(service.get_access_token_expiry(), 900);
        assert_eq!(service.get_refresh_token_expiry(), 604800);
    }

    #[test]
    fn test_token_claims_timestamps() {
        let service = create_test_service();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        let before = Utc::now().timestamp();
        let token = service.generate_access_token(user_id, username).unwrap();
        let after = Utc::now().timestamp();

        let claims = service.validate_access_token(&token).unwrap();

        // iat debe estar entre before y after
        assert!(claims.iat >= before && claims.iat <= after);

        // exp debe ser iat + access_token_expiry
        assert_eq!(claims.exp, claims.iat + service.get_access_token_expiry());
    }

    #[test]
    fn test_multiple_tokens_for_same_user() {
        let service = create_test_service();
        let user_id = UserId::new();
        let username = "testuser".to_string();

        // Generar dos tokens para el mismo usuario
        let token1 = service.generate_access_token(user_id, username.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10)); // Pequeña pausa
        let token2 = service.generate_access_token(user_id, username).unwrap();

        // Los tokens deben ser diferentes (diferentes timestamps)
        assert_ne!(token1, token2);

        // Pero ambos deben ser válidos
        let claims1 = service.validate_access_token(&token1).unwrap();
        let claims2 = service.validate_access_token(&token2).unwrap();

        assert_eq!(claims1.sub, claims2.sub);
        assert_eq!(claims1.username, claims2.username);
    }
}
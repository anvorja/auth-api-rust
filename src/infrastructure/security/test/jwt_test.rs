// src/infrastructure/security/test/jwt_test.rs
use crate::domain::UserId;
use crate::error::AppError;
use crate::infrastructure::security::{JwtService, JwtServiceImpl};

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

    assert!(result.is_err());
}

#[test]
fn test_token_expiry_values() {
    let service = create_test_service();

    assert_eq!(service.get_access_token_expiry(), 900);
    assert_eq!(service.get_refresh_token_expiry(), 604800);
}

#[test]
fn test_multiple_tokens_for_same_user() {
    let service = create_test_service();
    let user_id = UserId::new();
    let username = "testuser".to_string();

    // Generar dos tokens para el mismo usuario
    let token1 = service.generate_access_token(user_id, username.clone()).unwrap();

    // Esperar al menos 1 segundo para que el timestamp (en segundos) sea diferente
    std::thread::sleep(std::time::Duration::from_secs(1));

    let token2 = service.generate_access_token(user_id, username).unwrap();

    // Los tokens deben ser diferentes (diferentes timestamps)
    assert_ne!(token1, token2);

    // Pero ambos deben ser válidos
    let claims1 = service.validate_access_token(&token1).unwrap();
    let claims2 = service.validate_access_token(&token2).unwrap();

    assert_eq!(claims1.sub, claims2.sub);
    assert_eq!(claims1.username, claims2.username);
}
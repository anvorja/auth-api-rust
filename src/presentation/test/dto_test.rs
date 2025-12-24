// src/presentation/test/dto_test.rs
use crate::presentation::{RegisterRequest, HealthResponse};
use validator::Validate;

#[test]
fn test_password_strength_validation() {
    use crate::presentation::dto::validate_password_strength;

    // Válidas
    assert!(validate_password_strength("Password1").is_ok());
    assert!(validate_password_strength("MyP@ssw0rd").is_ok());

    // Inválidas
    assert!(validate_password_strength("password").is_err()); // sin mayúscula ni número
    assert!(validate_password_strength("PASSWORD1").is_err()); // sin minúscula
    assert!(validate_password_strength("Password").is_err()); // sin número
}

#[test]
fn test_register_request_validation() {
    let valid_request = RegisterRequest {
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Username inválido (muy corto)
    let invalid_request = RegisterRequest {
        username: "jo".to_string(),
        email: "john@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Email inválido
    let invalid_request = RegisterRequest {
        username: "johndoe".to_string(),
        email: "invalid-email".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Password débil
    let invalid_request = RegisterRequest {
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "weak".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_health_response() {
    let health = HealthResponse::healthy();
    assert_eq!(health.status, "healthy");

    let unhealthy = HealthResponse::unhealthy();
    assert_eq!(unhealthy.status, "unhealthy");
}
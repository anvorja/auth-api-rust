// src/presentation/test/dto_test.rs
use crate::presentation::*;
use validator::Validate;

#[test]
fn test_password_strength_validation() {
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

#[test]
fn test_change_password_request_passwords_match() {
    // Contraseñas coinciden
    let request = ChangePasswordRequest {
        current_password: "OldPass123!".to_string(),
        new_password: "NewPass456!".to_string(),
        new_password_confirmation: "NewPass456!".to_string(),
    };
    assert!(request.passwords_match());

    // Contraseñas NO coinciden
    let request = ChangePasswordRequest {
        current_password: "OldPass123!".to_string(),
        new_password: "NewPass456!".to_string(),
        new_password_confirmation: "DifferentPass789!".to_string(),
    };
    assert!(!request.passwords_match());
}

#[test]
fn test_change_password_request_validation() {
    // Request válido
    let valid_request = ChangePasswordRequest {
        current_password: "OldPass123!".to_string(),
        new_password: "NewSecurePass456!".to_string(),
        new_password_confirmation: "NewSecurePass456!".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Nueva contraseña débil
    let invalid_request = ChangePasswordRequest {
        current_password: "OldPass123!".to_string(),
        new_password: "weak".to_string(),
        new_password_confirmation: "weak".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Nueva contraseña sin mayúscula
    let invalid_request = ChangePasswordRequest {
        current_password: "OldPass123!".to_string(),
        new_password: "newpass123!".to_string(),
        new_password_confirmation: "newpass123!".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_login_request_validation() {
    // Request válido
    let valid_request = LoginRequest {
        username: "johndoe".to_string(),
        password: "SecurePass123!".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Username muy corto
    let invalid_request = LoginRequest {
        username: "jo".to_string(),
        password: "SecurePass123!".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Password vacía
    let invalid_request = LoginRequest {
        username: "johndoe".to_string(),
        password: "".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_refresh_request_validation() {
    // Request válido
    let valid_request = RefreshRequest {
        refresh_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Token vacío
    let invalid_request = RefreshRequest {
        refresh_token: "".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_register_request_to_domain_types() {
    let request = RegisterRequest {
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result = request.to_domain_types();
    assert!(result.is_ok());

    let (username, email) = result.unwrap();
    assert_eq!(username.value(), "johndoe");
    assert_eq!(email.value(), "john@example.com");
}

#[test]
fn test_register_request_to_domain_types_invalid_username() {
    let request = RegisterRequest {
        username: "123invalid".to_string(), // No empieza con letra
        email: "john@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result = request.to_domain_types();
    assert!(result.is_err());
}

#[test]
fn test_register_request_to_domain_types_invalid_email() {
    let request = RegisterRequest {
        username: "johndoe".to_string(),
        email: "invalid-email".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result = request.to_domain_types();
    assert!(result.is_err());
}

#[test]
fn test_user_response_from_domain() {
    use crate::domain::{User, Username, Email, PasswordHash};

    let user = User::new(
        Username::new("johndoe".to_string()).unwrap(),
        Email::new("john@example.com".to_string()).unwrap(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("$argon2id$...".to_string()),
    );

    let response = UserResponse::from_domain(&user);

    assert_eq!(response.username, "johndoe");
    assert_eq!(response.email, "john@example.com");
    assert_eq!(response.first_name, "John");
    assert_eq!(response.last_name, "Doe");
}

#[test]
fn test_auth_response_new() {
    use crate::domain::{User, Username, Email, PasswordHash};

    let user = User::new(
        Username::new("johndoe".to_string()).unwrap(),
        Email::new("john@example.com".to_string()).unwrap(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("$argon2id$...".to_string()),
    );

    let response = AuthResponse::new(
        "access_token_here".to_string(),
        "refresh_token_here".to_string(),
        900,
        &user,
    );

    assert_eq!(response.access_token, "access_token_here");
    assert_eq!(response.refresh_token, "refresh_token_here");
    assert_eq!(response.token_type, "Bearer");
    assert_eq!(response.expires_in, 900);
    assert_eq!(response.user.username, "johndoe");
}

#[test]
fn test_access_token_claims_new() {
    use crate::domain::UserId;

    let user_id = UserId::new();
    let username = "johndoe".to_string();
    let expires_in = 900;

    let claims = AccessTokenClaims::new(user_id, username.clone(), expires_in);

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.username, username);
    assert!(claims.exp > claims.iat);
    assert_eq!(claims.exp - claims.iat, expires_in);
}

#[test]
fn test_refresh_token_claims_new() {
    use crate::domain::UserId;

    let user_id = UserId::new();
    let expires_in = 604800;

    let claims = RefreshTokenClaims::new(user_id, expires_in);

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.token_type, "refresh");
    assert!(claims.exp > claims.iat);
    assert_eq!(claims.exp - claims.iat, expires_in);
}
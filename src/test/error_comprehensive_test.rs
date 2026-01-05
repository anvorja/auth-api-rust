// src/test/error_comprehensive_test.rs
use crate::error::{AppError, ErrorResponse};
use axum::http::StatusCode;

// ============================================================================
// TESTS DE CONVERSIÓN DESDE DIFERENTES ERRORES
// ============================================================================

#[test]
fn test_from_sqlx_row_not_found() {
    let sqlx_error = sqlx::Error::RowNotFound;
    let app_error: AppError = sqlx_error.into();

    assert!(matches!(app_error, AppError::UserNotFound));
    assert_eq!(app_error.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn test_from_sqlx_pool_timeout() {
    let sqlx_error = sqlx::Error::PoolTimedOut;
    let app_error: AppError = sqlx_error.into();

    assert!(matches!(app_error, AppError::DatabaseConnectionError));
    assert_eq!(app_error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn test_from_sqlx_pool_closed() {
    let sqlx_error = sqlx::Error::PoolClosed;
    let app_error: AppError = sqlx_error.into();

    assert!(matches!(app_error, AppError::DatabaseConnectionError));
    assert_eq!(app_error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn test_from_argon2_error() {
    use argon2::password_hash::Error as Argon2Error;

    let argon2_error = Argon2Error::Password;
    let app_error: AppError = argon2_error.into();

    assert!(matches!(app_error, AppError::PasswordHashError));
    assert_eq!(app_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_from_jwt_expired_signature() {
    use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

    let jwt_error = JwtError::from(ErrorKind::ExpiredSignature);
    let app_error: AppError = jwt_error.into();

    assert!(matches!(app_error, AppError::InvalidToken));
    assert_eq!(app_error.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_from_jwt_invalid_token() {
    use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

    let jwt_error = JwtError::from(ErrorKind::InvalidToken);
    let app_error: AppError = jwt_error.into();

    assert!(matches!(app_error, AppError::InvalidToken));
}

#[test]
fn test_from_jwt_invalid_signature() {
    use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

    let jwt_error = JwtError::from(ErrorKind::InvalidSignature);
    let app_error: AppError = jwt_error.into();

    assert!(matches!(app_error, AppError::InvalidToken));
}

#[test]
fn test_from_validation_errors() {
    use validator::{ValidationError, ValidationErrors};
    use std::borrow::Cow;

    // Crear ValidationErrors correctamente
    let mut errors = ValidationErrors::new();

    let mut error = ValidationError::new("invalid");
    error.message = Some(Cow::Borrowed("El campo es inválido"));

    // Usar add_param para agregar el error
    errors.add("username", error);

    let app_error: AppError = errors.into();

    assert!(matches!(app_error, AppError::ValidationError(_)));
    assert_eq!(app_error.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_from_env_var_error() {
    use std::env::VarError;

    let env_error = VarError::NotPresent;
    let app_error: AppError = env_error.into();

    assert!(matches!(app_error, AppError::ConfigError(_)));
    assert_eq!(app_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_from_io_error_already_tested() {
    // Ya existe en error_test.rs, pero lo documentamos aquí
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let app_error: AppError = io_error.into();

    assert!(matches!(app_error, AppError::ConfigError(_)));
}

#[test]
fn test_from_anyhow_error() {
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let app_error: AppError = anyhow_error.into();

    assert!(matches!(app_error, AppError::InternalServerError));
    assert_eq!(app_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// TESTS DE ERROR_CODE
// ============================================================================

#[test]
fn test_all_error_codes() {
    assert_eq!(AppError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
    assert_eq!(AppError::InvalidToken.error_code(), "INVALID_TOKEN");
    assert_eq!(AppError::InvalidRefreshToken.error_code(), "INVALID_REFRESH_TOKEN");
    assert_eq!(AppError::Unauthorized.error_code(), "UNAUTHORIZED");
    assert_eq!(AppError::ValidationError("test".to_string()).error_code(), "VALIDATION_ERROR");
    assert_eq!(AppError::UserAlreadyExists.error_code(), "USER_ALREADY_EXISTS");
    assert_eq!(AppError::UserNotFound.error_code(), "USER_NOT_FOUND");
    assert_eq!(AppError::DatabaseError("test".to_string()).error_code(), "DATABASE_ERROR");
    assert_eq!(AppError::DatabaseConnectionError.error_code(), "DATABASE_CONNECTION_ERROR");
    assert_eq!(AppError::PasswordHashError.error_code(), "PASSWORD_HASH_ERROR");
    assert_eq!(AppError::JwtGenerationError.error_code(), "JWT_GENERATION_ERROR");
    assert_eq!(AppError::JwtDecodingError.error_code(), "JWT_DECODING_ERROR");
    assert_eq!(AppError::ConfigError("test".to_string()).error_code(), "CONFIG_ERROR");
    assert_eq!(AppError::InternalServerError.error_code(), "INTERNAL_SERVER_ERROR");
    assert_eq!(AppError::ServiceUnavailable.error_code(), "SERVICE_UNAVAILABLE");
}

// ============================================================================
// TESTS DE CLIENT_MESSAGE (seguridad)
// ============================================================================

#[test]
fn test_client_messages_no_leak_internal_details() {
    // Los errores internos NO deben revelar detalles
    let errors = vec![
        AppError::DatabaseError("SELECT * FROM secret_table WHERE password = 'admin'".to_string()),
        AppError::PasswordHashError,
        AppError::JwtGenerationError,
        AppError::JwtDecodingError,
        AppError::ConfigError("DATABASE_URL=postgresql://user:pass@host/db".to_string()),
    ];

    for error in errors {
        let message = error.client_message();
        // No debe contener detalles internos
        assert!(!message.contains("SELECT"));
        assert!(!message.contains("password"));
        assert!(!message.contains("DATABASE_URL"));
        assert!(!message.contains("postgresql"));
        assert!(message.contains("Error interno") || message.contains("interno del servidor"));
    }
}

#[test]
fn test_client_messages_user_friendly() {
    // Los errores de usuario deben ser claros
    assert!(AppError::InvalidCredentials.client_message().contains("incorrectos"));
    assert!(AppError::UserAlreadyExists.client_message().contains("ya"));
    assert!(AppError::UserNotFound.client_message().contains("no encontrado"));
    assert!(AppError::ValidationError("test".to_string()).client_message().contains("test"));
}

// ============================================================================
// TESTS DE DETAILS (solo en development)
// ============================================================================

#[test]
fn test_details_in_development() {
    let error = AppError::DatabaseError("Connection timeout".to_string());

    // Con detalles (development)
    let details = error.details(true);
    assert!(details.is_some());
    assert_eq!(details.unwrap(), "Connection timeout");
}

#[test]
fn test_details_in_production() {
    let error = AppError::DatabaseError("Connection timeout".to_string());

    // Sin detalles (production)
    let details = error.details(false);
    assert!(details.is_none());
}

#[test]
fn test_details_only_for_specific_errors() {
    // Solo DatabaseError y ConfigError tienen detalles
    assert!(AppError::DatabaseError("msg".to_string()).details(true).is_some());
    assert!(AppError::ConfigError("msg".to_string()).details(true).is_some());

    // Otros errores no tienen detalles adicionales
    assert!(AppError::InvalidCredentials.details(true).is_none());
    assert!(AppError::UserNotFound.details(true).is_none());
}

// ============================================================================
// TESTS DE CONSTRUCTORES CONVENIENTES
// ============================================================================

#[test]
fn test_config_constructor() {
    let error1 = AppError::config("Missing DATABASE_URL");
    let error2 = AppError::config(String::from("Missing DATABASE_URL"));

    assert!(matches!(error1, AppError::ConfigError(_)));
    assert!(matches!(error2, AppError::ConfigError(_)));
}

#[test]
fn test_internal_constructor() {
    let error = AppError::internal();
    assert!(matches!(error, AppError::InternalServerError));
}

#[test]
fn test_unavailable_constructor() {
    let error = AppError::unavailable();
    assert!(matches!(error, AppError::ServiceUnavailable));
}

// ============================================================================
// TESTS DE INTO_RESPONSE (serialización HTTP)
// ============================================================================

#[test]
fn test_into_response_structure() {
    use axum::response::IntoResponse;

    let error = AppError::UserNotFound;
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_error_response_serialization() {
    use serde_json;

    let error = AppError::ValidationError("Username too short".to_string());
    let error_code = error.error_code();
    let message = error.client_message();

    // Verificar que la estructura es serializable
    let response = ErrorResponse {
        error: crate::error::ErrorDetail {
            code: error_code,
            message,
            details: None,
        },
    };

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());
}

// ============================================================================
// TESTS ADICIONALES DE STATUS_CODE
// ============================================================================

#[test]
fn test_all_status_codes() {
    // 401 Unauthorized
    assert_eq!(AppError::InvalidCredentials.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(AppError::InvalidToken.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(AppError::InvalidRefreshToken.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);

    // 400 Bad Request
    assert_eq!(AppError::ValidationError("test".to_string()).status_code(), StatusCode::BAD_REQUEST);

    // 404 Not Found
    assert_eq!(AppError::UserNotFound.status_code(), StatusCode::NOT_FOUND);

    // 409 Conflict
    assert_eq!(AppError::UserAlreadyExists.status_code(), StatusCode::CONFLICT);

    // 503 Service Unavailable
    assert_eq!(AppError::ServiceUnavailable.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(AppError::DatabaseConnectionError.status_code(), StatusCode::SERVICE_UNAVAILABLE);

    // 500 Internal Server Error
    assert_eq!(AppError::DatabaseError("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::PasswordHashError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::JwtGenerationError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::JwtDecodingError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::ConfigError("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(AppError::InternalServerError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// TESTS DE CONVERSIÓN CONFIG_ERROR
// ============================================================================

#[test]
fn test_from_config_error() {
    use crate::config::settings::ConfigError;

    let config_error = ConfigError::MissingEnvVar("DATABASE_URL".to_string());
    let app_error: AppError = config_error.into();

    assert!(matches!(app_error, AppError::ConfigError(_)));
}
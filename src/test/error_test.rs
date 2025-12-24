// src/test/error_test.rs
use crate::error::AppError;
use axum::http::StatusCode;

#[test]
fn test_status_codes() {
    assert_eq!(
        AppError::InvalidCredentials.status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        AppError::UserAlreadyExists.status_code(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        AppError::ValidationError("test".to_string()).status_code(),
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_error_codes() {
    assert_eq!(
        AppError::InvalidCredentials.error_code(),
        "INVALID_CREDENTIALS"
    );
    assert_eq!(AppError::Unauthorized.error_code(), "UNAUTHORIZED");
}

#[test]
fn test_client_messages_safe() {
    // Los errores internos no deben revelar detalles
    let internal_error = AppError::DatabaseError("SELECT * FROM secret_table".to_string());
    assert!(!internal_error
        .client_message()
        .contains("secret_table"));
}
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

// ============================================================================
// TESTS PARA LAS VARIANTES USADAS
// ============================================================================

#[test]
fn test_config_error() {
    let error = AppError::config("DATABASE_URL no configurada");
    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error.error_code(), "CONFIG_ERROR");
    assert!(error.client_message().contains("Error interno del servidor"));
}

#[test]
fn test_config_error_constructor() {
    // Test que el constructor funciona correctamente
    let error1 = AppError::config("Test error");
    let error2 = AppError::config(String::from("Test error"));

    assert_eq!(error1.error_code(), error2.error_code());
    assert_eq!(error1.status_code(), error2.status_code());
}

#[test]
fn test_internal_server_error() {
    let error = AppError::internal();
    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error.error_code(), "INTERNAL_SERVER_ERROR");
    assert!(error.client_message().contains("Error interno del servidor"));
}

#[test]
fn test_service_unavailable() {
    let error = AppError::unavailable();
    assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.error_code(), "SERVICE_UNAVAILABLE");
    assert!(error.client_message().contains("temporalmente no disponible"));
}

#[test]
fn test_service_unavailable_vs_database_connection_error() {
    // ServiceUnavailable es temporal (503)
    let unavailable = AppError::unavailable();
    assert_eq!(unavailable.status_code(), StatusCode::SERVICE_UNAVAILABLE);

    // DatabaseConnectionError es más permanente (500)
    let db_error = AppError::DatabaseConnectionError;
    assert_eq!(db_error.status_code(), StatusCode::SERVICE_UNAVAILABLE);

    // Pero sus códigos de error son diferentes
    assert_ne!(unavailable.error_code(), db_error.error_code());
}

#[test]
fn test_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let app_error: AppError = io_error.into();

    assert_eq!(app_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(app_error.error_code(), "CONFIG_ERROR");
}
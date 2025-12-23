// V1
// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

/// Errores de la aplicación
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Errores de autenticación
    #[error("Credenciales inválidas")]
    InvalidCredentials,

    #[error("Token inválido o expirado")]
    InvalidToken,

    #[error("Token de refresh inválido")]
    InvalidRefreshToken,

    #[error("No autorizado")]
    Unauthorized,

    // Errores de validación
    #[error("Validación falló: {0}")]
    ValidationError(String),

    #[error("Usuario ya existe")]
    UserAlreadyExists,

    #[error("Usuario no encontrado")]
    UserNotFound,

    // Errores de base de datos
    #[error("Error de base de datos")]
    DatabaseError(String),

    #[error("Error de conexión a base de datos")]
    DatabaseConnectionError,

    // Errores de seguridad
    #[error("Error al hashear password")]
    PasswordHashError,

    #[error("Error al generar token JWT")]
    JwtGenerationError,

    #[error("Error al decodificar token JWT")]
    JwtDecodingError,

    // Errores de configuración
    #[error("Error de configuración: {0}")]
    ConfigError(String),

    // Errores internos
    #[error("Error interno del servidor")]
    InternalServerError,

    #[error("Servicio no disponible")]
    ServiceUnavailable,
}

/// Respuesta de error estandarizada
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl AppError {
    /// Convierte el error en un código HTTP
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 401 Unauthorized
            AppError::InvalidCredentials
            | AppError::InvalidToken
            | AppError::InvalidRefreshToken
            | AppError::Unauthorized => StatusCode::UNAUTHORIZED,

            // 400 Bad Request
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,

            // 404 Not Found
            AppError::UserNotFound => StatusCode::NOT_FOUND,

            // 409 Conflict
            AppError::UserAlreadyExists => StatusCode::CONFLICT,

            // 503 Service Unavailable
            AppError::ServiceUnavailable | AppError::DatabaseConnectionError => {
                StatusCode::SERVICE_UNAVAILABLE
            }

            // 500 Internal Server Error
            AppError::DatabaseError(_)
            | AppError::PasswordHashError
            | AppError::JwtGenerationError
            | AppError::JwtDecodingError
            | AppError::ConfigError(_)
            | AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convierte el error en un código de error para el cliente
    pub fn error_code(&self) -> String {
        match self {
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::InvalidToken => "INVALID_TOKEN",
            AppError::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::UserAlreadyExists => "USER_ALREADY_EXISTS",
            AppError::UserNotFound => "USER_NOT_FOUND",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::DatabaseConnectionError => "DATABASE_CONNECTION_ERROR",
            AppError::PasswordHashError => "PASSWORD_HASH_ERROR",
            AppError::JwtGenerationError => "JWT_GENERATION_ERROR",
            AppError::JwtDecodingError => "JWT_DECODING_ERROR",
            AppError::ConfigError(_) => "CONFIG_ERROR",
            AppError::InternalServerError => "INTERNAL_SERVER_ERROR",
            AppError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
        }
            .to_string()
    }

    /// Mensaje seguro para mostrar al cliente (sin detalles internos)
    pub fn client_message(&self) -> String {
        match self {
            AppError::InvalidCredentials => "Usuario o contraseña incorrectos".to_string(),
            AppError::InvalidToken | AppError::InvalidRefreshToken => {
                "Sesión inválida o expirada".to_string()
            }
            AppError::Unauthorized => "No autorizado para realizar esta acción".to_string(),
            AppError::ValidationError(msg) => msg.clone(),
            AppError::UserAlreadyExists => "El usuario ya está registrado".to_string(),
            AppError::UserNotFound => "Usuario no encontrado".to_string(),
            AppError::DatabaseConnectionError => {
                "No se pudo conectar con la base de datos".to_string()
            }
            AppError::ServiceUnavailable => "Servicio temporalmente no disponible".to_string(),
            // Errores internos: no revelar detalles
            AppError::DatabaseError(_)
            | AppError::PasswordHashError
            | AppError::JwtGenerationError
            | AppError::JwtDecodingError
            | AppError::ConfigError(_)
            | AppError::InternalServerError => {
                "Error interno del servidor. Por favor, intente más tarde".to_string()
            }
        }
    }

    /// Detalles adicionales (solo en desarrollo)
    pub fn details(&self, include_details: bool) -> Option<String> {
        if !include_details {
            return None;
        }

        match self {
            AppError::DatabaseError(msg) | AppError::ConfigError(msg) => Some(msg.clone()),
            _ => None,
        }
    }
}

/// Implementación de IntoResponse para convertir errores en respuestas HTTP
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // En producción, no incluir detalles internos
        let include_details = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "production".to_string())
            == "development";

        // Log del error (solo detalles internos)
        tracing::error!(
            error = ?self,
            error_code = %self.error_code(),
            "Application error occurred"
        );

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code(),
                message: self.client_message(),
                details: self.details(include_details),
            },
        };

        (self.status_code(), Json(error_response)).into_response()
    }
}

/// Conversión desde errores de SQLx
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            sqlx::Error::Database(db_err) => {
                // Chequear constraint violations (e.g., unique)
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("users_username_key")
                        || constraint.contains("users_email_key")
                    {
                        return AppError::UserAlreadyExists;
                    }
                }
                AppError::DatabaseError(db_err.to_string())
            }
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
                AppError::DatabaseConnectionError
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

/// Conversión desde errores de Argon2
impl From<argon2::password_hash::Error> for AppError {
    fn from(_: argon2::password_hash::Error) -> Self {
        AppError::PasswordHashError
    }
}

/// Conversión desde errores de JWT
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::InvalidToken,
            ErrorKind::InvalidToken | ErrorKind::InvalidSignature => AppError::InvalidToken,
            _ => AppError::JwtDecodingError,
        }
    }
}

/// Conversión desde errores de validación
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    let message = error
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Validación falló para {}", field));
                    message
                })
            })
            .collect();

        AppError::ValidationError(messages.join(", "))
    }
}

/// Alias para Result con AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

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
}
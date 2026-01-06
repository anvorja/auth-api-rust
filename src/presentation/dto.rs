// src/presentation/dto.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domain::{Email, User, UserId, Username};

// ============================================================================
// DTOs de Request (Input)
// ============================================================================

/// DTO para registro de nuevo usuario
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// Nombre de usuario único
    #[validate(
        length(min = 3, max = 30, message = "El username debe tener entre 3 y 30 caracteres"),
        regex(
            path = *USERNAME_REGEX,
            message = "El username solo puede contener letras, números y guión bajo, y debe comenzar con una letra"
        )
    )]
    #[schema(example = "johndoe", min_length = 3, max_length = 30)]
    pub username: String,

    /// Email del usuario
    #[validate(email(message = "Email inválido"), length(max = 255))]
    #[schema(example = "john@example.com", format = "email")]
    pub email: String,

    /// Nombre
    #[validate(length(min = 1, max = 100, message = "El nombre debe tener entre 1 y 100 caracteres"))]
    #[schema(example = "John", min_length = 1, max_length = 100)]
    pub first_name: String,

    /// Apellido
    #[validate(length(min = 1, max = 100, message = "El apellido debe tener entre 1 y 100 caracteres"))]
    #[schema(example = "Doe", min_length = 1, max_length = 100)]
    pub last_name: String,

    /// Contraseña (mínimo 8 caracteres, debe contener mayúscula, minúscula, número)
    #[validate(
        length(min = 8, max = 128, message = "La contraseña debe tener entre 8 y 128 caracteres"),
        custom(function = "validate_password_strength")
    )]
    #[schema(example = "SecurePass123!", min_length = 8, max_length = 128, format = "password")]
    pub password: String,
}

impl UpdateProfileRequest {
    /// Validar que al menos un campo esté presente
    pub fn has_updates(&self) -> bool {
        self.first_name.is_some() || self.last_name.is_some()
    }
}

/// DTO para login
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Username o email
    #[validate(length(min = 3, max = 255, message = "Credencial inválida"))]
    #[schema(example = "johndoe")]
    pub username: String,

    /// Contraseña
    #[validate(length(min = 1, max = 128, message = "Contraseña requerida"))]
    #[schema(example = "SecurePass123!", format = "password")]
    pub password: String,
}

/// DTO para refresh token
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshRequest {
    /// Refresh token
    #[validate(length(min = 1, message = "Refresh token requerido"))]
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

/// DTO para cambiar contraseña
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    /// Contraseña actual
    #[validate(length(min = 1, max = 128, message = "Contraseña actual requerida"))]
    #[schema(example = "CurrentPass123!", format = "password")]
    pub current_password: String,

    /// Nueva contraseña
    #[validate(
        length(min = 8, max = 128, message = "La nueva contraseña debe tener entre 8 y 128 caracteres"),
        custom(function = "validate_password_strength")
    )]
    #[schema(example = "NewSecurePass123!", min_length = 8, max_length = 128, format = "password")]
    pub new_password: String,

    /// Confirmación de nueva contraseña
    #[validate(length(min = 8, max = 128, message = "Confirmación requerida"))]
    #[schema(example = "NewSecurePass123!", format = "password")]
    pub new_password_confirmation: String,
}

/// DTO para actualizar perfil
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    /// Nombre
    #[validate(length(min = 1, max = 100, message = "El nombre debe tener entre 1 y 100 caracteres"))]
    #[schema(example = "Jane", min_length = 1, max_length = 100)]
    pub first_name: Option<String>,

    /// Apellido
    #[validate(length(min = 1, max = 100, message = "El apellido debe tener entre 1 y 100 caracteres"))]
    #[schema(example = "Smith", min_length = 1, max_length = 100)]
    pub last_name: Option<String>,
}

/// DTO para admin actualizar username
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AdminUpdateUsernameRequest {
    /// ID del usuario a actualizar
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub user_id: String,

    /// Nuevo username
    #[validate(
        length(min = 3, max = 30, message = "El username debe tener entre 3 y 30 caracteres"),
        regex(
            path = *USERNAME_REGEX,
            message = "El username solo puede contener letras, números y guión bajo"
        )
    )]
    #[schema(example = "newusername", min_length = 3, max_length = 30)]
    pub new_username: String,
}

/// DTO para admin actualizar email
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AdminUpdateEmailRequest {
    /// ID del usuario a actualizar
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub user_id: String,

    /// Nuevo email
    #[validate(email(message = "Email inválido"), length(max = 255))]
    #[schema(example = "newemail@example.com", format = "email")]
    pub new_email: String,
}


impl ChangePasswordRequest {
    /// Valida que las contraseñas coincidan
    pub fn passwords_match(&self) -> bool {
        self.new_password == self.new_password_confirmation
    }
}

// ============================================================================
// DTOs de Response (Output)
// ============================================================================

/// Respuesta de autenticación exitosa
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// Access token JWT
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    /// Refresh token JWT
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,

    /// Tipo de token
    #[schema(example = "Bearer")]
    pub token_type: String,

    /// Tiempo de expiración en segundos
    #[schema(example = 900)]
    pub expires_in: i64,

    /// Información del usuario
    pub user: UserResponse,
}

/// Información del usuario (sin datos sensibles)
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct UserResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "john@example.com", format = "email")]
    pub email: String,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "user")]
    pub role: String,
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,
}
/// Respuesta genérica de éxito
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    /// Mensaje de éxito
    #[schema(example = "Operación exitosa")]
    pub message: String,
}

/// Respuesta de health check
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Estado del servicio
    #[schema(example = "healthy")]
    pub status: String,

    /// Timestamp
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub timestamp: DateTime<Utc>,

    /// Versión de la API
    #[schema(example = "1.0.0")]
    pub version: String,
}

// ============================================================================
// Validaciones Personalizadas
// ============================================================================

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex =
        regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]{2,29}$").unwrap();
}

/// Valida la fortaleza de la contraseña
pub(crate) fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(validator::ValidationError::new(
            "La contraseña debe contener al menos una mayúscula, una minúscula y un número",
        ));
    }

    Ok(())
}

// ============================================================================
// Conversiones entre DTOs y Domain
// ============================================================================

impl UserResponse {
    /// Convierte una entidad de dominio User a UserResponse
    pub fn from_domain(user: &User) -> Self {
        Self {
            id: user.id().value(),
            username: user.username().value().to_string(),
            email: user.email().value().to_string(),
            first_name: user.first_name().to_string(),
            last_name: user.last_name().to_string(),
            role: user.role().to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

impl AuthResponse {
    /// Crea una respuesta de autenticación
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user: &User,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_domain(user),
        }
    }
}

impl HealthResponse {
    /// Crea una respuesta de health check
    pub fn new(status: String, version: String) -> Self {
        Self {
            status,
            timestamp: Utc::now(),
            version,
        }
    }

    /// Respuesta healthy por defecto
    pub fn healthy() -> Self {
        Self::new("healthy".to_string(), env!("CARGO_PKG_VERSION").to_string())
    }

    /// Respuesta unhealthy
    pub fn unhealthy() -> Self {
        Self::new("unhealthy".to_string(), env!("CARGO_PKG_VERSION").to_string())
    }
}

impl RegisterRequest {
    /// Convierte el DTO a tipos de dominio para Username y Email
    pub fn to_domain_types(&self) -> Result<(Username, Email), String> {
        let username = Username::new(self.username.clone())?;
        let email = Email::new(self.email.clone())?;
        Ok((username, email))
    }
}

// ============================================================================
// Claims JWT (para uso interno en infraestructura)
// ============================================================================

/// Claims del access token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
}

/// Claims del refresh token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Token type
    pub token_type: String,
}

impl AccessTokenClaims {
    pub fn new(user_id: UserId, username: String, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            username,
            exp: now + expires_in,
            iat: now,
        }
    }
}

impl RefreshTokenClaims {
    pub fn new(user_id: UserId, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            exp: now + expires_in,
            iat: now,
            token_type: "refresh".to_string(),
        }
    }
}
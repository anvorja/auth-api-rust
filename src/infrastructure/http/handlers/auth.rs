// src/infrastructure/http/handlers/auth.rs
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use std::sync::Arc;
use time::Duration;
use validator::Validate;

use crate::application::AuthUseCase;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::presentation::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, SuccessResponse};

/// Handler para registro de usuario
///
/// POST /api/v1/auth/register
///
/// Crea un nuevo usuario y retorna tokens JWT en cookies HttpOnly
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Usuario registrado exitosamente", body = AuthResponse),
        (status = 400, description = "Validación fallida"),
        (status = 409, description = "Usuario ya existe")
    ),
    tag = "Authentication"
)]
pub async fn register_handler<R, P, J, S>(
    State(state): State<S>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + AsRef<Arc<J>> + Clone + Send + Sync + 'static,
{
    // Validar el request
    request.validate()?;

    // Obtener servicios del state
    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();
    let jwt_service: &Arc<J> = state.as_ref();

    // Ejecutar caso de uso
    let (user, access_token, refresh_token) = auth_usecase.register_user(request).await?;

    // Crear cookies HttpOnly
    let access_cookie = create_access_token_cookie(&access_token, jwt_service.get_access_token_expiry());
    let refresh_cookie = create_refresh_token_cookie(&refresh_token, jwt_service.get_refresh_token_expiry());

    // Crear respuesta
    let response = AuthResponse::new(
        access_token.clone(),
        refresh_token.clone(),
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, access_cookie.to_string()), (header::SET_COOKIE, refresh_cookie.to_string())],
        Json(response),
    ))
}

/// Handler para login de usuario
///
/// POST /api/v1/auth/login
///
/// Autentica al usuario y retorna tokens JWT en cookies HttpOnly
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = AuthResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "Credenciales inválidas")
    ),
    tag = "Authentication"
)]
pub async fn login_handler<R, P, J, S>(
    State(state): State<S>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + AsRef<Arc<J>> + Clone + Send + Sync + 'static,
{
    // Validar el request
    request.validate()?;

    // Obtener servicios del state
    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();
    let jwt_service: &Arc<J> = state.as_ref();

    // Ejecutar caso de uso
    let (user, access_token, refresh_token) = auth_usecase.login_user(request).await?;

    // Crear cookies HttpOnly
    let access_cookie = create_access_token_cookie(&access_token, jwt_service.get_access_token_expiry());
    let refresh_cookie = create_refresh_token_cookie(&refresh_token, jwt_service.get_refresh_token_expiry());

    // Crear respuesta
    let response = AuthResponse::new(
        access_token.clone(),
        refresh_token.clone(),
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, access_cookie.to_string()), (header::SET_COOKIE, refresh_cookie.to_string())],
        Json(response),
    ))
}

/// Handler para refresh token
///
/// POST /api/v1/auth/refresh
///
/// Renueva los tokens JWT usando un refresh token válido
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Tokens renovados", body = AuthResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "Refresh token inválido")
    ),
    tag = "Authentication"
)]
pub async fn refresh_handler<R, P, J, S>(
    State(state): State<S>,
    Json(request): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + AsRef<Arc<J>> + Clone + Send + Sync + 'static,
{
    // Validar el request
    request.validate()?;

    // Obtener servicios del state
    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();
    let jwt_service: &Arc<J> = state.as_ref();

    // Ejecutar caso de uso
    let (user, access_token, refresh_token) = auth_usecase
        .refresh_token(&request.refresh_token)
        .await?;

    // Crear cookies HttpOnly
    let access_cookie = create_access_token_cookie(&access_token, jwt_service.get_access_token_expiry());
    let refresh_cookie = create_refresh_token_cookie(&refresh_token, jwt_service.get_refresh_token_expiry());

    // Crear respuesta
    let response = AuthResponse::new(
        access_token.clone(),
        refresh_token.clone(),
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, access_cookie.to_string()), (header::SET_COOKIE, refresh_cookie.to_string())],
        Json(response),
    ))
}

/// Handler para logout
///
/// POST /api/v1/auth/logout
///
/// Invalida las cookies de autenticación
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logout exitoso", body = SuccessResponse)
    ),
    tag = "Authentication"
)]
pub async fn logout_handler() -> impl IntoResponse {
    // Crear cookies expiradas para eliminarlas
    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(Duration::seconds(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(Duration::seconds(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let response = SuccessResponse {
        message: "Logout exitoso".to_string(),
    };

    (
        StatusCode::OK,
        [(header::SET_COOKIE, access_cookie.to_string()), (header::SET_COOKIE, refresh_cookie.to_string())],
        Json(response),
    )
}

// ============================================================================
// Helper functions para crear cookies
// ============================================================================

/// Crea una cookie HttpOnly para el access token
fn create_access_token_cookie(token: &str, max_age_seconds: i64) -> Cookie<'static> {
    Cookie::build(("access_token", token.to_string()))
        .path("/")
        .max_age(Duration::seconds(max_age_seconds))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(false) // En producción debería ser true (HTTPS)
        .build()
}

/// Crea una cookie HttpOnly para el refresh token
fn create_refresh_token_cookie(token: &str, max_age_seconds: i64) -> Cookie<'static> {
    Cookie::build(("refresh_token", token.to_string()))
        .path("/")
        .max_age(Duration::seconds(max_age_seconds))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(false) // En producción debería ser true (HTTPS)
        .build()
}
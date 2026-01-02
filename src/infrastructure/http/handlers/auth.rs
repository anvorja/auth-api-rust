// src/infrastructure/http/handlers/auth.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use validator::Validate;

use crate::application::AuthUseCase;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::presentation::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, SuccessResponse};

/// Handler para registro de usuario
///
/// POST /api/v1/auth/register
///
/// Retorna tokens JWT en el body (NO en cookies)
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

    // Crear respuesta (SOLO JSON, sin cookies)
    let response = AuthResponse::new(
        access_token,
        refresh_token,
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler para login de usuario
///
/// POST /api/v1/auth/login
///
/// Retorna tokens JWT en el body (NO en cookies)
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

    // Crear respuesta (SOLO JSON, sin cookies)
    let response = AuthResponse::new(
        access_token,
        refresh_token,
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((StatusCode::OK, Json(response)))
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

    // Crear respuesta (SOLO JSON, sin cookies)
    let response = AuthResponse::new(
        access_token,
        refresh_token,
        jwt_service.get_access_token_expiry(),
        &user,
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Handler para logout
///
/// POST /api/v1/auth/logout
///
/// En arquitectura stateless con Bearer tokens, el logout es client-side
/// (el cliente simplemente elimina los tokens)
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logout exitoso", body = SuccessResponse)
    ),
    tag = "Authentication"
)]
pub async fn logout_handler() -> impl IntoResponse {
    let response = SuccessResponse {
        message: "Logout exitoso. Elimina los tokens del lado del cliente.".to_string(),
    };

    (StatusCode::OK, Json(response))
}
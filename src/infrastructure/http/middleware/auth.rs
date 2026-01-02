// src/infrastructure/http/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::application::AuthUseCase;
use crate::domain::User;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};

/// Extension que contiene el usuario autenticado
#[derive(Clone)]
pub struct AuthenticatedUser(pub User);

/// Middleware de autenticación - Solo Bearer Token
///
/// Extrae el token del header: Authorization: Bearer <token>
pub async fn auth_middleware<R, P, J>(
    State(auth_usecase): State<Arc<AuthUseCase<R, P, J>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
{
    // Extraer token del header Authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| {
            tracing::warn!("❌ Header Authorization no encontrado");
            AppError::Unauthorized
        })?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| {
            tracing::warn!("❌ Header Authorization con formato inválido");
            AppError::Unauthorized
        })?;

    // Verificar formato "Bearer <token>"
    let access_token = auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            tracing::warn!("❌ Header Authorization sin prefijo 'Bearer '");
            AppError::Unauthorized
        })?;

    tracing::debug!("🔑 Token Bearer extraído correctamente");

    // Validar el token y obtener el usuario
    let user = auth_usecase
        .validate_access_token(access_token)
        .await?;

    // Añadir el usuario a las extensions del request
    request.extensions_mut().insert(AuthenticatedUser(user));

    // Continuar con el siguiente middleware/handler
    Ok(next.run(request).await)
}

/// Extractor personalizado para obtener el usuario autenticado
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Usuario no autenticado. Debe incluir: Authorization: Bearer <token>",
            ))
    }
}
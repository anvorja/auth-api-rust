// V1
// src/infrastructure/http/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;

use crate::application::AuthUseCase;
use crate::domain::User;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};

/// Extension que contiene el usuario autenticado
///
/// Los handlers pueden extraer esto para obtener el usuario actual:
/// ```
/// async fn protected_handler(
///     Extension(user): Extension<User>,
/// ) -> impl IntoResponse {
///     // user está autenticado
/// }
/// ```
#[derive(Clone)]
pub struct AuthenticatedUser(pub User);

/// Middleware de autenticación
///
/// Valida el access token de las cookies y añade el usuario a las extensions
///
/// Si el token es inválido o no existe, retorna 401 Unauthorized
pub async fn auth_middleware<R, P, J>(
    State(auth_usecase): State<Arc<AuthUseCase<R, P, J>>>,
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
{
    // Extraer access token de las cookies
    let access_token = cookie_jar
        .get("access_token")
        .ok_or(AppError::Unauthorized)?
        .value();

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
///
/// Uso en handlers:
/// ```
/// use axum::extract::Extension;
/// use crate::infrastructure::http::middleware::AuthenticatedUser;
///
/// async fn my_handler(
///     Extension(AuthenticatedUser(user)): Extension<AuthenticatedUser>,
/// ) -> impl IntoResponse {
///     format!("Hola, {}!", user.username().value())
/// }
/// ```
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
                "Usuario no autenticado. Debes pasar por el middleware de autenticación.",
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_clone() {
        use crate::domain::{Email, PasswordHash, User, Username};

        let user = User::new(
            Username::new("testuser".to_string()).unwrap(),
            Email::new("test@example.com".to_string()).unwrap(),
            "Test".to_string(),
            "User".to_string(),
            PasswordHash::from_hash("hash".to_string()),
        );

        let auth_user = AuthenticatedUser(user.clone());
        let cloned = auth_user.clone();

        assert_eq!(auth_user.0.id(), cloned.0.id());
    }
}
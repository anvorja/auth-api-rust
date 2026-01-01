// src/infrastructure/http/handlers/user.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use validator::Validate;

use crate::application::AuthUseCase;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::infrastructure::http::middleware::AuthenticatedUser;
use crate::presentation::{ChangePasswordRequest, SuccessResponse, UserResponse};
use std::sync::Arc;

/// Handler para obtener perfil del usuario autenticado
///
/// GET /api/v1/users/profile
///
/// Requiere autenticación (access token en cookies)
#[utoipa::path(
    get,
    path = "/api/v1/users/profile",
    responses(
        (status = 200, description = "Perfil del usuario", body = UserResponse),
        (status = 401, description = "No autenticado")
    ),
    tag = "Users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile_handler<R, P, J, S>(
    Extension(AuthenticatedUser(user)): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: Clone + Send + Sync + 'static,
{
    // El usuario ya está autenticado gracias al middleware
    let response = UserResponse::from_domain(&user);

    Ok((StatusCode::OK, Json(response)))
}

/// Handler para cambiar contraseña
///
/// POST /api/v1/users/change-password
///
/// Requiere autenticación (access token en cookies)
#[utoipa::path(
    post,
    path = "/api/v1/users/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Contraseña cambiada exitosamente", body = SuccessResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "No autenticado o contraseña actual incorrecta")
    ),
    tag = "Users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(user)): Extension<AuthenticatedUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    // Validar el request
    request.validate()?;

    // Verificar que las contraseñas coincidan
    if !request.passwords_match() {
        return Err(AppError::ValidationError(
            "Las contraseñas no coinciden".to_string(),
        ));
    }

    // Obtener el use case del state
    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    // Cambiar la contraseña
    auth_usecase
        .change_password(
            user.id(),
            &request.current_password,
            &request.new_password,
        )
        .await?;

    let response = SuccessResponse {
        message: "Contraseña actualizada exitosamente".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}
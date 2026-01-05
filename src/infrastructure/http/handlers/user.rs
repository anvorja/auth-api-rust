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
use crate::presentation::{AdminUpdateEmailRequest, AdminUpdateUsernameRequest, ChangePasswordRequest, SuccessResponse, UpdateProfileRequest, UserResponse};
use std::sync::Arc;
use crate::domain::UserId;

/// Handler para obtener perfil del usuario autenticado
///
/// GET /api/v1/users/profile
///
/// Requiere autenticación (access token desde Authorization header)
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

/// Handler para actualizar perfil
///
/// PUT /api/v1/users/profile
#[utoipa::path(
    put,
    path = "/api/v1/users/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Perfil actualizado", body = UserResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "No autenticado")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn update_profile_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(user)): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    request.validate()?;

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    let updated_user = auth_usecase
        .update_profile(
            user.id(),
            request.first_name,
            request.last_name,
        )
        .await?;

    let response = UserResponse::from_domain(&updated_user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handler para eliminar cuenta
///
/// DELETE /api/v1/users/account
#[utoipa::path(
    delete,
    path = "/api/v1/users/account",
    responses(
        (status = 200, description = "Cuenta eliminada", body = SuccessResponse),
        (status = 401, description = "No autenticado")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn delete_account_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(user)): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    auth_usecase.delete_account(user.id()).await?;

    let response = SuccessResponse {
        message: "Cuenta eliminada exitosamente".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Handler para admin actualizar username
///
/// PUT /api/v1/admin/users/username
#[utoipa::path(
    put,
    path = "/api/v1/admin/users/username",
    request_body = AdminUpdateUsernameRequest,
    responses(
        (status = 200, description = "Username actualizado", body = UserResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_update_username_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Json(request): Json<AdminUpdateUsernameRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    request.validate()?;

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    let target_user_id = UserId::from_string(&request.user_id)
        .map_err(|e| AppError::ValidationError(e))?;

    let updated_user = auth_usecase
        .admin_update_username(
            admin.id(),
            target_user_id,
            request.new_username,
        )
        .await?;

    let response = UserResponse::from_domain(&updated_user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handler para admin actualizar email
///
/// PUT /api/v1/admin/users/email
#[utoipa::path(
    put,
    path = "/api/v1/admin/users/email",
    request_body = AdminUpdateEmailRequest,
    responses(
        (status = 200, description = "Email actualizado", body = UserResponse),
        (status = 400, description = "Validación fallida"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_update_email_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Json(request): Json<AdminUpdateEmailRequest>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    request.validate()?;

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    let target_user_id = UserId::from_string(&request.user_id)
        .map_err(|e| AppError::ValidationError(e))?;

    let updated_user = auth_usecase
        .admin_update_email(
            admin.id(),
            target_user_id,
            request.new_email,
        )
        .await?;

    let response = UserResponse::from_domain(&updated_user);
    Ok((StatusCode::OK, Json(response)))
}
// src/infrastructure/http/handlers/user.rs
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use validator::Validate;
use serde::Deserialize;

use crate::application::AuthUseCase;
use crate::error::AppError;
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::infrastructure::http::middleware::AuthenticatedUser;
use crate::presentation::{AdminUpdateEmailRequest, AdminUpdateUsernameRequest, ChangePasswordRequest, SuccessResponse, UpdateProfileRequest, UserResponse};
use std::sync::Arc;
use crate::domain::{UserId, Username, Email};

/// Query parameter para búsqueda de usuario
#[derive(Debug, Deserialize)]
pub struct SearchUserQuery {
    pub identifier: String,
}

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
/// PATCH /api/v1/users/profile
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
    // Validar formato de campos (si están presentes)
    request.validate()?;

    // Validar que al menos un campo esté presente
    if !request.has_updates() {
        return Err(AppError::ValidationError(
            "Debe proporcionar al menos un campo para actualizar (first_name o last_name)".to_string()
        ));
    }

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

/// Handler para buscar usuario por username o email
///
/// GET /api/v1/admin/users/search?identifier={username_or_email}
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/search",
    params(
        ("identifier" = String, Query, description = "Username o email del usuario a buscar")
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)"),
        (status = 404, description = "Usuario no encontrado")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_search_user_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Query(query): Query<SearchUserQuery>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    // Verificar que es admin
    if !admin.is_admin() {
        tracing::warn!("Intento de búsqueda admin por usuario no autorizado: {}", admin.id().to_string());
        return Err(AppError::Unauthorized);
    }

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    // Buscar por username o email usando el método del usecase
    let user = auth_usecase
        .find_user_by_username_or_email(&query.identifier)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let response = UserResponse::from_domain(&user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handler para obtener usuario por ID (admin)
///
/// GET /api/v1/admin/users/{user_id}
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "UUID del usuario")
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 400, description = "UUID inválido"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)"),
        (status = 404, description = "Usuario no encontrado")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_get_user_by_id_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    // Verificar que es admin
    if !admin.is_admin() {
        tracing::warn!("Intento de obtener usuario por ID por usuario no autorizado: {}", admin.id().to_string());
        return Err(AppError::Unauthorized);
    }

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    // Parsear y validar UUID
    let target_user_id = UserId::from_string(&user_id)
        .map_err(|e| AppError::ValidationError(e))?;

    // Obtener usuario
    let user = auth_usecase
        .get_user_by_id(target_user_id)
        .await?;

    let response = UserResponse::from_domain(&user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handler para obtener usuario por username (admin)
///
/// GET /api/v1/admin/users/by-username/{username}
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/by-username/{username}",
    params(
        ("username" = String, Path, description = "Username del usuario")
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 400, description = "Username inválido"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)"),
        (status = 404, description = "Usuario no encontrado")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_get_user_by_username_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    // Verificar que es admin
    if !admin.is_admin() {
        tracing::warn!("Intento de obtener usuario por username por usuario no autorizado: {}", admin.id().to_string());
        return Err(AppError::Unauthorized);
    }

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    // Validar y parsear username
    let username_vo = Username::new(username)
        .map_err(|e| AppError::ValidationError(e))?;

    // Buscar usuario usando el método del usecase
    let user = auth_usecase
        .find_user_by_username(&username_vo)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let response = UserResponse::from_domain(&user);
    Ok((StatusCode::OK, Json(response)))
}

/// Handler para obtener usuario por email (admin)
///
/// GET /api/v1/admin/users/by-email/{email}
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/by-email/{email}",
    params(
        ("email" = String, Path, description = "Email del usuario")
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 400, description = "Email inválido"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No autorizado (no es admin)"),
        (status = 404, description = "Usuario no encontrado")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_get_user_by_email_handler<R, P, J, S>(
    State(state): State<S>,
    Extension(AuthenticatedUser(admin)): Extension<AuthenticatedUser>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    R: UserRepository + 'static,
    P: PasswordHasher + 'static,
    J: JwtService + 'static,
    S: AsRef<Arc<AuthUseCase<R, P, J>>> + Clone + Send + Sync + 'static,
{
    // Verificar que es admin
    if !admin.is_admin() {
        tracing::warn!("Intento de obtener usuario por email por usuario no autorizado: {}", admin.id().to_string());
        return Err(AppError::Unauthorized);
    }

    let auth_usecase: &Arc<AuthUseCase<R, P, J>> = state.as_ref();

    // Validar y parsear email
    let email_vo = Email::new(email)
        .map_err(|e| AppError::ValidationError(e))?;

    // Buscar usuario
    let user = auth_usecase
        .find_user_by_email(&email_vo)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let response = UserResponse::from_domain(&user);
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
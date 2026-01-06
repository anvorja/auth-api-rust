// src/application/auth_usecase.rs
use std::sync::Arc;

use crate::domain::{Email, User, UserId, Username};
use crate::error::{AppError, AppResult};
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::presentation::{LoginRequest, RegisterRequest};

/// Casos de uso de autenticación
///
/// Esta capa orquesta la lógica de negocio:
/// - Validaciones de dominio
/// - Coordinación de repositorios
/// - Hashing de passwords
/// - Generación de tokens
///
/// NO contiene lógica de HTTP (eso va en handlers)
/// NO contiene lógica de DB (eso va en repositories)
pub struct AuthUseCase<R, P, J>
where
    R: UserRepository,
    P: PasswordHasher,
    J: JwtService,
{
    user_repository: Arc<R>,
    password_hasher: Arc<P>,
    jwt_service: Arc<J>,
}

impl<R, P, J> AuthUseCase<R, P, J>
where
    R: UserRepository,
    P: PasswordHasher,
    J: JwtService,
{
    /// Crea una nueva instancia del caso de uso
    pub fn new(
        user_repository: Arc<R>,
        password_hasher: Arc<P>,
        jwt_service: Arc<J>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            jwt_service,
        }
    }

    /// Getter para el repositorio (usado principalmente en tests)
    pub(crate) fn user_repository(&self) -> &Arc<R> {
        &self.user_repository
    }

    /// Caso de uso: Registrar nuevo usuario
    ///
    /// Pasos:
    /// 1. Validar datos del request (ya validados por DTOs)
    /// 2. Convertir a tipos de dominio
    /// 3. Verificar que username y email no existan
    /// 4. Hashear el password
    /// 5. Crear entidad User
    /// 6. Persistir en base de datos
    /// 7. Generar tokens JWT
    pub async fn register_user(
        &self,
        request: RegisterRequest,
    ) -> AppResult<(User, String, String)> {
        tracing::info!("Registrando usuario: {}", request.username);

        // 1. Convertir a tipos de dominio (con validaciones)
        let (username, email) = request.to_domain_types()
            .map_err(|e| AppError::ValidationError(e))?;

        // 2. Verificar que el username no exista
        if self.user_repository.exists_by_username(&username).await? {
            tracing::warn!("Intento de registro con username existente: {}", username.value());
            return Err(AppError::UserAlreadyExists);
        }

        // 3. Verificar que el email no exista
        if self.user_repository.exists_by_email(&email).await? {
            tracing::warn!("Intento de registro con email existente: {}", email.value());
            return Err(AppError::UserAlreadyExists);
        }

        // 4. Hashear el password
        let password_hash = self.password_hasher.hash_password(&request.password).await?;

        // 5. Crear entidad de dominio User
        let user = User::new(
            username,
            email,
            request.first_name,
            request.last_name,
            password_hash,
        );

        // 6. Persistir en base de datos
        self.user_repository.create(&user).await?;

        // 7. Generar tokens JWT
        let access_token = self.jwt_service.generate_access_token(
            user.id(),
            user.username().value().to_string(),
        )?;

        let refresh_token = self.jwt_service.generate_refresh_token(user.id())?;

        tracing::info!("Usuario registrado exitosamente: {}", user.username().value());

        Ok((user, access_token, refresh_token))
    }

    /// Caso de uso: Login de usuario
    ///
    /// Pasos:
    /// 1. Buscar usuario por username o email
    /// 2. Verificar que el usuario existe
    /// 3. Verificar el password
    /// 4. Generar tokens JWT
    pub async fn login_user(
        &self,
        request: LoginRequest,
    ) -> AppResult<(User, String, String)> {
        tracing::info!("Intento de login: {}", request.username);

        // 1. Buscar usuario por username o email
        let user = self.user_repository
            .find_by_username_or_email(&request.username)
            .await?
            .ok_or_else(|| {
                tracing::warn!("Login fallido: usuario no encontrado '{}'", request.username);
                AppError::InvalidCredentials
            })?;

        // 2. Verificar el password
        let is_valid = self.password_hasher
            .verify_password(&request.password, user.password_hash())
            .await?;

        if !is_valid {
            tracing::warn!("Login fallido: password incorrecta para '{}'", request.username);
            return Err(AppError::InvalidCredentials);
        }

        // 3. Generar tokens JWT
        let access_token = self.jwt_service.generate_access_token(
            user.id(),
            user.username().value().to_string(),
        )?;

        let refresh_token = self.jwt_service.generate_refresh_token(user.id())?;

        tracing::info!("Login exitoso: {}", user.username().value());

        Ok((user, access_token, refresh_token))
    }

    /// Caso de uso: Refresh token
    ///
    /// Pasos:
    /// 1. Validar el refresh token
    /// 2. Extraer user_id del token
    /// 3. Buscar el usuario
    /// 4. Generar nuevos tokens
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> AppResult<(User, String, String)> {
        tracing::debug!("Renovando tokens");

        // 1. Validar y decodificar el refresh token
        let claims = self.jwt_service.validate_refresh_token(refresh_token)?;

        // 2. Extraer user_id
        let user_id = UserId::from_string(&claims.sub)
            .map_err(|e| {
                tracing::error!("Invalid user_id in refresh token: {}", e);
                AppError::InvalidRefreshToken
            })?;

        // 3. Buscar el usuario (verificar que aún existe)
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("Refresh token para usuario inexistente: {}", user_id.to_string());
                AppError::InvalidRefreshToken
            })?;

        // 4. Generar nuevos tokens
        let new_access_token = self.jwt_service.generate_access_token(
            user.id(),
            user.username().value().to_string(),
        )?;

        let new_refresh_token = self.jwt_service.generate_refresh_token(user.id())?;

        tracing::info!("Tokens renovados para: {}", user.username().value());

        Ok((user, new_access_token, new_refresh_token))
    }

    /// Caso de uso: Validar access token y obtener usuario
    ///
    /// Útil para middleware de autenticación
    pub async fn validate_access_token(&self, token: &str) -> AppResult<User> {
        // 1. Validar y decodificar el access token
        let claims = self.jwt_service.validate_access_token(token)?;

        // 2. Extraer user_id
        let user_id = UserId::from_string(&claims.sub)
            .map_err(|e| {
                tracing::error!("Invalid user_id in access token: {}", e);
                AppError::InvalidToken
            })?;

        // 3. Buscar el usuario
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| {
                tracing::warn!("Access token para usuario inexistente: {}", user_id.to_string());
                AppError::InvalidToken
            })?;

        Ok(user)
    }

    /// Caso de uso: Obtener usuario por ID
    ///
    /// Para endpoints que requieren información del usuario autenticado
    pub async fn get_user_by_id(&self, user_id: UserId) -> AppResult<User> {
        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)
    }

    /// Caso de uso: Cambiar password
    ///
    /// Pasos:
    /// 1. Buscar usuario
    /// 2. Verificar password actual
    /// 3. Hashear nuevo password
    /// 4. Actualizar usuario
    pub async fn change_password(
        &self,
        user_id: UserId,
        current_password: &str,
        new_password: &str,
    ) -> AppResult<()> {
        tracing::info!("Cambiando password para usuario: {}", user_id.to_string());

        // 1. Buscar usuario
        let mut user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // 2. Verificar password actual
        let is_valid = self.password_hasher
            .verify_password(current_password, user.password_hash())
            .await?;

        if !is_valid {
            tracing::warn!("Cambio de password fallido: password actual incorrecta");
            return Err(AppError::InvalidCredentials);
        }

        // 3. Hashear nuevo password
        let new_password_hash = self.password_hasher.hash_password(new_password).await?;

        // 4. Actualizar usuario
        user.update_password(new_password_hash);
        self.user_repository.update(&user).await?;

        tracing::info!("Password actualizada exitosamente");

        Ok(())
    }

    /// Caso de uso: Actualizar perfil (nombre y/o apellido)
    pub async fn update_profile(
        &self,
        user_id: UserId,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> AppResult<User> {
        tracing::info!("Actualizando perfil para usuario: {}", user_id.to_string());

        // Buscar usuario
        let mut user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // Aplicar solo los cambios proporcionados
        let new_first_name = first_name.unwrap_or_else(|| user.first_name().to_string());
        let new_last_name = last_name.unwrap_or_else(|| user.last_name().to_string());

        // Actualizar perfil
        user.update_profile(new_first_name, new_last_name);

        // Guardar cambios
        self.user_repository.update(&user).await?;

        tracing::info!("Perfil actualizado exitosamente");
        Ok(user)
    }

    /// Caso de uso: Eliminar cuenta (el usuario elimina su propia cuenta)
    pub async fn delete_account(&self, user_id: UserId) -> AppResult<()> {
        tracing::info!("Eliminando cuenta: {}", user_id.to_string());

        // Verificar que el usuario existe
        let user = self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // Eliminar usuario
        self.user_repository.delete(user.id()).await?;

        tracing::info!("Cuenta eliminada exitosamente: {}", user.username().value());
        Ok(())
    }

    /// Caso de uso: Admin actualiza username de un usuario
    pub async fn admin_update_username(
        &self,
        admin_id: UserId,
        target_user_id: UserId,
        new_username: String,
    ) -> AppResult<User> {
        tracing::info!("Admin {} actualizando username de usuario {}",
        admin_id.to_string(), target_user_id.to_string());

        // Verificar que quien hace la petición es admin
        let admin = self.user_repository
            .find_by_id(admin_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        if !admin.is_admin() {
            tracing::warn!("Intento de operación admin por usuario no autorizado: {}",
            admin_id.to_string());
            return Err(AppError::Unauthorized);
        }

        // Buscar usuario objetivo
        let mut user = self.user_repository
            .find_by_id(target_user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // Validar nuevo username
        let username = Username::new(new_username)
            .map_err(|e| AppError::ValidationError(e))?;

        // Verificar que el username no esté en uso
        if self.user_repository.exists_by_username(&username).await? {
            return Err(AppError::UserAlreadyExists);
        }

        // Actualizar username
        user.update_username(username);
        self.user_repository.update(&user).await?;

        tracing::info!("Username actualizado exitosamente por admin");
        Ok(user)
    }

    /// Caso de uso: Admin actualiza email de un usuario
    pub async fn admin_update_email(
        &self,
        admin_id: UserId,
        target_user_id: UserId,
        new_email: String,
    ) -> AppResult<User> {
        tracing::info!("Admin {} actualizando email de usuario {}",
        admin_id.to_string(), target_user_id.to_string());

        // Verificar que quien hace la petición es admin
        let admin = self.user_repository
            .find_by_id(admin_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        if !admin.is_admin() {
            tracing::warn!("Intento de operación admin por usuario no autorizado: {}",
            admin_id.to_string());
            return Err(AppError::Unauthorized);
        }

        // Buscar usuario objetivo
        let mut user = self.user_repository
            .find_by_id(target_user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // Validar nuevo email
        let email = Email::new(new_email)
            .map_err(|e| AppError::ValidationError(e))?;

        // Verificar que el email no esté en uso
        if self.user_repository.exists_by_email(&email).await? {
            return Err(AppError::UserAlreadyExists);
        }

        // Actualizar email
        user.update_email(email);
        self.user_repository.update(&user).await?;

        tracing::info!("Email actualizado exitosamente por admin");
        Ok(user)
    }

    // ==========================================================================
    // MÉTODOS DE BÚSQUEDA (para endpoints admin)
    // ==========================================================================

    /// Caso de uso: Buscar usuario por username o email
    ///
    /// Usado por admin para buscar usuarios
    pub async fn find_user_by_username_or_email(&self, identifier: &str) -> AppResult<Option<User>> {
        self.user_repository
            .find_by_username_or_email(identifier)
            .await
    }

    /// Caso de uso: Buscar usuario por username
    ///
    /// Usado por admin para buscar usuarios específicamente por username
    pub async fn find_user_by_username(&self, username: &Username) -> AppResult<Option<User>> {
        self.user_repository
            .find_by_username(username)
            .await
    }

    /// Caso de uso: Buscar usuario por email
    ///
    /// Usado por admin para buscar usuarios específicamente por email
    pub async fn find_user_by_email(&self, email: &Email) -> AppResult<Option<User>> {
        self.user_repository
            .find_by_email(email)
            .await
    }
}

// Implementar Clone manualmente para evitar restricciones del compilador
impl<R, P, J> Clone for AuthUseCase<R, P, J>
where
    R: UserRepository,
    P: PasswordHasher,
    J: JwtService,
{
    fn clone(&self) -> Self {
        Self {
            user_repository: Arc::clone(&self.user_repository),
            password_hasher: Arc::clone(&self.password_hasher),
            jwt_service: Arc::clone(&self.jwt_service),
        }
    }
}
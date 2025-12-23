// V1
// src/application/auth_usecase.rs
use std::sync::Arc;

use crate::domain::{Email, PasswordHash, User, UserId, Username};
use crate::error::{AppError, AppResult};
use crate::infrastructure::{JwtService, PasswordHasher, UserRepository};
use crate::presentation::{LoginRequest, RefreshTokenClaims, RegisterRequest};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx};
    use crate::infrastructure::db::create_pool;
    use crate::config::Settings;

    async fn create_test_auth_usecase() -> AuthUseCase<UserRepositorySqlx, ArgonPasswordHasher, JwtServiceImpl> {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test");
            std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
        }

        let settings = Settings::from_env().expect("Failed to load test settings");
        let pool = create_pool(&settings).await.expect("Failed to create pool");

        let user_repository = Arc::new(UserRepositorySqlx::new(pool));
        let password_hasher = Arc::new(ArgonPasswordHasher::new_for_testing());
        let jwt_service = Arc::new(JwtServiceImpl::new(
            settings.jwt.secret,
            settings.jwt.access_token_expiry,
            settings.jwt.refresh_token_expiry,
        ));

        AuthUseCase::new(user_repository, password_hasher, jwt_service)
    }

    fn create_test_register_request() -> RegisterRequest {
        RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "SecurePass123!".to_string(),
        }
    }

    #[tokio::test]
    #[ignore] // Requiere PostgreSQL
    async fn test_register_user() {
        let usecase = create_test_auth_usecase().await;
        let request = create_test_register_request();

        let result = usecase.register_user(request).await;
        assert!(result.is_ok());

        let (user, access_token, refresh_token) = result.unwrap();
        assert_eq!(user.username().value(), "testuser");
        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());

        // Cleanup
        let _ = usecase.user_repository.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_login_user() {
        let usecase = create_test_auth_usecase().await;
        let register_request = create_test_register_request();

        // Primero registrar
        let (user, _, _) = usecase.register_user(register_request).await.unwrap();

        // Luego login
        let login_request = LoginRequest {
            username: "testuser".to_string(),
            password: "SecurePass123!".to_string(),
        };

        let result = usecase.login_user(login_request).await;
        assert!(result.is_ok());

        // Cleanup
        let _ = usecase.user_repository.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_login_wrong_password() {
        let usecase = create_test_auth_usecase().await;
        let register_request = create_test_register_request();

        let (user, _, _) = usecase.register_user(register_request).await.unwrap();

        let login_request = LoginRequest {
            username: "testuser".to_string(),
            password: "WrongPassword123!".to_string(),
        };

        let result = usecase.login_user(login_request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidCredentials));

        // Cleanup
        let _ = usecase.user_repository.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_refresh_token() {
        let usecase = create_test_auth_usecase().await;
        let register_request = create_test_register_request();

        let (user, _, refresh_token) = usecase.register_user(register_request).await.unwrap();

        let result = usecase.refresh_token(&refresh_token).await;
        assert!(result.is_ok());

        let (_, new_access, new_refresh) = result.unwrap();
        assert!(!new_access.is_empty());
        assert!(!new_refresh.is_empty());
        assert_ne!(refresh_token, new_refresh); // Los tokens deben ser diferentes

        // Cleanup
        let _ = usecase.user_repository.delete(user.id()).await;
    }
}
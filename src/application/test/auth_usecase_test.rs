// src/application/test/auth_usecase_test.rs
use crate::application::AuthUseCase;
use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx, UserRepository};
use crate::infrastructure::db::create_pool;
use crate::config::Settings;
use crate::presentation::{LoginRequest, RegisterRequest};
use crate::error::AppError;
use std::sync::Arc;

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
    let _ = usecase.user_repository().delete(user.id()).await;
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
    let _ = usecase.user_repository().delete(user.id()).await;
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
    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_refresh_token() {
    let usecase = create_test_auth_usecase().await;
    let register_request = create_test_register_request();

    let (user, _, refresh_token) = usecase.register_user(register_request).await.unwrap();

    // Pequeña pausa para asegurar que el timestamp sea diferente
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let result = usecase.refresh_token(&refresh_token).await;
    assert!(result.is_ok());

    let (_, new_access, new_refresh) = result.unwrap();
    assert!(!new_access.is_empty());
    assert!(!new_refresh.is_empty());
    assert_ne!(refresh_token, new_refresh); // Los tokens deben ser diferentes

    // Cleanup
    let _ = usecase.user_repository().delete(user.id()).await;
}
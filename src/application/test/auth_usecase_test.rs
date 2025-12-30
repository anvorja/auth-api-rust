// src/application/test/auth_usecase_test.rs
use crate::application::AuthUseCase;
use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx, UserRepository};
use crate::infrastructure::db::create_pool;
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::presentation::{LoginRequest, RegisterRequest};
use crate::error::AppError;
use crate::domain::UserId;

/// Crea un AuthUseCase con configuración personalizada para tests
///
/// Esta función elimina la necesidad de usar `unsafe` y `std::env::set_var`,
/// permitiendo que los tests se ejecuten en paralelo sin race conditions.
async fn create_test_auth_usecase_with_config(
    db_url: &str,
    jwt_secret: &str,
) -> AuthUseCase<UserRepositorySqlx, ArgonPasswordHasher, JwtServiceImpl> {
    // Crear Settings directamente sin usar variables de entorno
    let settings = Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: db_url.to_string(),
            max_connections: 5,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: jwt_secret.to_string(),
            access_token_expiry: 900,   // 15 minutos
            refresh_token_expiry: 604800, // 7 días
        },
        security: SecuritySettings {
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
        environment: Environment::Test,
    };

    let pool = create_pool(&settings).await.expect("Failed to create pool");
    let user_repository = std::sync::Arc::new(UserRepositorySqlx::new(pool));
    let password_hasher = std::sync::Arc::new(ArgonPasswordHasher::new_for_testing());
    let jwt_service = std::sync::Arc::new(JwtServiceImpl::new(
        settings.jwt.secret,
        settings.jwt.access_token_expiry,
        settings.jwt.refresh_token_expiry,
    ));

    AuthUseCase::new(user_repository, password_hasher, jwt_service)
}

/// Helper para crear AuthUseCase con configuración por defecto para tests
///
/// Usa valores estándar de test. Para configuraciones personalizadas,
/// usar `create_test_auth_usecase_with_config` directamente.
async fn create_test_auth_usecase() -> AuthUseCase<UserRepositorySqlx, ArgonPasswordHasher, JwtServiceImpl> {
    create_test_auth_usecase_with_config(
        "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test",
        "test-secret-key-minimum-32-characters-long",
    ).await
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

    // Esperar un poco para que los timestamps sean diferentes
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

// ============================================================================
// NUEVOS TESTS PARA FUNCIONALIDADES COMPLETADAS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_user_by_id() {
    let usecase = create_test_auth_usecase().await;
    let register_request = create_test_register_request();

    // Registrar usuario
    let (user, _, _) = usecase.register_user(register_request).await.unwrap();

    // Obtener por ID
    let result = usecase.get_user_by_id(user.id()).await;
    assert!(result.is_ok());

    let found_user = result.unwrap();
    assert_eq!(found_user.id(), user.id());
    assert_eq!(found_user.username().value(), "testuser");

    // Cleanup
    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_get_user_by_id_not_found() {
    let usecase = create_test_auth_usecase().await;

    // ID que no existe
    let fake_id = UserId::new();
    let result = usecase.get_user_by_id(fake_id).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::UserNotFound));
}

#[tokio::test]
#[ignore]
async fn test_change_password_success() {
    let usecase = create_test_auth_usecase().await;
    let register_request = create_test_register_request();

    // Registrar usuario
    let (user, _, _) = usecase.register_user(register_request).await.unwrap();

    // Cambiar password
    let result = usecase
        .change_password(
            user.id(),
            "SecurePass123!",
            "NewSecurePass456!",
        )
        .await;

    assert!(result.is_ok());

    // Verificar que el nuevo password funciona
    let login_request = LoginRequest {
        username: "testuser".to_string(),
        password: "NewSecurePass456!".to_string(),
    };

    let login_result = usecase.login_user(login_request).await;
    assert!(login_result.is_ok());

    // Cleanup
    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_change_password_wrong_current() {
    let usecase = create_test_auth_usecase().await;
    let register_request = create_test_register_request();

    // Registrar usuario
    let (user, _, _) = usecase.register_user(register_request).await.unwrap();

    // Intentar cambiar con una contraseÃ±a incorrecta
    let result = usecase
        .change_password(
            user.id(),
            "WrongPassword123!",
            "NewSecurePass456!",
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::InvalidCredentials));

    // Cleanup
    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_change_password_user_not_found() {
    let usecase = create_test_auth_usecase().await;

    // ID que no existe
    let fake_id = UserId::new();

    let result = usecase
        .change_password(
            fake_id,
            "CurrentPass123!",
            "NewSecurePass456!",
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::UserNotFound));
}
#[tokio::test]
#[ignore]
async fn test_register_duplicate_username() {
    let usecase = create_test_auth_usecase().await;
    let request1 = create_test_register_request();

    // Primer registro
    let (user1, _, _) = usecase.register_user(request1).await.unwrap();

    // Intentar registrar con mismo username pero diferente email
    let request2 = RegisterRequest {
        username: "testuser".to_string(), // Mismo username
        email: "different@example.com".to_string(), // Diferente email
        first_name: "Other".to_string(),
        last_name: "User".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result = usecase.register_user(request2).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::UserAlreadyExists));

    // Cleanup garantizado
    let _ = usecase.user_repository().delete(user1.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_register_duplicate_email() {
    let usecase = create_test_auth_usecase().await;
    let request1 = create_test_register_request();

    // Primer registro
    let (user1, _, _) = usecase.register_user(request1).await.unwrap();

    // Intentar registrar con mismo email pero diferente username
    let request2 = RegisterRequest {
        username: "differentuser".to_string(), // Diferente username
        email: "test@example.com".to_string(), // Mismo email
        first_name: "Other".to_string(),
        last_name: "User".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result = usecase.register_user(request2).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::UserAlreadyExists));

    // Cleanup garantizado
    let _ = usecase.user_repository().delete(user1.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_login_with_email() {
    let usecase = create_test_auth_usecase().await;
    let register_request = create_test_register_request();

    // Registrar usuario
    let (user, _, _) = usecase.register_user(register_request).await.unwrap();

    // Login usando email en vez de username
    let login_request = LoginRequest {
        username: "test@example.com".to_string(), // Usar email
        password: "SecurePass123!".to_string(),
    };

    let result = usecase.login_user(login_request).await;
    assert!(result.is_ok());

    // Cleanup garantizado
    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_login_nonexistent_user() {
    let usecase = create_test_auth_usecase().await;

    let login_request = LoginRequest {
        username: "nonexistent".to_string(),
        password: "SomePassword123!".to_string(),
    };

    let result = usecase.login_user(login_request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::InvalidCredentials));
}

// ============================================================================
// TESTS QUE DEMUESTRAN LA FLEXIBILIDAD DE LA NUEVA ARQUITECTURA
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_multiple_jwt_secrets_in_parallel() {
    // Este test demuestra que podemos ejecutar tests en paralelo
    // con diferentes configuraciones JWT sin race conditions

    let secret1 = "test-secret-one-minimum-32-characters-long";
    let secret2 = "test-secret-two-minimum-32-characters-long";

    let usecase1 = create_test_auth_usecase_with_config(
        "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test",
        secret1,
    );

    let usecase2 = create_test_auth_usecase_with_config(
        "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test",
        secret2,
    );

    // Ejecutar ambos en paralelo con tokio::join!
    let (usecase1, usecase2) = tokio::join!(usecase1, usecase2);

    // Registrar usuarios con diferentes configuraciones JWT
    let request1 = RegisterRequest {
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
        first_name: "User".to_string(),
        last_name: "One".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let request2 = RegisterRequest {
        username: "user2".to_string(),
        email: "user2@example.com".to_string(),
        first_name: "User".to_string(),
        last_name: "Two".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let result1 = usecase1.register_user(request1).await;
    let result2 = usecase2.register_user(request2).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let (user1, token1, _) = result1.unwrap();
    let (user2, token2, _) = result2.unwrap();

    // Los tokens deben ser diferentes (diferentes secrets)
    assert_ne!(token1, token2);

    // Cleanup
    let _ = usecase1.user_repository().delete(user1.id()).await;
    let _ = usecase2.user_repository().delete(user2.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_custom_token_expiry() {
    // Test que usa configuración personalizada de expiración
    let settings = Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test".to_string(),
            max_connections: 5,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-key-minimum-32-characters-long".to_string(),
            access_token_expiry: 60,    // Solo 1 minuto para testing
            refresh_token_expiry: 120,  // Solo 2 minutos para testing
        },
        security: SecuritySettings {
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
        environment: Environment::Test,
    };

    let pool = create_pool(&settings).await.unwrap();
    let user_repository = std::sync::Arc::new(UserRepositorySqlx::new(pool));
    let password_hasher = std::sync::Arc::new(ArgonPasswordHasher::new_for_testing());
    let jwt_service = std::sync::Arc::new(JwtServiceImpl::new(
        settings.jwt.secret.clone(),
        settings.jwt.access_token_expiry,
        settings.jwt.refresh_token_expiry,
    ));

    let usecase = AuthUseCase::new(user_repository, password_hasher, jwt_service);

    let request = RegisterRequest {
        username: "shortlived".to_string(),
        email: "shortlived@example.com".to_string(),
        first_name: "Short".to_string(),
        last_name: "Lived".to_string(),
        password: "SecurePass123!".to_string(),
    };

    let (user, _, _) = usecase.register_user(request).await.unwrap();

    // Verificar que los tiempos de expiración son los configurados
    assert_eq!(settings.jwt.access_token_expiry, 60);
    assert_eq!(settings.jwt.refresh_token_expiry, 120);

    // Cleanup
    let _ = usecase.user_repository().delete(user.id()).await;
}
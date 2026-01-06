// src/application/test/user_management_test.rs
use crate::application::AuthUseCase;
use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx, UserRepository};
use crate::infrastructure::db::create_pool;
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::presentation::RegisterRequest;
use crate::error::AppError;

async fn create_test_auth_usecase() -> AuthUseCase<UserRepositorySqlx, ArgonPasswordHasher, JwtServiceImpl> {
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
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
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

fn create_test_register_request(username: &str, email: &str) -> RegisterRequest {
    RegisterRequest {
        username: username.to_string(),
        email: email.to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        password: "SecurePass123!".to_string(),
    }
}

#[tokio::test]
#[ignore]
async fn test_update_profile() {
    let usecase = create_test_auth_usecase().await;
    let request = create_test_register_request("profiletest", "profile@example.com");

    let (user, _, _) = usecase.register_user(request).await.unwrap();

    let updated = usecase
        .update_profile(
            user.id(),
            Some("Updated".to_string()),
            Some("Name".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(updated.first_name(), "Updated");
    assert_eq!(updated.last_name(), "Name");

    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_account() {
    let usecase = create_test_auth_usecase().await;
    let request = create_test_register_request("deletetest", "delete@example.com");

    let (user, _, _) = usecase.register_user(request).await.unwrap();
    let user_id = user.id();

    let result = usecase.delete_account(user_id).await;
    assert!(result.is_ok());

    // Verificar que ya no existe
    let found = usecase.user_repository().find_by_id(user_id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore]
async fn test_admin_update_username_unauthorized() {
    let usecase = create_test_auth_usecase().await;

    // Crear usuario normal (no admin)
    let request1 = create_test_register_request("normaluser", "normal@example.com");
    let (user1, _, _) = usecase.register_user(request1).await.unwrap();

    // Crear usuario objetivo
    let request2 = create_test_register_request("targetuser", "target@example.com");
    let (user2, _, _) = usecase.register_user(request2).await.unwrap();

    // Intentar actualizar username (debe fallar - no es admin)
    let result = usecase
        .admin_update_username(
            user1.id(),
            user2.id(),
            "newusername".to_string(),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Unauthorized));

    let _ = usecase.user_repository().delete(user1.id()).await;
    let _ = usecase.user_repository().delete(user2.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_admin_update_email_unauthorized() {
    let usecase = create_test_auth_usecase().await;

    let request1 = create_test_register_request("normaluser2", "normal2@example.com");
    let (user1, _, _) = usecase.register_user(request1).await.unwrap();

    let request2 = create_test_register_request("targetuser2", "target2@example.com");
    let (user2, _, _) = usecase.register_user(request2).await.unwrap();

    let result = usecase
        .admin_update_email(
            user1.id(),
            user2.id(),
            "newemail@example.com".to_string(),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Unauthorized));

    let _ = usecase.user_repository().delete(user1.id()).await;
    let _ = usecase.user_repository().delete(user2.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_update_profile_partial_first_name() {
    let usecase = create_test_auth_usecase().await;
    let request = create_test_register_request("partialtest", "partial@example.com");

    let (user, _, _) = usecase.register_user(request).await.unwrap();
    let original_last_name = user.last_name().to_string();

    // Actualizar solo first_name
    let updated = usecase
        .update_profile(
            user.id(),
            Some("NewFirst".to_string()),
            None,
        )
        .await
        .unwrap();

    assert_eq!(updated.first_name(), "NewFirst");
    assert_eq!(updated.last_name(), original_last_name); // No cambió

    let _ = usecase.user_repository().delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_update_profile_partial_last_name() {
    let usecase = create_test_auth_usecase().await;
    let request = create_test_register_request("partialtest2", "partial2@example.com");

    let (user, _, _) = usecase.register_user(request).await.unwrap();
    let original_first_name = user.first_name().to_string();

    // Actualizar solo last_name
    let updated = usecase
        .update_profile(
            user.id(),
            None,
            Some("NewLast".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(updated.first_name(), original_first_name); // No cambió
    assert_eq!(updated.last_name(), "NewLast");

    let _ = usecase.user_repository().delete(user.id()).await;
}
// src/infrastructure/repositories/test/user_repository_sqlx_test.rs
use crate::infrastructure::repositories::{UserRepository, UserRepositorySqlx};
use crate::infrastructure::db::create_pool;
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::domain::{User, Username, Email, PasswordHash};
use sqlx::PgPool;

/// Crea un pool de base de datos para tests sin usar variables de entorno
async fn create_test_pool() -> PgPool {
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

    create_pool(&settings).await.expect("Failed to create test pool")
}

async fn create_test_user() -> User {
    User::new(
        Username::new("testuser".to_string()).unwrap(),
        Email::new("test@example.com".to_string()).unwrap(),
        "Test".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("$argon2id$v=19$m=19456,t=2,p=1$...".to_string()),
    )
}

#[tokio::test]
#[ignore] // Requiere PostgreSQL corriendo
async fn test_create_user() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    let result = repo.create(&user).await;
    assert!(result.is_ok());

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_find_by_id() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let found = repo.find_by_id(user.id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), user.id());

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_find_by_username() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let found = repo.find_by_username(user.username()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username().value(), user.username().value());

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_exists_by_username() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    let exists_before = repo.exists_by_username(user.username()).await.unwrap();
    assert!(!exists_before);

    repo.create(&user).await.unwrap();

    let exists_after = repo.exists_by_username(user.username()).await.unwrap();
    assert!(exists_after);

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_duplicate_username() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user1 = create_test_user().await;
    let user2 = User::new(
        Username::new("testuser".to_string()).unwrap(),
        Email::new("another@example.com".to_string()).unwrap(),
        "Another".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("$argon2id$...".to_string()),
    );

    repo.create(&user1).await.unwrap();
    let result = repo.create(&user2).await;

    assert!(result.is_err());

    // Cleanup
    let _ = repo.delete(user1.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_find_by_username_or_email_with_username() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let found = repo.find_by_username_or_email("testuser").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username().value(), "testuser");

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_find_by_username_or_email_with_email() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let found = repo.find_by_username_or_email("test@example.com").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email().value(), "test@example.com");

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_update_user() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let mut user = create_test_user().await;

    repo.create(&user).await.unwrap();

    // Actualizar el usuario
    user.update_profile("Updated".to_string(), "Name".to_string());
    let result = repo.update(&user).await;

    assert!(result.is_ok());

    // Verificar que se actualizó
    let found = repo.find_by_id(user.id()).await.unwrap().unwrap();
    assert_eq!(found.first_name(), "Updated");
    assert_eq!(found.last_name(), "Name");

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_user() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let result = repo.delete(user.id()).await;
    assert!(result.is_ok());

    // Verificar que se eliminó
    let found = repo.find_by_id(user.id()).await.unwrap();
    assert!(found.is_none());
}
#[tokio::test]
#[ignore]
async fn test_find_by_email() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);
    let user = create_test_user().await;

    repo.create(&user).await.unwrap();

    let found = repo.find_by_email(user.email()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email().value(), user.email().value());

    // Cleanup
    let _ = repo.delete(user.id()).await;
}

#[tokio::test]
#[ignore]
async fn test_find_by_email_not_found() {
    let pool = create_test_pool().await;
    let repo = UserRepositorySqlx::new(pool);

    let non_existent_email = Email::new("nonexistent@example.com".to_string()).unwrap();
    let found = repo.find_by_email(&non_existent_email).await.unwrap();

    assert!(found.is_none());
}

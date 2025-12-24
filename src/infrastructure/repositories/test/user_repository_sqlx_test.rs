// src/infrastructure/repositories/test/user_repository_sqlx_test.rs
use crate::config::Settings;
use crate::domain::{User, Username, Email, PasswordHash};
use crate::infrastructure::db::create_pool;
use crate::infrastructure::repositories::UserRepositorySqlx;
use crate::infrastructure::UserRepository;
use crate::error::AppError;

// Helper para crear settings de prueba
async fn create_test_pool() -> sqlx::PgPool {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test");
        std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
    }

    let settings = Settings::from_env().expect("Failed to load test settings");
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
    assert!(matches!(result.unwrap_err(), AppError::UserAlreadyExists));

    // Cleanup
    let _ = repo.delete(user1.id()).await;
}
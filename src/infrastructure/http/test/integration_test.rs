// src/infrastructure/http/test/integration_test.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::Service;
use serde_json::json;

use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx, UserRepository};
use crate::infrastructure::db::create_pool;
use crate::infrastructure::http::routes::create_router;
use crate::application::AuthUseCase;
use crate::domain::{Username};
use std::sync::Arc;

async fn create_test_app() -> axum::Router {
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

    let pool = create_pool(&settings).await.unwrap();
    let user_repository = Arc::new(UserRepositorySqlx::new(pool.clone()));
    let password_hasher = Arc::new(ArgonPasswordHasher::new_for_testing());
    let jwt_service = Arc::new(JwtServiceImpl::new(
        settings.jwt.secret.clone(),
        settings.jwt.access_token_expiry,
        settings.jwt.refresh_token_expiry,
    ));

    let auth_usecase = Arc::new(AuthUseCase::new(
        user_repository,
        password_hasher,
        jwt_service.clone(),
    ));

    create_router(pool, auth_usecase, jwt_service, &settings)
}

/// Helper para limpiar un usuario específico de la DB
async fn cleanup_user(username: &str) {
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

    let pool = create_pool(&settings).await.unwrap();
    let repo = UserRepositorySqlx::new(pool);

    let username_vo = Username::new(username.to_string()).unwrap();
    if let Ok(Some(user)) = repo.find_by_username(&username_vo).await {
        let _ = repo.delete(user.id()).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_health_endpoint() {
    let mut app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_register_endpoint() {
    // Cleanup antes del test
    cleanup_user("testuser123").await;

    let mut app = create_test_app().await;

    let body = json!({
        "username": "testuser123",
        "email": "test123@example.com",
        "first_name": "Test",
        "last_name": "User",
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Cleanup después del test
    cleanup_user("testuser123").await;
}

#[tokio::test]
#[ignore]
async fn test_login_endpoint() {
    // Cleanup antes del test
    cleanup_user("logintest").await;

    let mut app = create_test_app().await;

    // Primero registrar
    let register_body = json!({
        "username": "logintest",
        "email": "logintest@example.com",
        "first_name": "Login",
        "last_name": "Test",
        "password": "SecurePass123!"
    });

    let register_request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_body).unwrap()))
        .unwrap();

    let _ = app.call(register_request).await.unwrap();

    // Recrear app porque Service::call consume &mut self
    let mut app = create_test_app().await;

    // Luego login
    let login_body = json!({
        "username": "logintest",
        "password": "SecurePass123!"
    });

    let login_request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_body).unwrap()))
        .unwrap();

    let response = app.call(login_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Cleanup después del test
    cleanup_user("logintest").await;
}

#[tokio::test]
#[ignore]
async fn test_register_validation_error() {
    let mut app = create_test_app().await;

    // Username inválido (muy corto)
    let body = json!({
        "username": "ab",
        "email": "test@example.com",
        "first_name": "Test",
        "last_name": "User",
        "password": "SecurePass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_login_invalid_credentials() {
    let mut app = create_test_app().await;

    let body = json!({
        "username": "nonexistent",
        "password": "WrongPass123!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
// src/infrastructure/http/test/routes_test.rs
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx};
use crate::infrastructure::db::create_pool;
use crate::infrastructure::http::routes::create_router;
use crate::application::AuthUseCase;
use std::sync::Arc;

#[test]
fn test_openapi_generation() {
    use crate::infrastructure::http::routes::ApiDoc;
    use utoipa::OpenApi;

    // Verificar que la documentación OpenAPI se genera correctamente
    let openapi = ApiDoc::openapi();

    assert_eq!(openapi.info.title, "Auth API");
    assert_eq!(openapi.info.version, "1.0.0");
    assert!(!openapi.paths.paths.is_empty());
}

#[tokio::test]
#[ignore] // Requiere configuración completa
async fn test_router_creation() {
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

    // Crear router - no debe panic
    let _router = create_router(pool, auth_usecase, jwt_service, &settings);
}
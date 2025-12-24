// src/infrastructure/http/test/routes_test.rs
use crate::config::Settings;
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
    unsafe {
        std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test");
        std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
    }

    let settings = Settings::from_env().unwrap();
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
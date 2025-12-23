// V1
// src/infrastructure/http/routes.rs
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::AuthUseCase;
use crate::config::Settings;
use crate::infrastructure::{
    db::DbPool,
    http::{
        handlers::{health_handler, login_handler, logout_handler, refresh_handler, register_handler},
        middleware::{auth_middleware, create_cors_layer, security_headers_middleware},
    },
    JwtService, PasswordHasher, UserRepository,
};

/// State compartido de la aplicación
#[derive(Clone)]
pub struct AppState<R, P, J>
where
    R: UserRepository + Clone,
    P: PasswordHasher + Clone,
    J: JwtService + Clone,
{
    pub pool: DbPool,
    pub auth_usecase: Arc<AuthUseCase<R, P, J>>,
    pub jwt_service: Arc<J>,
}

impl<R, P, J> AppState<R, P, J>
where
    R: UserRepository + Clone,
    P: PasswordHasher + Clone,
    J: JwtService + Clone,
{
    pub fn new(pool: DbPool, auth_usecase: Arc<AuthUseCase<R, P, J>>, jwt_service: Arc<J>) -> Self {
        Self {
            pool,
            auth_usecase,
            jwt_service,
        }
    }
}

// Implementar AsRef para que health_handler pueda acceder al pool
impl<R, P, J> AsRef<DbPool> for AppState<R, P, J>
where
    R: UserRepository + Clone,
    P: PasswordHasher + Clone,
    J: JwtService + Clone,
{
    fn as_ref(&self) -> &DbPool {
        &self.pool
    }
}

// Implementar AsRef para que los handlers puedan acceder a AuthUseCase
impl<R, P, J> AsRef<Arc<AuthUseCase<R, P, J>>> for AppState<R, P, J>
where
    R: UserRepository + Clone,
    P: PasswordHasher + Clone,
    J: JwtService + Clone,
{
    fn as_ref(&self) -> &Arc<AuthUseCase<R, P, J>> {
        &self.auth_usecase
    }
}

// Implementar AsRef para que los handlers puedan acceder a JwtService
impl<R, P, J> AsRef<Arc<J>> for AppState<R, P, J>
where
    R: UserRepository + Clone,
    P: PasswordHasher + Clone,
    J: JwtService + Clone,
{
    fn as_ref(&self) -> &Arc<J> {
        &self.jwt_service
    }
}

/// Crea el router principal de la aplicación
///
/// Estructura:
/// - /api/v1/health       - Health check (público)
/// - /api/v1/auth/*       - Endpoints de autenticación (públicos)
/// - /api/v1/swagger-ui/  - Documentación Swagger UI
///
/// Middlewares aplicados:
/// - CORS (restrictivo, desde settings)
/// - Security headers
pub fn create_router<R, P, J>(
    pool: DbPool,
    auth_usecase: Arc<AuthUseCase<R, P, J>>,
    jwt_service: Arc<J>,
    settings: &Settings,
) -> Router
where
    R: UserRepository + Clone + 'static,
    P: PasswordHasher + Clone + 'static,
    J: JwtService + Clone + 'static,
{
    let app_state = AppState::new(pool, auth_usecase.clone(), jwt_service);

    // Rutas públicas de autenticación
    let auth_routes: Router<AppState<R, P, J>> = Router::new()
        .route("/register", post(register_handler::<R, P, J, AppState<R, P, J>>))
        .route("/login", post(login_handler::<R, P, J, AppState<R, P, J>>))
        .route("/refresh", post(refresh_handler::<R, P, J, AppState<R, P, J>>))
        .route("/logout", post(logout_handler));

    // Rutas públicas (sin autenticación)
    let public_routes: Router<AppState<R, P, J>> = Router::new()
        .route("/health", get(health_handler::<AppState<R, P, J>>))
        .nest("/auth", auth_routes);

    // Rutas protegidas (requieren autenticación)
    // Por ahora vacías, se pueden añadir más adelante
    let protected_routes: Router<AppState<R, P, J>> = Router::new()
        // Ejemplo de ruta protegida:
        // .route("/profile", get(get_profile_handler))
        .layer(middleware::from_fn_with_state(
            auth_usecase.clone(),
            auth_middleware::<R, P, J>,
        ));

    // API v1
    let api_v1 = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state);

    // Router principal con documentación Swagger
    Router::new()
        .merge(SwaggerUi::new("/api/v1/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_v1)
        // Middlewares globales
        .layer(create_cors_layer(settings))
        .layer(middleware::from_fn(security_headers_middleware))
}

/// Documentación OpenAPI
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::infrastructure::http::handlers::health_handler,
        crate::infrastructure::http::handlers::register_handler,
        crate::infrastructure::http::handlers::login_handler,
        crate::infrastructure::http::handlers::refresh_handler,
        crate::infrastructure::http::handlers::logout_handler,
    ),
    components(
        schemas(
            crate::presentation::RegisterRequest,
            crate::presentation::LoginRequest,
            crate::presentation::RefreshRequest,
            crate::presentation::AuthResponse,
            crate::presentation::UserResponse,
            crate::presentation::SuccessResponse,
            crate::presentation::HealthResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "Endpoints de autenticación de usuarios"),
        (name = "Health", description = "Health check del servicio")
    ),
    info(
        title = "Auth API",
        version = "1.0.0",
        description = "API de autenticación enterprise en Rust con Axum, SQLx y JWT",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
        )
    ),
    servers(
        (url = "http://localhost:9090", description = "Servidor de desarrollo"),
    )
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{ArgonPasswordHasher, JwtServiceImpl, UserRepositorySqlx};

    #[test]
    fn test_openapi_generation() {
        // Verificar que la documentación OpenAPI se genera correctamente
        let openapi = ApiDoc::openapi();

        assert_eq!(openapi.info.title, "Auth API");
        assert_eq!(openapi.info.version, "1.0.0");
        assert!(!openapi.paths.paths.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requiere configuración completa
    async fn test_router_creation() {
        use crate::config::Settings;
        use crate::infrastructure::db::create_pool;

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
}
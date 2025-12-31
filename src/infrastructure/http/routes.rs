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
use crate::config::{Environment, Settings};
use crate::infrastructure::{
    db::DbPool,
    http::{
        handlers::{
            health_handler,
            login_handler,
            logout_handler,
            refresh_handler,
            register_handler,
            get_profile_handler,
            change_password_handler,
        },
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
/// - /api/v1/users/*      - Endpoints de usuario (protegidos)
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

    // Rutas protegidas de usuarios (requieren autenticación)
    let user_routes: Router<AppState<R, P, J>> = Router::new()
        .route("/profile", get(get_profile_handler::<R, P, J, AppState<R, P, J>>))
        .route("/change-password", post(change_password_handler::<R, P, J, AppState<R, P, J>>));

    let protected_routes: Router<AppState<R, P, J>> = Router::new()
        .nest("/users", user_routes)
        .layer(middleware::from_fn_with_state(
            auth_usecase.clone(),
            auth_middleware::<R, P, J>,
        ));

    match settings.environment {
        Environment::Development => {
            tracing::info!("🌍  Entorno: Desarrollo");
            tracing::info!("    • CSP: Permisivo para Swagger UI");
            tracing::info!("    • CORS: Permite localhost");
        }
        Environment::Test => {
            tracing::info!("🧪  Entorno: Testing");
            tracing::info!("    • CSP: Restrictivo");
            tracing::info!("    • CORS: Desde configuración");
        }
        Environment::Production => {
            tracing::info!("🔐  Entorno: Producción");
            tracing::info!("    • CSP: Máxima seguridad");
            tracing::info!("    • CORS: Muy restrictivo");
        }
    }

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
        crate::infrastructure::http::handlers::get_profile_handler,
        crate::infrastructure::http::handlers::change_password_handler,
    ),
    components(
        schemas(
            crate::presentation::RegisterRequest,
            crate::presentation::LoginRequest,
            crate::presentation::RefreshRequest,
            crate::presentation::ChangePasswordRequest,
            crate::presentation::AuthResponse,
            crate::presentation::UserResponse,
            crate::presentation::SuccessResponse,
            crate::presentation::HealthResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "Endpoints de autenticación de usuarios"),
        (name = "Users", description = "Gestión de perfil de usuario"),
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
pub(crate) struct ApiDoc;
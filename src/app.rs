// src/app.rs
use std::sync::Arc;

use crate::application::AuthUseCase;
use crate::config::Settings;
use crate::error::AppResult;
use crate::infrastructure::{
    db::{create_pool, DbPool},
    http::routes::create_router,
    security::{ArgonPasswordHasher, JwtServiceImpl},
    repositories::UserRepositorySqlx,
};
use axum::Router;

/// Inicializa y configura la aplicación
///
/// Este es el punto de entrada principal que:
/// 1. Carga la configuración
/// 2. Crea el pool de base de datos
/// 3. Inicializa los servicios (repositorio, hasher, JWT)
/// 4. Crea el caso de uso de autenticación
/// 5. Construye el router con todos los endpoints
pub async fn build_app(settings: Settings) -> AppResult<Router> {
    tracing::info!("🚀 Iniciando aplicación...");

    // 1. Crear pool de base de datos
    tracing::info!("📦 Creando pool de conexiones a la base de datos...");
    let pool = create_pool(&settings).await?;
    tracing::info!("✅ Pool de conexiones creado exitosamente");

    // 2. Crear servicios de infraestructura
    tracing::info!("🔧 Inicializando servicios...");

    let user_repository = Arc::new(UserRepositorySqlx::new(pool.clone()));
    let password_hasher = Arc::new(ArgonPasswordHasher::new());
    let jwt_service = Arc::new(JwtServiceImpl::new(
        settings.jwt.secret.clone(),
        settings.jwt.access_token_expiry,
        settings.jwt.refresh_token_expiry,
    ));

    tracing::info!("✅ Servicios inicializados");

    // 3. Crear caso de uso de autenticación
    let auth_usecase = Arc::new(AuthUseCase::new(
        user_repository,
        password_hasher,
        jwt_service.clone(),
    ));

    tracing::info!("✅ Casos de uso configurados");

    // 4. Construir el router
    tracing::info!("🌐 Construyendo router...");
    let router = create_router(pool, auth_usecase, jwt_service, &settings);
    tracing::info!("✅ Router construido exitosamente");

    tracing::info!("🎉 Aplicación inicializada correctamente");

    Ok(router)
}

/// Información de la aplicación para logging
pub fn log_app_info(settings: &Settings) {
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📋 Auth API - Rust Edition");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🌐 Servidor: {}", settings.socket_addr());
    tracing::info!("🗄️  Base de datos: {}", mask_db_url(&settings.database.url));
    tracing::info!("🔐 JWT configurado: {} segundos", settings.jwt.access_token_expiry);
    tracing::info!("🌍 Entorno: {:?}", settings.environment);
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📚 Documentación API:");
    tracing::info!("   Swagger UI: http://{}/api/v1/swagger-ui/", settings.socket_addr());
    tracing::info!("   OpenAPI JSON: http://{}/api/v1/openapi.json", settings.socket_addr());
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Enmascara la contraseña en la URL de la base de datos para logging seguro
pub(crate) fn mask_db_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}
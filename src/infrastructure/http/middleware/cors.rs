// src/infrastructure/http/middleware/cors.rs
use tower_http::cors::CorsLayer;
use axum::http::{HeaderValue, Method};
use crate::config::{Settings, Environment};

/// Crea CORS según el entorno
pub fn create_cors_layer(settings: &Settings) -> CorsLayer {
    match settings.environment {
        Environment::Development => create_development_cors(settings),
        Environment::Test => create_test_cors(settings),
        Environment::Production => create_production_cors(settings),
    }
}

/// CORS para desarrollo (permisivo, localhost sin HTTPS)
fn create_development_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("🔓 CORS: Desarrollo - Permisivo (HTTP localhost)");

    // La configuración viene de settings (de .env)
    let allowed_origins: Vec<HeaderValue> = settings
        .security
        .allowed_origins
        .iter()
        .filter_map(|origin| {
            origin.parse::<HeaderValue>().ok()
        })
        .collect();

    if allowed_origins.is_empty() {
        tracing::warn!("⚠️  No hay orígenes configurados en ALLOWED_ORIGINS");
    }

    tracing::info!("   Orígenes permitidos: {:?}",
        settings.security.allowed_origins.iter().collect::<Vec<_>>()
    );

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ])
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(false)
        .max_age(std::time::Duration::from_secs(3600))
}

/// CORS para testing
fn create_test_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("🧪 CORS: Testing - Configuración balanceada");

    let allowed_origins: Vec<HeaderValue> = settings
        .security
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(false)
        .max_age(std::time::Duration::from_secs(1800))
}

/// CORS para producción (restrictivo, SOLO HTTPS)
fn create_production_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("🔒 CORS: Producción - Máxima seguridad (HTTPS required)");

    // ✅ Validar que SOLO haya orígenes HTTPS en producción
    let allowed_origins: Vec<HeaderValue> = settings
        .security
        .allowed_origins
        .iter()
        .filter(|origin| {
            if !origin.starts_with("https://") {
                tracing::error!("❌ Origen NO HTTPS rechazado en producción: {}", origin);
                false
            } else {
                true
            }
        })
        .filter_map(|origin| origin.parse().ok())
        .collect();

    if allowed_origins.is_empty() {
        tracing::error!("❌ PRODUCCIÓN: No hay orígenes HTTPS configurados!");
        panic!("CORS en producción requiere al menos un origen HTTPS en ALLOWED_ORIGINS");
    }

    tracing::info!("   Orígenes HTTPS permitidos: {:?}",
        settings.security.allowed_origins
            .iter()
            .filter(|o| o.starts_with("https://"))
            .collect::<Vec<_>>()
    );

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(false)
        .max_age(std::time::Duration::from_secs(3600))
}
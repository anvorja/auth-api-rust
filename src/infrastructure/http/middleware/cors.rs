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

/// CORS para desarrollo (Swagger UI friendly)
fn create_development_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("🔓 CORS: Desarrollo - Permisivo para Swagger UI");

    let mut allowed_origins: Vec<HeaderValue> = settings
        .security
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    // Agregar localhost automáticamente en desarrollo
    let dev_origins = [
        "http://localhost:9090",
        "http://127.0.0.1:9090",
    ];

    for origin in dev_origins {
        if let Ok(header) = origin.parse::<HeaderValue>() {
            if !allowed_origins.contains(&header) {
                allowed_origins.push(header);
            }
        }
    }

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
            axum::http::header::SET_COOKIE,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// CORS para testing (restrictivo pero funcional)
fn create_test_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("  CORS: Testing - Configuración balanceada");

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
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(1800))
}

/// CORS para producción (máxima seguridad)
fn create_production_cors(settings: &Settings) -> CorsLayer {
    tracing::info!("🔐 CORS: Producción - Máxima seguridad");

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
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}
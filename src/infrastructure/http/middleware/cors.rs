// V1
// src/infrastructure/http/middleware/cors.rs
use tower_http::cors::{Any, CorsLayer};
use axum::http::{HeaderValue, Method};

use crate::config::Settings;

/// Crea el middleware CORS configurado desde settings
///
/// Configuración:
/// - Permite origins específicos desde .env
/// - Permite métodos comunes (GET, POST, PUT, DELETE, OPTIONS)
/// - Permite headers necesarios para JWT y JSON
/// - Permite credentials (cookies HttpOnly)
pub fn create_cors_layer(settings: &Settings) -> CorsLayer {
    // Parsear los origins permitidos
    let allowed_origins: Vec<HeaderValue> = settings
        .security
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        // Origins permitidos (desde configuración)
        .allow_origin(allowed_origins)
        // Métodos permitidos
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // Headers permitidos
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        // Exponer headers al cliente
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
        ])
        // Permitir credentials (necesario para cookies HttpOnly)
        .allow_credentials(true)
        // Tiempo de cache para preflight requests (1 hora)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Crea un CORS permisivo para desarrollo
///
/// ⚠️ NUNCA usar en producción
#[cfg(debug_assertions)]
pub fn create_permissive_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer() {
        unsafe {
            std::env::set_var("ALLOWED_ORIGINS", "http://localhost:3000");
            std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
            std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        }

        let settings = Settings::from_env().unwrap();
        let _cors = create_cors_layer(&settings);

        // Si no panic, el layer se creó correctamente
    }
}
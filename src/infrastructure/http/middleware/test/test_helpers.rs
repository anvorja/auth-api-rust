// src/infrastructure/http/middleware/test/test_helpers.rs
//! Test helpers para security headers
//!
//! Este módulo contiene utilidades para testing que NO deben
//! estar en el código de producción.

use axum::{
    body::Body,
    extract::Request,
    http::{header, Response},
    middleware::Next,
};

/// Configuración de test para security headers
pub struct SecurityHeadersTestConfig {
    pub is_production: bool,
}

impl Default for SecurityHeadersTestConfig {
    fn default() -> Self {
        Self {
            is_production: false,
        }
    }
}

impl SecurityHeadersTestConfig {
    pub fn development() -> Self {
        Self {
            is_production: false,
        }
    }

    pub fn production() -> Self {
        Self {
            is_production: true,
        }
    }
}

/// Aplica security headers con configuración inyectada (solo para tests)
///
/// Esta función replica la lógica de security_headers_middleware
/// pero acepta configuración explícita en lugar de leer de env vars.
pub async fn apply_test_security_headers(
    request: Request,
    next: Next,
    config: SecurityHeadersTestConfig,
) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Headers básicos (siempre)
    headers.insert(
        header::X_FRAME_OPTIONS,
        "DENY".parse().unwrap(),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // CSP según configuración
    if config.is_production {
        // 🔐 Producción: CSP restrictivo
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; \
             script-src 'self'; \
             style-src 'self'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
                .parse()
                .unwrap(),
        );

        // Headers adicionales de producción
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
        );
        headers.insert(
            axum::http::HeaderName::from_static("x-xss-protection"),
            "1; mode=block".parse().unwrap(),
        );
        headers.insert(
            axum::http::HeaderName::from_static("permissions-policy"),
            "geolocation=(), microphone=(), camera=(), payment=()".parse().unwrap(),
        );
    } else {
        // 🔓 Desarrollo: CSP permisivo
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self' http://localhost:* http://127.0.0.1:*; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
                .parse()
                .unwrap(),
        );
    }

    response
}
// src/infrastructure/http/middleware/security_headers.rs
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

/// Middleware de security headers que se adapta al entorno
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Detectar el entorno desde variable de entorno
    let is_production = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase()
        == "production";

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

    // CSP según entorno
    if is_production {
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

        tracing::debug!("🔐 Security Headers: Producción (restrictivo)");
    } else {

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

        tracing::debug!("🔓 Security Headers: Desarrollo (permisivo para Swagger)");
    }

    response
}
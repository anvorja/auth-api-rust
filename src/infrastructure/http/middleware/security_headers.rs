// src/infrastructure/http/middleware/security_headers.rs
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

/// Middleware que añade headers de seguridad HTTP
///
/// Headers configurados:
/// - Content-Security-Policy (CSP)
/// - X-Frame-Options
/// - X-Content-Type-Options
/// - X-XSS-Protection
/// - Referrer-Policy
/// - Permissions-Policy
/// - Strict-Transport-Security (HSTS)
///
/// Estos headers protegen contra:
/// - XSS (Cross-Site Scripting)
/// - Clickjacking
/// - MIME sniffing
/// - Content injection
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content-Security-Policy (CSP)
    // Política restrictiva que previene XSS y otros ataques de inyección
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
    
    // Producción: CSP más restrictivo
    #[cfg(not(debug_assertions))]
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; \
         script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: https:; \
         font-src 'self' data:; \
         connect-src 'self'; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self'"
            .parse()
            .unwrap(),
    );

    // X-Frame-Options
    headers.insert(
        header::X_FRAME_OPTIONS,
        "DENY".parse().unwrap(),
    );

    // X-Content-Type-Options
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // X-XSS-Protection
    headers.insert(
        axum::http::HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );

    // Referrer-Policy
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Permissions-Policy
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        "geolocation=(), microphone=(), camera=(), payment=()".parse().unwrap(),
    );

    // HSTS solo en producción
    #[cfg(not(debug_assertions))]
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
    );

    response
}

/// Headers de seguridad permisivos para desarrollo
///
/// Útil para Swagger UI y desarrollo local
/// ⚠️ NUNCA usar en producción
#[cfg(debug_assertions)]
pub async fn permissive_security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // CSP más permisivo para Swagger UI
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: https:; \
         frame-ancestors 'self'"
            .parse()
            .unwrap(),
    );

    headers.insert(
        header::X_FRAME_OPTIONS,
        "SAMEORIGIN".parse().unwrap(),
    );

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    response
}
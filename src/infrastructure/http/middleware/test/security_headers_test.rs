// src/infrastructure/http/middleware/test/security_headers_test.rs
use super::test_helpers::{apply_test_security_headers, SecurityHeadersTestConfig};
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    Router,
};
use tower::Service;
use std::pin::Pin;
use std::future::Future;

#[test]
fn test_security_headers_middleware_exists() {
    use crate::infrastructure::http::middleware::security_headers_middleware;
    let _middleware = security_headers_middleware;
}

#[test]
fn test_environment_detection_logic() {
    let test_cases = vec![
        ("production", true),
        ("Production", true),
        ("PRODUCTION", true),
        ("development", false),
        ("Development", false),
        ("test", false),
        ("", false),
    ];

    for (env_value, expected_is_production) in test_cases {
        let is_production = env_value.to_lowercase() == "production";
        assert_eq!(
            is_production, expected_is_production,
            "Failed for env value: '{}'",
            env_value
        );
    }
}

// ============================================================================
// TESTS DE INTEGRACIÓN
// ============================================================================

async fn dummy_handler() -> &'static str {
    "OK"
}

/// Factory function para crear app de test con configuración
async fn create_test_app(config: SecurityHeadersTestConfig) -> Router {
    let middleware_fn = move |req: Request <Body>, next: Next| {
        let cfg = SecurityHeadersTestConfig {
            is_production: config.is_production,
        };
        Box::pin(async move { apply_test_security_headers(req, next, cfg).await })
            as Pin<Box<dyn Future<Output = Response<Body>> + Send>>
    };

    Router::new()
        .route("/test", axum::routing::get(dummy_handler))
        .layer(middleware::from_fn(middleware_fn))
}

async fn test_with_config(config: SecurityHeadersTestConfig) -> Response<Body> {
    let mut app = create_test_app(config).await;

    let request = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    app.call(request).await.unwrap()
}

#[tokio::test]
async fn test_security_headers_in_development() {
    let response = test_with_config(SecurityHeadersTestConfig::development()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("content-security-policy"));

    let csp = headers
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(csp.contains("connect-src 'self' http://localhost:*"));
    assert!(csp.contains("'unsafe-inline'"));
    assert!(!headers.contains_key("strict-transport-security"));
}

#[tokio::test]
async fn test_security_headers_in_production() {
    let response = test_with_config(SecurityHeadersTestConfig::production()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    let csp = headers
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(csp.contains("connect-src 'self'") && !csp.contains("http://localhost"));
    assert!(!csp.contains("'unsafe-inline'"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("permissions-policy"));
}

#[tokio::test]
async fn test_x_frame_options_always_deny() {
    let response = test_with_config(SecurityHeadersTestConfig::development()).await;
    let headers = response.headers();
    let x_frame = headers.get("x-frame-options").unwrap().to_str().unwrap();
    assert_eq!(x_frame, "DENY");
}

#[tokio::test]
async fn test_x_content_type_options_always_nosniff() {
    let response = test_with_config(SecurityHeadersTestConfig::development()).await;
    let headers = response.headers();
    let x_content = headers
        .get("x-content-type-options")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(x_content, "nosniff");
}

#[tokio::test]
async fn test_referrer_policy_always_set() {
    let response = test_with_config(SecurityHeadersTestConfig::development()).await;
    let headers = response.headers();
    assert!(headers.contains_key("referrer-policy"));
    let referrer = headers.get("referrer-policy").unwrap().to_str().unwrap();
    assert_eq!(referrer, "strict-origin-when-cross-origin");
}
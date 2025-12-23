use axum::http::{Method, HeaderValue};
use tower_http::cors::CorsLayer;

pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            // Add other origins here
        ])
        .allow_credentials(true)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
}

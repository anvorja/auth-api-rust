// V2
// src/infrastructure/http/routes.rs
use axum::{
    routing::{get, post},
    Router,
};
use crate::app::AppState;
use crate::infrastructure::http::handlers::{auth, health};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::presentation::dto;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::register,
        auth::login,
        auth::refresh,
        auth::logout,
    ),
    components(
        schemas(dto::RegisterUserRequest, dto::LoginUserRequest, dto::AuthResponse, dto::UserResponse)
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Health", description = "Health check endpoint")
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout));

    Router::new()
        .merge(SwaggerUi::new("/api/v1/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_routes)
        .with_state(state)
}

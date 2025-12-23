use axum::{Json, response::IntoResponse};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "API is healthy", body = serde_json::Value)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok", "message": "Service is running" }))
}

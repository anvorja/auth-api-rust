// src/infrastructure/http/handlers/health.rs
use axum::{extract::State, Json};

use crate::infrastructure::db::health_check;
use crate::presentation::HealthResponse;

/// Handler para health check
///
/// GET /api/v1/health
///
/// Verifica:
/// - Que el servicio esté corriendo
/// - Que la base de datos esté disponible
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Servicio saludable", body = HealthResponse),
        (status = 503, description = "Servicio no disponible", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_handler<S>(State(state): State<S>) -> Json<HealthResponse>
where
    S: AsRef<crate::infrastructure::DbPool> + Clone + Send + Sync + 'static,
{
    // Verificar conexión a la base de datos
    let pool = state.as_ref();
    let db_healthy = health_check(pool).await;

    let status = if db_healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    Json(HealthResponse::new(
        status.to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    ))
}
// src/infrastructure/db.rs
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::config::Settings;
use crate::error::AppError;

/// Alias para el pool de base de datos
pub type DbPool = PgPool;

/// Crea un pool de conexiones a PostgreSQL
///
/// Configuración
/// - Usa las settings de la aplicación
/// - Configura timeouts y límites de conexiones
/// - Verifica la conexión antes de retornar
pub async fn create_pool(settings: &Settings) -> Result<DbPool, AppError> {
    tracing::info!(
        "Creando pool de conexiones a PostgreSQL (max: {}, min: {})",
        settings.database.max_connections,
        settings.database.min_connections
    );

    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .acquire_timeout(Duration::from_secs(settings.database.acquire_timeout))
        // Tiempo máximo que una conexión puede estar idle
        .idle_timeout(Duration::from_secs(600)) // 10 minutos
        // Tiempo máximo de vida de una conexión
        .max_lifetime(Duration::from_secs(1800)) // 30 minutos
        // Test de conexión antes de usar (opcional, pero bueno para producción)
        .test_before_acquire(true)
        .connect(&settings.database.url)
        .await
        .map_err(|e| {
            tracing::error!("Error al conectar con PostgreSQL: {}", e);
            // Clasificar el error apropiadamente
            match e {
                sqlx::Error::PoolTimedOut => {
                    tracing::error!("Timeout del pool - base de datos no responde");
                    AppError::unavailable()  // ✓ 503 Service Unavailable
                }
                sqlx::Error::PoolClosed => {
                    tracing::error!("Pool cerrado - posible reinicio de base de datos");
                    AppError::unavailable()  // ✓ 503 Service Unavailable
                }
                sqlx::Error::Configuration(_) => {
                    // Error de configuración de la base de datos
                    tracing::error!("Configuración de DB inválida");
                    AppError::config(format!("URL de base de datos inválida: {}", e))  // ✓ ConfigError
                }
                sqlx::Error::Database(db_err) => {
                    if db_err.is_unique_violation() {
                        // Esto no debería pasar al conectar, pero por si acaso
                        AppError::config("Configuración de DB duplicada o conflictiva".to_string())
                    } else {
                        AppError::DatabaseConnectionError
                    }
                }
                // Otros errores de conexión (network, auth, etc.)
                _ => AppError::DatabaseConnectionError,  // 500 Internal Server Error
            }
        })?;

    tracing::info!("✓ Pool de conexiones creado exitosamente");

    // Verificar conexión ejecutando una query simple
    verify_connection(&pool).await?;

    Ok(pool)
}

/// Verifica que la conexión a la base de datos funcione
pub async fn verify_connection(pool: &DbPool) -> Result<(), AppError> {
    tracing::debug!("Verificando conexión a la base de datos...");

    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Error al verificar conexión: {}", e);
            AppError::DatabaseConnectionError
        })?;

    tracing::debug!("✓ Conexión verificada");
    Ok(())
}

/// Health check de la base de datos
/// Retorna true si la DB está disponible, false en caso contrario
pub async fn health_check(pool: &DbPool) -> bool {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            tracing::debug!("Health check DB: OK");
            true
        }
        Err(e) => {
            tracing::warn!("Health check DB: FAIL - {}", e);
            false
        }
    }
}

/// Cierra el pool de conexiones de manera ordenada
pub async fn close_pool(pool: DbPool) {
    tracing::info!("Cerrando pool de conexiones...");
    pool.close().await;
    tracing::info!("✓ Pool cerrado");
}

// Se usan solo en testing: close_pool, pool_stats, PoolStats
/// Obtiene estadísticas del pool de conexiones
pub fn pool_stats(pool: &DbPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle(),
    }
}

/// Estadísticas del pool de conexiones
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Número total de conexiones en el pool
    pub size: u32,
    /// Número de conexiones idle (disponibles)
    pub idle: usize,
}

impl PoolStats {
    /// Número de conexiones activas (en uso)
    pub fn active(&self) -> u32 {
        self.size.saturating_sub(self.idle as u32)
    }
}
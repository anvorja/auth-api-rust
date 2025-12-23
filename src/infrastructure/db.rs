// V1
// src/infrastructure/db.rs
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::config::Settings;
use crate::error::AppError;

/// Alias para el pool de base de datos
pub type DbPool = PgPool;

/// Crea un pool de conexiones a PostgreSQL
///
/// # Configuración
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
            AppError::DatabaseConnectionError
        })?;

    tracing::info!("✅ Pool de conexiones creado exitosamente");

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

    tracing::debug!("✅ Conexión verificada");
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
    tracing::info!("✅ Pool cerrado");
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    // Helper para crear settings de prueba
    fn create_test_settings() -> Settings {
        // Usar variables de entorno o valores por defecto para testing
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db");
            std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
        }

        Settings::from_env().expect("Failed to load test settings")
    }

    #[tokio::test]
    #[ignore] // Ignorar por defecto (requiere PostgreSQL corriendo)
    async fn test_create_pool() {
        let settings = create_test_settings();
        let pool = create_pool(&settings).await;

        assert!(pool.is_ok());

        if let Ok(pool) = pool {
            close_pool(pool).await;
        }
    }

    #[tokio::test]
    #[ignore] // Ignorar por defecto
    async fn test_verify_connection() {
        let settings = create_test_settings();
        let pool = create_pool(&settings).await.unwrap();

        let result = verify_connection(&pool).await;
        assert!(result.is_ok());

        close_pool(pool).await;
    }

    #[tokio::test]
    #[ignore] // Ignorar por defecto
    async fn test_health_check() {
        let settings = create_test_settings();
        let pool = create_pool(&settings).await.unwrap();

        let is_healthy = health_check(&pool).await;
        assert!(is_healthy);

        close_pool(pool).await;
    }

    #[tokio::test]
    #[ignore] // Ignorar por defecto
    async fn test_pool_stats() {
        let settings = create_test_settings();
        let pool = create_pool(&settings).await.unwrap();

        let stats = pool_stats(&pool);

        // Debe haber al menos min_connections
        assert!(stats.size >= settings.database.min_connections);
        // Todas deben estar idle inicialmente
        assert!(stats.idle > 0);
        // Active debe ser 0 o bajo
        assert!(stats.active() < stats.size);

        close_pool(pool).await;
    }

    #[test]
    fn test_pool_stats_active_calculation() {
        let stats = PoolStats {
            size: 10,
            idle: 7,
        };

        assert_eq!(stats.active(), 3);
    }

    #[test]
    fn test_pool_stats_no_underflow() {
        let stats = PoolStats {
            size: 5,
            idle: 10, // Más idle que size (no debería pasar, pero por seguridad)
        };

        // No debe hacer underflow
        assert_eq!(stats.active(), 0);
    }
}
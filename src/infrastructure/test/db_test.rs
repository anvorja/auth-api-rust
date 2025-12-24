// src/infrastructure/test/db_test.rs
use crate::config::Settings;
use crate::infrastructure::db::{create_pool, verify_connection, health_check, close_pool, pool_stats, PoolStats};

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
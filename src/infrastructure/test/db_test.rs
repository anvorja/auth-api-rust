// src/infrastructure/test/db_test.rs
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::infrastructure::db::{create_pool, verify_connection, health_check, close_pool, pool_stats, PoolStats};

/// Crea Settings para tests sin usar variables de entorno
fn create_test_settings() -> Settings {
    Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test".to_string(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-key-minimum-32-characters-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
        },
        security: SecuritySettings {
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
        environment: Environment::Test,
    }
}

#[tokio::test]
// #[ignore] // Ignorar por defecto (requiere PostgreSQL corriendo)
async fn test_create_pool() {
    let settings = create_test_settings();
    let pool = create_pool(&settings).await;

    assert!(pool.is_ok());

    if let Ok(pool) = pool {
        close_pool(pool).await;
    }
}

#[tokio::test]
// #[ignore] // Ignorar por defecto
async fn test_verify_connection() {
    let settings = create_test_settings();
    let pool = create_pool(&settings).await.unwrap();

    let result = verify_connection(&pool).await;
    assert!(result.is_ok());

    close_pool(pool).await;
}

#[tokio::test]
//#[ignore] // Ignorar por defecto
async fn test_health_check() {
    let settings = create_test_settings();
    let pool = create_pool(&settings).await.unwrap();

    let is_healthy = health_check(&pool).await;
    assert!(is_healthy);

    close_pool(pool).await;
}

#[tokio::test]
// #[ignore] // Ignorar por defecto
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
        idle: 10,
    };

    // No debe hacer underflow
    assert_eq!(stats.active(), 0);
}
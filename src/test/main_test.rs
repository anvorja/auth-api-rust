// src/test/main_test.rs
//
// Tests para main.rs
//
// NOTA: main.rs es principalmente código de bootstrap (inicialización, logging, TCP)
// que es difícil de testear directamente. Estos tests verifican los componentes
// individuales que main.rs usa, pero no el binario completo.

use crate::config::Settings;

// ============================================================================
// TESTS DE CONFIGURACIÓN (lo que main.rs hace primero)
// ============================================================================

#[test]
fn test_settings_loading_behavior() {
    use temp_env::with_vars;

    // Simular carga exitosa de configuración
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_ok(), "Settings should load successfully");
        },
    );
}

#[test]
fn test_settings_loading_failure_missing_db() {
    use temp_env::with_vars;

    // Simular fallo de configuración (lo que causaría exit(1) en main)
    with_vars(
        vec![
            ("ENVIRONMENT", Some("production")),
            ("DATABASE_URL", None::<&str>),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err(), "Settings should fail without DATABASE_URL");
        },
    );
}

#[test]
fn test_settings_loading_failure_missing_jwt() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("ENVIRONMENT", Some("production")),
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", None::<&str>),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err(), "Settings should fail without JWT_SECRET");
        },
    );
}

// ============================================================================
// TESTS DE SOCKET ADDRESS (lo que main.rs usa para bind)
// ============================================================================

#[test]
fn test_socket_addr_formatting() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("SERVER_HOST", Some("127.0.0.1")),
            ("SERVER_PORT", Some("9090")),
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            let addr = settings.socket_addr();

            assert_eq!(addr, "127.0.0.1:9090");
        },
    );
}

#[test]
fn test_socket_addr_different_port() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("SERVER_HOST", Some("0.0.0.0")),
            ("SERVER_PORT", Some("8080")),
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            let addr = settings.socket_addr();

            assert_eq!(addr, "0.0.0.0:8080");
        },
    );
}

// ============================================================================
// TESTS DE INTEGRACIÓN (flujo completo hasta antes del servidor)
// ============================================================================

#[tokio::test]
#[ignore] // Requiere PostgreSQL
async fn test_app_initialization_flow() {
    // Para tests async, usamos env vars directamente sin temp_env
    unsafe { std::env::set_var("SERVER_HOST", "127.0.0.1"); }
    unsafe { std::env::set_var("SERVER_PORT", "8080"); }
    unsafe { std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test"); }
    unsafe { std::env::set_var("DATABASE_MAX_CONNECTIONS", "5"); }
    unsafe { std::env::set_var("DATABASE_MIN_CONNECTIONS", "2"); }
    unsafe { std::env::set_var("DATABASE_ACQUIRE_TIMEOUT", "30"); }
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long"); }
    unsafe { std::env::set_var("JWT_ACCESS_TOKEN_EXPIRY", "900"); }
    unsafe { std::env::set_var("JWT_REFRESH_TOKEN_EXPIRY", "604800"); }
    unsafe { std::env::set_var("ALLOWED_ORIGINS", "http://localhost:3000"); }
    unsafe { std::env::set_var("ENVIRONMENT", "test"); }

    // 1. Cargar configuración (lo que main.rs hace primero)
    let settings = Settings::from_env();
    assert!(settings.is_ok(), "Settings should load");

    let settings = settings.unwrap();

    // 2. Construir app (lo que main.rs hace después)
    let app_result = crate::app::build_app(settings).await;
    assert!(app_result.is_ok(), "App should build successfully");
}

#[tokio::test]
#[ignore] // Requiere intentar conectar a DB inexistente
async fn test_app_initialization_with_invalid_db() {
    unsafe { std::env::set_var("SERVER_HOST", "127.0.0.1"); }
    unsafe { std::env::set_var("SERVER_PORT", "8080"); }
    unsafe { std::env::set_var("DATABASE_URL", "postgresql://invalid:invalid@nonexistent:9999/fake"); }
    unsafe { std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long"); }
    unsafe { std::env::set_var("ENVIRONMENT", "test"); }

    let settings = Settings::from_env().unwrap();

    // Esto debería fallar porque la DB no existe
    let app_result = crate::app::build_app(settings).await;
    assert!(app_result.is_err(), "App should fail with invalid DB");
}

// ============================================================================
// TESTS DE LOG_APP_INFO
// ============================================================================

#[test]
fn test_log_app_info_does_not_panic() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let settings = Settings::from_env().unwrap();

            // log_app_info solo hace logging, no debería panic
            crate::app::log_app_info(&settings);
        },
    );
}

// ============================================================================
// TESTS DE COMPORTAMIENTO DE ERROR HANDLING
// ============================================================================

#[test]
fn test_error_handling_config_error() {
    use temp_env::with_vars;

    // Simular el escenario donde main.rs haría exit(1)
    with_vars(
        vec![
            ("ENVIRONMENT", Some("production")), // ← ¡AGREGA ESTO!
            ("DATABASE_URL", None::<&str>),
            ("JWT_SECRET", None::<&str>),
        ],
        || {
            let result = Settings::from_env();

            assert!(result.is_err());

            // Verificar que el error es apropiado
            let err = result.unwrap_err();
            assert!(matches!(
                err,
                crate::config::settings::ConfigError::MissingEnvVar(_)
            ));
        },
    );
}

// ============================================================================
// TESTS DE ENTORNO DE EJECUCIÓN
// ============================================================================

#[test]
fn test_development_environment_detection() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("development")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            assert!(settings.is_development());
            assert!(!settings.is_production());
        },
    );
}

#[test]
fn test_production_environment_detection() {
    use temp_env::with_vars;

    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("production")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            assert!(settings.is_production());
            assert!(!settings.is_development());
        },
    );
}

// ============================================================================
// TEST ADICIONAL: Verificar que los componentes de main.rs funcionan juntos
// ============================================================================

#[test]
fn test_main_components_integration() {
    use temp_env::with_vars;

    // Este test verifica que los pasos que main.rs ejecuta funcionan
    with_vars(
        vec![
            ("SERVER_HOST", Some("127.0.0.1")),
            ("SERVER_PORT", Some("9090")),
            ("DATABASE_URL", Some("postgresql://postgres:pass@localhost/test")),
            ("JWT_SECRET", Some("test-secret-key-minimum-32-characters-long")),
            ("ENVIRONMENT", Some("development")),
            ("RUST_LOG", Some("info,auth_api_rust=debug")),
        ],
        || {
            // Paso 1: Cargar settings (main.rs línea ~15)
            let settings_result = Settings::from_env();
            assert!(settings_result.is_ok());

            let settings = settings_result.unwrap();

            // Paso 2: Verificar socket_addr (main.rs línea ~35)
            let addr = settings.socket_addr();
            assert_eq!(addr, "127.0.0.1:9090");

            // Paso 3: log_app_info (main.rs línea ~25)
            crate::app::log_app_info(&settings);

            // Paso 4: build_app se testea en test_app_initialization_flow (async)
            // No podemos llamarlo aquí porque es async
        },
    );
}

#[test]
fn test_error_scenarios_that_would_exit() {
    // Escenarios que en main.rs causarían std::process::exit(1)

    // Escenario 1: Fallo al cargar settings
    {
        use temp_env::with_vars;
        with_vars(
            vec![
                ("ENVIRONMENT", Some("production")),
                ("DATABASE_URL", None::<&str>),
            ],
            || {
                let result = Settings::from_env();
                assert!(result.is_err(), "Should fail - would exit(1) in main");
            },
        );
    }

    // Escenario 2: JWT secret inseguro
    {
        use temp_env::with_vars;
        with_vars(
            vec![
                ("ENVIRONMENT", Some("production")),
                ("DATABASE_URL", Some("postgresql://localhost/test")),
                ("JWT_SECRET", Some("short")),
            ],
            || {
                let result = Settings::from_env();
                assert!(result.is_err(), "Should fail - would exit(1) in main");
            },
        );
    }
}

// ============================================================================
// DOCUMENTACIÓN DE LIMITACIONES
// ============================================================================

// NOTA: Los siguientes aspectos de main.rs NO se testean directamente:
//
// 1. tracing_subscriber::fmt() initialization
//    - Difícil de testear porque es global state
//    - Se verifica manualmente durante desarrollo
//
// 2. std::process::exit(1)
//    - No se puede testear porque termina el proceso
//    - Se verifica que las condiciones que lo activan funcionen
//
// 3. TcpListener::bind() y axum::serve()
//    - Requiere puertos reales y puede conflictuar con tests paralelos
//    - Se testea en integration_test.rs con el router completo
//
// 4. tokio runtime (@tokio::main)
//    - Es manejado por tokio automáticamente
//    - Tests async verifican que funciona
//
// Cobertura esperada de main.rs: ~40-50%
// Esto es normal para código de bootstrap que principalmente:
// - Inicializa sistemas externos (logging, TCP)
// - Hace exit() en errores
// - Es código de pegamento sin lógica de negocio
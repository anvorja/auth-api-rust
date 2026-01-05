// src/infrastructure/http/middleware/test/cors_comprehensive_test.rs
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::infrastructure::http::middleware::create_cors_layer;

#[test]
fn test_development_cors_configuration() {
    let settings = Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 5,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-key-minimum-32-characters-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
        },
        security: SecuritySettings {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:9090".to_string(),
            ],
        },
        environment: Environment::Development,
    };

    let _cors = create_cors_layer(&settings);
    // Si no panic, el layer se creó correctamente
}

#[test]
fn test_production_cors_https_only() {
    let settings = Settings {
        server: ServerSettings {
            host: "0.0.0.0".to_string(),
            port: 443,
        },
        database: DatabaseSettings {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 5,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-key-minimum-32-characters-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
        },
        security: SecuritySettings {
            allowed_origins: vec![
                "https://api.example.com".to_string(),
                "https://app.example.com".to_string(),
            ],
        },
        environment: Environment::Production,
    };

    let _cors = create_cors_layer(&settings);
}

#[test]
#[should_panic(expected = "CORS en producción requiere al menos un origen HTTPS")]
fn test_production_cors_rejects_http() {
    let settings = Settings {
        server: ServerSettings {
            host: "0.0.0.0".to_string(),
            port: 443,
        },
        database: DatabaseSettings {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 5,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-key-minimum-32-characters-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
        },
        security: SecuritySettings {
            allowed_origins: vec![
                "http://insecure.com".to_string(), // ❌ HTTP en producción
            ],
        },
        environment: Environment::Production,
    };

    let _cors = create_cors_layer(&settings); // Debe panic
}

#[test]
fn test_test_environment_cors() {
    let settings = Settings {
        server: ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 5,
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
    };

    let _cors = create_cors_layer(&settings);
}
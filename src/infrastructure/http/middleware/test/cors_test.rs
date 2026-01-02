// src/infrastructure/http/middleware/test/cors_test.rs
use crate::config::{Settings, DatabaseSettings, JwtSettings, ServerSettings, SecuritySettings, Environment};
use crate::infrastructure::http::middleware::create_cors_layer;

#[test]
fn test_create_cors_layer() {
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

    // Si no panic, el layer se creó correctamente
}
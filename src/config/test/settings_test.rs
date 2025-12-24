// V1
// src/config/test/settings_test.rs
use crate::config::settings::{
    default_host, default_port, default_max_connections,
    default_access_token_expiry, Settings, ServerSettings,
    DatabaseSettings, JwtSettings, SecuritySettings, Environment
};

#[test]
fn test_default_values() {
    assert_eq!(default_host(), "127.0.0.1");
    assert_eq!(default_port(), 9090);
    assert_eq!(default_max_connections(), 10);
    assert_eq!(default_access_token_expiry(), 900);
}

#[test]
fn test_socket_addr() {
    let settings = Settings {
        server: ServerSettings {
            host: "0.0.0.0".to_string(),
            port: 8080,
        },
        database: DatabaseSettings {
            url: "".to_string(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: 30,
        },
        jwt: JwtSettings {
            secret: "test-secret-at-least-32-chars-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
        },
        security: SecuritySettings {
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
        environment: Environment::Test,
    };

    assert_eq!(settings.socket_addr(), "0.0.0.0:8080");
    assert!(!settings.is_development());
    assert!(!settings.is_production());
}
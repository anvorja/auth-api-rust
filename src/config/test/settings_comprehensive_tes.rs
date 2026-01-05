// src/config/test/settings_comprehensive_test.rs
use temp_env::{with_vars};
use crate::config::{Settings, Environment, ConfigError};

#[test]
fn test_load_full_settings_from_env() {
    with_vars(
        vec![
            ("SERVER_HOST", Some("0.0.0.0")),
            ("SERVER_PORT", Some("3000")),
            ("DATABASE_URL", Some("postgresql://user:pass@localhost/db")),
            ("DATABASE_MAX_CONNECTIONS", Some("20")),
            ("DATABASE_MIN_CONNECTIONS", Some("5")),
            ("DATABASE_ACQUIRE_TIMEOUT", Some("60")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("JWT_ACCESS_TOKEN_EXPIRY", Some("1800")),
            ("JWT_REFRESH_TOKEN_EXPIRY", Some("2592000")),
            ("ALLOWED_ORIGINS", Some("https://example.com,https://app.example.com")),
            ("ENVIRONMENT", Some("production")),
        ],
        || {
            let settings = Settings::from_env().unwrap();

            assert_eq!(settings.server.host, "0.0.0.0");
            assert_eq!(settings.server.port, 3000);
            assert_eq!(settings.database.max_connections, 20);
            assert_eq!(settings.database.min_connections, 5);
            assert_eq!(settings.database.acquire_timeout, 60);
            assert_eq!(settings.jwt.access_token_expiry, 1800);
            assert_eq!(settings.jwt.refresh_token_expiry, 2592000);
            assert_eq!(settings.security.allowed_origins.len(), 2);
            assert_eq!(settings.environment, Environment::Production);
        },
    );
}

#[test]
fn test_missing_database_url() {
    with_vars(
        vec![
            ("ENVIRONMENT", Some("production")), // ← ¡AGREGA ESTO!
            ("DATABASE_URL", None),
            ("SERVER_HOST", Some("127.0.0.1")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err(), "Expected error when DATABASE_URL is missing");
            if let Err(err) = result {
                assert!(matches!(err, ConfigError::MissingEnvVar(_)));
            }
        },
    );
}

#[test]
fn test_missing_jwt_secret() {
    with_vars(
        vec![
            ("ENVIRONMENT", Some("production")), // ← ¡AGREGA ESTO!
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", None),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err(), "Expected error when JWT_SECRET is missing");
            if let Err(err) = result {
                assert!(matches!(err, ConfigError::MissingEnvVar(_)));
            }
        },
    );
}

#[test]
fn test_insecure_jwt_secret() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("short")), // Menos de 32 caracteres
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ConfigError::InsecureJwtSecret));
        },
    );
}

#[test]
fn test_invalid_environment() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("invalid_env")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ConfigError::InvalidEnvironment(_)));
        },
    );
}

#[test]
fn test_invalid_port() {
    with_vars(
        vec![
            ("SERVER_PORT", Some("not_a_number")),
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ENVIRONMENT", Some("test")),
        ],
        || {
            let result = Settings::from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ConfigError::InvalidPort));
        },
    );
}

#[test]
fn test_default_values() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
        ],
        || {
            let settings = Settings::from_env().unwrap();

            // Defaults
            assert_eq!(settings.server.host, "127.0.0.1");
            assert_eq!(settings.server.port, 9090);
            assert_eq!(settings.database.max_connections, 10);
            assert_eq!(settings.database.min_connections, 2);
            assert_eq!(settings.jwt.access_token_expiry, 900);
            assert_eq!(settings.jwt.refresh_token_expiry, 604800);
            assert_eq!(settings.environment, Environment::Development);
        },
    );
}

#[test]
fn test_is_development() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
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
fn test_is_production() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
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

#[test]
fn test_allowed_origins_parsing() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ALLOWED_ORIGINS", Some("http://localhost:3000, https://app.com,https://api.com  ")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            assert_eq!(settings.security.allowed_origins.len(), 3);
            assert_eq!(settings.security.allowed_origins[0], "http://localhost:3000");
            assert_eq!(settings.security.allowed_origins[1], "https://app.com");
            assert_eq!(settings.security.allowed_origins[2], "https://api.com");
        },
    );
}

#[test]
fn test_empty_allowed_origins() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("postgresql://localhost/test")),
            ("JWT_SECRET", Some("test-secret-at-least-32-chars-long")),
            ("ALLOWED_ORIGINS", Some("")),
        ],
        || {
            let settings = Settings::from_env().unwrap();
            assert_eq!(settings.security.allowed_origins.len(), 0);
        },
    );
}
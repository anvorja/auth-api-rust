//src/config/settings.rs
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub security: SecuritySettings,
    pub environment: Environment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry: i64,
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

// Defaults - públicas para tests
pub(crate) fn default_host() -> String {
    "127.0.0.1".to_string()
}

pub(crate) fn default_port() -> u16 {
    9090
}

pub(crate) fn default_max_connections() -> u32 {
    10
}

pub(crate) fn default_min_connections() -> u32 {
    2
}

pub(crate) fn default_acquire_timeout() -> u64 {
    30
}

pub(crate) fn default_access_token_expiry() -> i64 {
    900 // 15 minutes
}

pub(crate) fn default_refresh_token_expiry() -> i64 {
    604800 // 7 days
}

impl Settings {
    /// Carga la configuración desde variables de entorno
    /// Sigue el patrón 12-Factor App
    pub fn from_env() -> Result<Self, ConfigError> {
        // Cargar .env solo en desarrollo
        if env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) != "production" {
            dotenvy::dotenv().ok();
        }

        let server = Self::load_server_settings()?;
        let database = Self::load_database_settings()?;
        let jwt = Self::load_jwt_settings()?;
        let security = Self::load_security_settings()?;
        let environment = Self::load_environment()?;

        Ok(Settings {
            server,
            database,
            jwt,
            security,
            environment,
        })
    }

    fn load_server_settings() -> Result<ServerSettings, ConfigError> {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| default_host());
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| default_port().to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort)?;

        Ok(ServerSettings { host, port })
    }

    fn load_database_settings() -> Result<DatabaseSettings, ConfigError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| default_max_connections().to_string())
            .parse::<u32>()
            .unwrap_or_else(|_| default_max_connections());

        let min_connections = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| default_min_connections().to_string())
            .parse::<u32>()
            .unwrap_or_else(|_| default_min_connections());

        let acquire_timeout = env::var("DATABASE_ACQUIRE_TIMEOUT")
            .unwrap_or_else(|_| default_acquire_timeout().to_string())
            .parse::<u64>()
            .unwrap_or_else(|_| default_acquire_timeout());

        Ok(DatabaseSettings {
            url,
            max_connections,
            min_connections,
            acquire_timeout,
        })
    }

    fn load_jwt_settings() -> Result<JwtSettings, ConfigError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        // Validación de seguridad: JWT secret debe tener al menos 32 caracteres
        if secret.len() < 32 {
            return Err(ConfigError::InsecureJwtSecret);
        }

        let access_token_expiry = env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| default_access_token_expiry().to_string())
            .parse::<i64>()
            .unwrap_or_else(|_| default_access_token_expiry());

        let refresh_token_expiry = env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| default_refresh_token_expiry().to_string())
            .parse::<i64>()
            .unwrap_or_else(|_| default_refresh_token_expiry());

        Ok(JwtSettings {
            secret,
            access_token_expiry,
            refresh_token_expiry,
        })
    }

    fn load_security_settings() -> Result<SecuritySettings, ConfigError> {
        let allowed_origins_str = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let allowed_origins = allowed_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(SecuritySettings { allowed_origins })
    }

    fn load_environment() -> Result<Environment, ConfigError> {
        let env_str = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        match env_str.to_lowercase().as_str() {
            "development" => Ok(Environment::Development),
            "production" => Ok(Environment::Production),
            "test" => Ok(Environment::Test),
            _ => Err(ConfigError::InvalidEnvironment(env_str)),
        }
    }

    /// Retorna el socket address completo
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Indica si estamos en modo desarrollo
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    /// Indica si estamos en modo producción
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Variable de entorno faltante: {0}")]
    MissingEnvVar(String),

    #[error("Puerto inválido en SERVER_PORT")]
    InvalidPort,

    #[error("JWT_SECRET debe tener al menos 32 caracteres para seguridad")]
    InsecureJwtSecret,

    #[error("Entorno inválido: {0}. Valores permitidos: development, production, test")]
    InvalidEnvironment(String),
}
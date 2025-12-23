// V2
//src/config/settings.rs
use std::env;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub server_host: String,
}

impl Settings {
    pub fn new() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| "secret_refresh".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1:9090".to_string()),
        })
    }
}

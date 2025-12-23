// V2
// src/main.rs
mod app;
mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::migrate::Migrator;

use crate::{
    app::AppState,
    application::auth_usecase::AuthUseCase,
    config::settings::Settings,
    infrastructure::{
        db,
        repositories::user_repository_sqlx::PostgresUserRepository,
        security::jwt::JwtService,
        http::routes,
        http::middleware::{cors, security_headers},
    },
};

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let settings = Settings::new().expect("Failed to load configuration");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&settings.database_url)) // Using DB_URL as Env var substitute for LOG, actually RUST_LOG
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting server at {}", settings.server_host);

    // Database connection
    let pool = db::create_pool(&settings.database_url).await?;

    // Run migrations
    tracing::info!("Running migrations...");
    MIGRATOR.run(&pool).await?;
    tracing::info!("Migrations executed successfully.");

    // Initialize dependencies
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let jwt_service = Arc::new(JwtService::new(
        settings.jwt_secret.clone(),
        settings.jwt_refresh_secret.clone(),
    ));
    let auth_usecase = Arc::new(AuthUseCase::new(user_repository, jwt_service));

    let state = AppState {
        auth_usecase,
        settings: settings.clone(),
    };

    // Router
    let app = routes::create_router(state)
        .layer(cors::cors())
        .layer(security_headers::x_content_type_options())
        .layer(security_headers::x_frame_options())
        .layer(security_headers::x_xss_protection());

    // Start server
    let listener = TcpListener::bind(&settings.server_host).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

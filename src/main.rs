// V1
// src/main.rs
mod app;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod presentation;

use config::Settings;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Inicializar el sistema de logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    tracing::info!("🚀 Iniciando Auth API en Rust...");

    // Cargar configuración
    let settings = match Settings::from_env() {
        Ok(settings) => settings,
        Err(e) => {
            tracing::error!("❌ Error al cargar configuración: {}", e);
            std::process::exit(1);
        }
    };

    // Mostrar información de la aplicación
    app::log_app_info(&settings);

    // Construir la aplicación
    let app = match app::build_app(settings.clone()).await {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("❌ Error al construir la aplicación: {}", e);
            std::process::exit(1);
        }
    };

    // Crear el listener TCP
    let addr = settings.socket_addr();
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => {
            tracing::info!("✅ Servidor escuchando en {}", addr);
            listener
        }
        Err(e) => {
            tracing::error!("❌ Error al bindear el puerto {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    tracing::info!("🎉 Servidor listo para recibir peticiones!");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Iniciar el servidor
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("❌ Error en el servidor: {}", e);
        std::process::exit(1);
    }
}
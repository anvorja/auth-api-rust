// src/infrastructure/http/middleware/test/cors_test.rs
use crate::config::Settings;
use crate::infrastructure::http::middleware::create_cors_layer;

#[test]
fn test_create_cors_layer() {
    unsafe {
        std::env::set_var("ALLOWED_ORIGINS", "http://localhost:3000");
        std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
    }

    let settings = Settings::from_env().unwrap();
    let _cors = create_cors_layer(&settings);

    // Si no panic, el layer se creó correctamente
}
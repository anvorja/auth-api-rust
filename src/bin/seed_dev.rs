// src/bin/seed_dev.rs
// Seed simple para poblar la base de datos en desarrollo con usuarios de prueba

// Uso: cargo run --bin seed_dev

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use chrono::Utc;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::env;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // ============================================================================
    // Verificar que NO estamos en producción
    // ============================================================================
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    if environment.to_lowercase() == "production" {
        eprintln!("❌ ERROR: NO ejecutar seed en producción");
        eprintln!();
        eprintln!("El entorno actual es: {}", environment);
        eprintln!("Los datos seed son solo para desarrollo/staging.");
        eprintln!();
        eprintln!("💡 Si necesitas datos en producción:");
        eprintln!("   - Usa migraciones con datos iniciales");
        eprintln!("   - O un proceso de importación controlado");
        eprintln!();
        std::process::exit(1);
    }

    println!();
    println!("✓ Entorno: {} (seguro para seed)", environment);
    println!();
    println!("⚙ Iniciando seed de usuarios de prueba...");

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar configurada en .env");

    println!("🗄 Conectando a la base de datos...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✓ Conectado exitosamente");
    println!();

    // Opcional: Limpiar usuarios existentes antes de escritura
    println!("🧹 Limpiando tabla de usuarios...");
    sqlx::query!("DELETE FROM users").execute(&pool).await?;
    println!("✓ Tabla limpiada");
    println!();

    println!("📦 Creando usuarios de prueba...");
    println!();

    // Definir usuarios de prueba
    let users = vec![
        ("admin", "Admin", "User", "admin@example.com", "Admin123!", "admin"),
        ("johndoe", "John", "Doe", "john.doe@example.com", "JohnDoe123!", "user"),
        ("janedoe", "Jane", "Doe", "jane.doe@example.com", "JaneDoe123!", "user"),
        ("testuser", "Test", "User", "test@example.com", "TestUser123!", "user"),
        ("developer", "Developer", "Account", "dev@example.com", "Developer123!", "user"),
    ];

    let mut created = 0;
    let mut skipped = 0;

    for (username, first_name, last_name, email, password, role) in users {
        // Verificar si el usuario ya existe
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 OR email = $2) as \"exists!\"",
            username,
            email
        )
            .fetch_one(&pool)
            .await?
            .exists;

        if exists {
            println!("⏭️  Usuario '{}' ya existe - omitido", username);
            skipped += 1;
            continue;
        }

        // Hashear password con Argon2
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .to_string();

        let id = Uuid::new_v4();
        let now = Utc::now();

        // Insertar usuario
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, first_name, last_name, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            id,
            username,
            email,
            first_name,
            last_name,
            password_hash,
            role,
            now,
            now
        )
            .execute(&pool)
            .await?;

        println!("✓ Usuario creado: {} ({})", username, email);
        created += 1;
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Resumen:");
    println!("   ✓ Usuarios creados: {}", created);
    println!("   - Usuarios omitidos: {}", skipped);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if created > 0 {
        println!();
        println!("✓ Seed completado exitosamente!");
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 Usuarios de prueba:");
        println!();
        println!("  Username: admin       Password: Admin123!");
        println!("  Username: johndoe     Password: JohnDoe123!");
        println!("  Username: janedoe     Password: JaneDoe123!");
        println!("  Username: testuser    Password: TestUser123!");
        println!("  Username: developer   Password: Developer123!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("-> Inicia la API con: cargo run");
        println!("📚  Documentación: http://localhost:9090/api/v1/swagger-ui/");
    }

    Ok(())
}
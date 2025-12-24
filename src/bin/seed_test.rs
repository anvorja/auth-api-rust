// src/bin/seed_test.rs
// Seed para la base de datos de TESTS

// Uso: cargo run --bin seed_test

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

    // Usar DB de test por defecto
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test".to_string());

    println!("Seeding test database: {}", mask_url(&database_url));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Limpiar datos existentes (para tests frescos)
    println!("Cleaning existing test data...");
    sqlx::query!("DELETE FROM users WHERE username LIKE 'test%' OR email LIKE 'test%'")
        .execute(&pool)
        .await?;

    // Usuarios básicos para tests
    let test_users = vec![
        ("testuser1", "Test", "User1", "test1@example.com", "TestPass123!"),
        ("testuser2", "Test", "User2", "test2@example.com", "TestPass123!"),
        ("testadmin", "Test", "Admin", "testadmin@example.com", "AdminPass123!"),
    ];

    let mut created = 0;

    for (username, first_name, last_name, email, password) in test_users {
        // Verificar si ya existe
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 OR email = $2) as \"exists!\"",
            username,
            email
        )
            .fetch_one(&pool)
            .await?
            .exists;

        if exists {
            println!("  - {} (skipped, already exists)", username);
            continue;
        }

        // Hashear password
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
            INSERT INTO users (id, username, email, first_name, last_name, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            username,
            email,
            first_name,
            last_name,
            password_hash,
            now,
            now
        )
            .execute(&pool)
            .await?;

        println!("  + {} ({})", username, email);
        created += 1;
    }

    println!("\nTest database seeded successfully!");
    println!("Created: {} users", created);

    Ok(())
}

fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}
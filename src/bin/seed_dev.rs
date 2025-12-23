// V2
// src/bin/seed_dev.rs
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
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await?;

    println!("Cleaning up users table...");
    sqlx::query!("DELETE FROM users").execute(&pool).await?;

    println!("Seeding users...");

    let users = vec![
        ("Alice", "Smith", "alice@example.com", "password123"),
        ("Bob", "Jones", "bob@example.com", "securePass456"),
    ];

    for (name, last_name, email, password) in users {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .to_string();
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO users (id, name, last_name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            id, name, last_name, email, password_hash, now, now
        )
        .execute(&pool)
        .await?;

        println!("Created user: {}", email);
    }

    println!("Seeding completed successfully.");
    Ok(())
}

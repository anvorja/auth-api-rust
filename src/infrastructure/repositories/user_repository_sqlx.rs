use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::error::Error;
use crate::domain::{user::User, repository::UserRepository};
use chrono::{DateTime, Utc};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<User, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO users (id, name, last_name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, last_name, email, password_hash, created_at, updated_at
            "#,
            user.id,
            user.name,
            user.last_name,
            user.email,
            user.password_hash,
            user.created_at,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: rec.id,
            name: rec.name,
            last_name: rec.last_name,
            email: rec.email,
            password_hash: rec.password_hash,
            created_at: rec.created_at,
            updated_at: rec.updated_at,
        })
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query!(
            r#"
            SELECT id, name, last_name, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(row) => Ok(Some(User {
                id: row.id,
                name: row.name,
                last_name: row.last_name,
                email: row.email,
                password_hash: row.password_hash,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let rec = sqlx::query!(
             r#"
            SELECT id, name, last_name, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(row) => Ok(Some(User {
                id: row.id,
                name: row.name,
                last_name: row.last_name,
                email: row.email,
                password_hash: row.password_hash,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }
}

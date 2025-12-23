// V1
// src/infrastructure/repositories/user_repository_sqlx.rs
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Email, PasswordHash, User, UserId, Username};
use crate::error::{AppError, AppResult};

/// Trait para abstraer el repositorio de usuarios
/// Permite testing con mocks y cambio de implementación
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    /// Crea un nuevo usuario en la base de datos
    async fn create(&self, user: &User) -> AppResult<()>;

    /// Busca un usuario por ID
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;

    /// Busca un usuario por username
    async fn find_by_username(&self, username: &Username) -> AppResult<Option<User>>;

    /// Busca un usuario por email
    async fn find_by_email(&self, email: &Email) -> AppResult<Option<User>>;

    /// Busca un usuario por username o email (útil para login)
    async fn find_by_username_or_email(&self, identifier: &str) -> AppResult<Option<User>>;

    /// Actualiza un usuario existente
    async fn update(&self, user: &User) -> AppResult<()>;

    /// Elimina un usuario por ID
    async fn delete(&self, id: UserId) -> AppResult<()>;

    /// Verifica si existe un usuario con el username dado
    async fn exists_by_username(&self, username: &Username) -> AppResult<bool>;

    /// Verifica si existe un usuario con el email dado
    async fn exists_by_email(&self, email: &Email) -> AppResult<bool>;
}

/// Implementación del repositorio usando SQLx y PostgreSQL
#[derive(Clone)]
pub struct UserRepositorySqlx {
    pool: PgPool,
}

impl UserRepositorySqlx {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convierte una fila de la base de datos a una entidad User
    fn row_to_user(
        id: Uuid,
        username: String,
        email: String,
        first_name: String,
        last_name: String,
        password_hash: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> AppResult<User> {
        let user_id = UserId::from_uuid(id);
        let username = Username::new(username)
            .map_err(|e| AppError::DatabaseError(format!("Invalid username from DB: {}", e)))?;
        let email = Email::new(email)
            .map_err(|e| AppError::DatabaseError(format!("Invalid email from DB: {}", e)))?;
        let password_hash = PasswordHash::from_hash(password_hash);

        Ok(User::from_repository(
            user_id,
            username,
            email,
            first_name,
            last_name,
            password_hash,
            created_at,
            updated_at,
        ))
    }
}

#[async_trait::async_trait]
impl UserRepository for UserRepositorySqlx {
    async fn create(&self, user: &User) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, first_name, last_name, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user.id().value(),
            user.username().value(),
            user.email().value(),
            user.first_name(),
            user.last_name(),
            user.password_hash().value(),
            user.created_at(),
            user.updated_at(),
        )
            .execute(&self.pool)
            .await?;

        tracing::debug!("Usuario creado: {}", user.username().value());
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id.value()
        )
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let user = Self::row_to_user(
                    row.id,
                    row.username,
                    row.email,
                    row.first_name,
                    row.last_name,
                    row.password_hash,
                    row.created_at,
                    row.updated_at,
                )?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &Username) -> AppResult<Option<User>> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username.value()
        )
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let user = Self::row_to_user(
                    row.id,
                    row.username,
                    row.email,
                    row.first_name,
                    row.last_name,
                    row.password_hash,
                    row.created_at,
                    row.updated_at,
                )?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> AppResult<Option<User>> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email.value()
        )
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let user = Self::row_to_user(
                    row.id,
                    row.username,
                    row.email,
                    row.first_name,
                    row.last_name,
                    row.password_hash,
                    row.created_at,
                    row.updated_at,
                )?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_username_or_email(&self, identifier: &str) -> AppResult<Option<User>> {
        let result = sqlx::query!(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, created_at, updated_at
            FROM users
            WHERE username = $1 OR email = $1
            "#,
            identifier
        )
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let user = Self::row_to_user(
                    row.id,
                    row.username,
                    row.email,
                    row.first_name,
                    row.last_name,
                    row.password_hash,
                    row.created_at,
                    row.updated_at,
                )?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, user: &User) -> AppResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET first_name = $2,
                last_name = $3,
                password_hash = $4,
                updated_at = $5
            WHERE id = $1
            "#,
            user.id().value(),
            user.first_name(),
            user.last_name(),
            user.password_hash().value(),
            user.updated_at(),
        )
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::UserNotFound);
        }

        tracing::debug!("Usuario actualizado: {}", user.id().to_string());
        Ok(())
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id.value()
        )
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::UserNotFound);
        }

        tracing::debug!("Usuario eliminado: {}", id.to_string());
        Ok(())
    }

    async fn exists_by_username(&self, username: &Username) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as "exists!"
            "#,
            username.value()
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(result.exists)
    }

    async fn exists_by_email(&self, email: &Email) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as "exists!"
            "#,
            email.value()
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(result.exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::create_pool;
    use crate::config::Settings;

    // Helper para crear settings de prueba
    async fn create_test_pool() -> PgPool {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db_test");
            std::env::set_var("JWT_SECRET", "test-secret-key-minimum-32-characters-long");
        }

        let settings = Settings::from_env().expect("Failed to load test settings");
        create_pool(&settings).await.expect("Failed to create test pool")
    }

    async fn create_test_user() -> User {
        User::new(
            Username::new("testuser".to_string()).unwrap(),
            Email::new("test@example.com".to_string()).unwrap(),
            "Test".to_string(),
            "User".to_string(),
            PasswordHash::from_hash("$argon2id$v=19$m=19456,t=2,p=1$...".to_string()),
        )
    }

    #[tokio::test]
    #[ignore] // Requiere PostgreSQL corriendo
    async fn test_create_user() {
        let pool = create_test_pool().await;
        let repo = UserRepositorySqlx::new(pool);
        let user = create_test_user().await;

        let result = repo.create(&user).await;
        assert!(result.is_ok());

        // Cleanup
        let _ = repo.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        let repo = UserRepositorySqlx::new(pool);
        let user = create_test_user().await;

        repo.create(&user).await.unwrap();

        let found = repo.find_by_id(user.id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), user.id());

        // Cleanup
        let _ = repo.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_username() {
        let pool = create_test_pool().await;
        let repo = UserRepositorySqlx::new(pool);
        let user = create_test_user().await;

        repo.create(&user).await.unwrap();

        let found = repo.find_by_username(user.username()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username().value(), user.username().value());

        // Cleanup
        let _ = repo.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_exists_by_username() {
        let pool = create_test_pool().await;
        let repo = UserRepositorySqlx::new(pool);
        let user = create_test_user().await;

        let exists_before = repo.exists_by_username(user.username()).await.unwrap();
        assert!(!exists_before);

        repo.create(&user).await.unwrap();

        let exists_after = repo.exists_by_username(user.username()).await.unwrap();
        assert!(exists_after);

        // Cleanup
        let _ = repo.delete(user.id()).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_duplicate_username() {
        let pool = create_test_pool().await;
        let repo = UserRepositorySqlx::new(pool);
        let user1 = create_test_user().await;
        let user2 = User::new(
            Username::new("testuser".to_string()).unwrap(),
            Email::new("another@example.com".to_string()).unwrap(),
            "Another".to_string(),
            "User".to_string(),
            PasswordHash::from_hash("$argon2id$...".to_string()),
        );

        repo.create(&user1).await.unwrap();
        let result = repo.create(&user2).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::UserAlreadyExists));

        // Cleanup
        let _ = repo.delete(user1.id()).await;
    }
}
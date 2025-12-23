// V2
// src/domain/repository.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::user::User;
use std::error::Error;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, Box<dyn Error + Send + Sync>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, Box<dyn Error + Send + Sync>>;
}

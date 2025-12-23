// V1
// src/infrastructure/repositories/mod.rs
pub mod user_repository_sqlx;

pub use user_repository_sqlx::{UserRepository, UserRepositorySqlx};
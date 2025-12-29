// V1
// src/infrastructure/repositories/mod.rs
pub mod user_repository_sqlx;

#[cfg(test)]
mod test;

pub use user_repository_sqlx::{UserRepository, UserRepositorySqlx};
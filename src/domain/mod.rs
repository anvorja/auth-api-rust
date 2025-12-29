// V1
// src/domain/mod.rs
pub mod user;

#[cfg(test)]
mod test;

pub use user::{Email, PasswordHash, User, UserId, Username};
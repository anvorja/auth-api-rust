// src/config/mod.rs
pub mod settings;

#[cfg(test)]
mod test;

pub use settings::{
    Settings,
    ServerSettings,
    DatabaseSettings,
    JwtSettings,
    SecuritySettings,
    Environment,
};
// src/infrastructure/security/password.rs
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

use crate::domain::PasswordHash as DomainPasswordHash;
use crate::error::AppError;

/// Trait para abstraer el hashing de passwords
/// Permite testing con mocks y cambio de implementación sin afectar casos de uso
#[async_trait::async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hashea un password plano
    async fn hash_password(&self, password: &str) -> Result<DomainPasswordHash, AppError>;

    /// Verifica un password plano contra un hash
    async fn verify_password(&self, password: &str, hash: &DomainPasswordHash) -> Result<bool, AppError>;
}

/// Implementación de PasswordHasher usando Argon2id
///
/// Argon2id es el algoritmo ganador de la Password Hashing Competition (PHC)
/// y es resistente a ataques de GPU, ASIC y side-channel.
///
/// Configuración enterprise-grade:
/// - Algorithm: Argon2id (híbrido, resistente a todos los ataques)
/// - Memory cost: 19 MiB (19456 KiB)
/// - Time cost: 2 iterations
/// - Parallelism: 1 thread
/// - Output length: 32 bytes
#[derive(Clone)]
pub struct ArgonPasswordHasher {
    argon2: Argon2<'static>,
}

impl ArgonPasswordHasher {
    /// Crea un nuevo hasher con configuración por defecto (segura)
    pub fn new() -> Self {
        Self::with_params(
            19456, // 19 MiB - balance entre seguridad y performance
            2,     // 2 iterations
            1,     // 1 thread (suficiente para passwords)
        )
    }

    /// Crea un hasher con parámetros personalizados
    ///
    /// # Parámetros
    /// - `memory_cost_kib`: Memoria en KiB (mínimo 8, recomendado 19456)
    /// - `time_cost`: Número de iteraciones (mínimo 1, recomendado 2-4)
    /// - `parallelism`: Número de threads paralelos (típicamente 1)
    pub fn with_params(memory_cost_kib: u32, time_cost: u32, parallelism: u32) -> Self {
        let params = Params::new(memory_cost_kib, time_cost, parallelism, None)
            .expect("Invalid Argon2 parameters");

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        Self { argon2 }
    }

    /// Configuración para testing (más rápida, menos segura)
    /// NUNCA usar en producción
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        Self::with_params(
            8,    // 8 KiB - mínimo
            1,    // 1 iteration - mínimo
            1,    // 1 thread
        )
    }
}

impl Default for ArgonPasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PasswordHasher for ArgonPasswordHasher {
    async fn hash_password(&self, password: &str) -> Result<DomainPasswordHash, AppError> {
        // Validación básica del password
        if password.is_empty() {
            return Err(AppError::ValidationError("Password vacío".to_string()));
        }

        if password.len() > 128 {
            return Err(AppError::ValidationError(
                "Password demasiado largo (máximo 128 caracteres)".to_string(),
            ));
        }

        // El hashing es CPU-intensivo, ejecutarlo en un thread bloqueante
        let password_owned = password.to_string();
        let argon2 = self.argon2.clone();

        let hash = tokio::task::spawn_blocking(move || {
            // Generar salt criptográficamente seguro
            let salt = SaltString::generate(&mut OsRng);

            // Hashear el password
            argon2
                .hash_password(password_owned.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        })
            .await
            .map_err(|_| AppError::PasswordHashError)?
            .map_err(|_| AppError::PasswordHashError)?;

        Ok(DomainPasswordHash::from_hash(hash))
    }

    async fn verify_password(&self, password: &str, hash: &DomainPasswordHash) -> Result<bool, AppError> {
        // Validación básica
        if password.is_empty() {
            return Ok(false);
        }

        let password_owned = password.to_string();
        let hash_owned = hash.value().to_string();
        let argon2 = self.argon2.clone();

        // La verificación es CPU-intensiva, ejecutarla en un thread bloqueante
        let result = tokio::task::spawn_blocking(move || {
            // Parsear el hash almacenado
            let parsed_hash = PasswordHash::new(&hash_owned)?;

            // Verificar el password
            argon2.verify_password(password_owned.as_bytes(), &parsed_hash)
        })
            .await
            .map_err(|_| AppError::PasswordHashError)?;

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
// V1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_password() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let password = "MySecurePassword123!";

        let hash = hasher.hash_password(password).await.unwrap();

        // Verificar que el hash no está vacío
        assert!(!hash.value().is_empty());

        // Verificar que el hash tiene el prefijo de Argon2id
        assert!(hash.value().starts_with("$argon2id$"));
    }

    #[tokio::test]
    async fn test_verify_password_correct() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let password = "MySecurePassword123!";

        let hash = hasher.hash_password(password).await.unwrap();
        let is_valid = hasher.verify_password(password, &hash).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_verify_password_incorrect() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let password = "MySecurePassword123!";
        let wrong_password = "WrongPassword123!";

        let hash = hasher.hash_password(password).await.unwrap();
        let is_valid = hasher.verify_password(wrong_password, &hash).await.unwrap();

        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_hash_password_different_salts() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let password = "MySecurePassword123!";

        let hash1 = hasher.hash_password(password).await.unwrap();
        let hash2 = hasher.hash_password(password).await.unwrap();

        // Los hashes deben ser diferentes (diferentes salts)
        assert_ne!(hash1.value(), hash2.value());

        // Pero ambos deben verificar correctamente
        assert!(hasher.verify_password(password, &hash1).await.unwrap());
        assert!(hasher.verify_password(password, &hash2).await.unwrap());
    }

    #[tokio::test]
    async fn test_hash_empty_password() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let result = hasher.hash_password("").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_hash_too_long_password() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let long_password = "a".repeat(129); // Más de 128 caracteres

        let result = hasher.hash_password(&long_password).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_verify_empty_password() {
        let hasher = ArgonPasswordHasher::new_for_testing();
        let password = "MySecurePassword123!";
        let hash = hasher.hash_password(password).await.unwrap();

        let is_valid = hasher.verify_password("", &hash).await.unwrap();

        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_argon2_params() {
        // Test con parámetros personalizados
        let hasher = ArgonPasswordHasher::with_params(8, 1, 1);
        let password = "TestPassword123!";

        let hash = hasher.hash_password(password).await.unwrap();
        let is_valid = hasher.verify_password(password, &hash).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_concurrent_hashing() {
        let hasher = ArgonPasswordHasher::new_for_testing();

        // Hash múltiples passwords concurrentemente
        let tasks: Vec<_> = (0..5)
            .map(|i| {
                let hasher = hasher.clone();
                let password = format!("Password{}!", i);
                tokio::spawn(async move {
                    hasher.hash_password(&password).await
                })
            })
            .collect();

        // Esperar a que todos terminen
        let results: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap().unwrap())
            .collect();

        // Todos deben tener hashes válidos
        assert_eq!(results.len(), 5);
        for hash in results {
            assert!(hash.value().starts_with("$argon2id$"));
        }
    }
}
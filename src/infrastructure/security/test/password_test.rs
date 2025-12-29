// src/infrastructure/security/test/password_test.rs
use crate::infrastructure::security::{PasswordHasher, ArgonPasswordHasher};
use crate::error::AppError;

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
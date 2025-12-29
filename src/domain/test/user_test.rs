// src/domain/test/user_test.rs
use crate::domain::{User, Username, Email, UserId, PasswordHash};

#[test]
fn test_user_creation() {
    let user = User::new(
        Username::new("johndoe".to_string()).unwrap(),
        Email::new("john@example.com".to_string()).unwrap(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("$argon2id$...".to_string()),
    );

    assert_eq!(user.username().value(), "johndoe");
    assert_eq!(user.email().value(), "john@example.com");
    assert_eq!(user.full_name(), "John Doe");
}

#[test]
fn test_username_validation() {
    // Válidos
    assert!(Username::new("abc".to_string()).is_ok());
    assert!(Username::new("user_123".to_string()).is_ok());
    assert!(Username::new("a".to_string().repeat(30)).is_ok());

    // Inválidos
    assert!(Username::new("ab".to_string()).is_err()); // muy corto
    assert!(Username::new("a".to_string().repeat(31)).is_err()); // muy largo
    assert!(Username::new("123abc".to_string()).is_err()); // no empieza con letra
    assert!(Username::new("user-name".to_string()).is_err()); // carácter inválido
    assert!(Username::new("user name".to_string()).is_err()); // espacio
}

#[test]
fn test_username_normalization() {
    let username = Username::new("  JohnDoe  ".to_string()).unwrap();
    assert_eq!(username.value(), "johndoe"); // lowercase y trimmed
}

#[test]
fn test_email_validation() {
    // Válidos
    assert!(Email::new("user@example.com".to_string()).is_ok());
    assert!(Email::new("user.name@example.co.uk".to_string()).is_ok());
    assert!(Email::new("user+tag@example.com".to_string()).is_ok());

    // Inválidos
    assert!(Email::new("".to_string()).is_err());
    assert!(Email::new("invalid".to_string()).is_err());
    assert!(Email::new("@example.com".to_string()).is_err());
    assert!(Email::new("user@".to_string()).is_err());
    assert!(Email::new("user@domain".to_string()).is_err());
    assert!(Email::new("user@.domain.com".to_string()).is_err());
}

#[test]
fn test_email_normalization() {
    let email = Email::new("  User@Example.COM  ".to_string()).unwrap();
    assert_eq!(email.value(), "user@example.com");
}

#[test]
fn test_user_id_generation() {
    let id1 = UserId::new();
    let id2 = UserId::new();
    assert_ne!(id1, id2); // IDs únicos
}

#[test]
fn test_user_id_from_string() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let user_id = UserId::from_string(uuid_str).unwrap();
    assert_eq!(user_id.to_string(), uuid_str);
}

#[test]
fn test_update_password() {
    let mut user = User::new(
        Username::new("johndoe".to_string()).unwrap(),
        Email::new("john@example.com".to_string()).unwrap(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("old_hash".to_string()),
    );

    let old_updated = user.updated_at();
    std::thread::sleep(std::time::Duration::from_millis(10));

    user.update_password(PasswordHash::from_hash("new_hash".to_string()));

    assert_eq!(user.password_hash().value(), "new_hash");
    assert!(user.updated_at() > old_updated);
}

#[test]
fn test_update_profile() {
    let mut user = User::new(
        Username::new("johndoe".to_string()).unwrap(),
        Email::new("john@example.com".to_string()).unwrap(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );

    user.update_profile("Jane".to_string(), "Smith".to_string());

    assert_eq!(user.first_name(), "Jane");
    assert_eq!(user.last_name(), "Smith");
    assert_eq!(user.full_name(), "Jane Smith");
}

#[test]
fn test_user_from_repository() {
    use chrono::Utc;

    let user_id = UserId::new();
    let username = Username::new("johndoe".to_string()).unwrap();
    let email = Email::new("john@example.com".to_string()).unwrap();
    let created_at = Utc::now();
    let updated_at = Utc::now();

    let user = User::from_repository(
        user_id,
        username.clone(),
        email.clone(),
        "John".to_string(),
        "Doe".to_string(),
        PasswordHash::from_hash("$argon2id$...".to_string()),
        created_at,
        updated_at,
    );

    assert_eq!(user.id(), user_id);
    assert_eq!(user.username().value(), "johndoe");
    assert_eq!(user.email().value(), "john@example.com");
    assert_eq!(user.created_at(), created_at);
    assert_eq!(user.updated_at(), updated_at);
}

#[test]
fn test_password_hash() {
    let hash_value = "$argon2id$v=19$m=19456,t=2,p=1$...".to_string();
    let password_hash = PasswordHash::from_hash(hash_value.clone());

    assert_eq!(password_hash.value(), hash_value);
}

#[test]
fn test_username_edge_cases() {
    // Username de exactamente 3 caracteres (mínimo)
    assert!(Username::new("abc".to_string()).is_ok());

    // Username de exactamente 30 caracteres (máximo)
    let max_username = "a".to_string().repeat(30);
    assert!(Username::new(max_username.clone()).is_ok());
    assert_eq!(Username::new(max_username).unwrap().value().len(), 30);

    // Username con guiones bajos
    assert!(Username::new("user_name_test".to_string()).is_ok());

    // Username con números
    assert!(Username::new("user123".to_string()).is_ok());
}

#[test]
fn test_email_edge_cases() {
    // Email con múltiples puntos en el dominio
    assert!(Email::new("user@mail.example.com".to_string()).is_ok());

    // Email con números
    assert!(Email::new("user123@example.com".to_string()).is_ok());

    // Email con guiones
    assert!(Email::new("user-name@example.com".to_string()).is_ok());
}

#[test]
fn test_user_id_display() {
    let user_id = UserId::new();
    let uuid_string = user_id.to_string();

    // Verificar que el formato UUID es correcto (36 caracteres con guiones)
    assert_eq!(uuid_string.len(), 36);
    assert_eq!(uuid_string.chars().filter(|&c| c == '-').count(), 4);
}
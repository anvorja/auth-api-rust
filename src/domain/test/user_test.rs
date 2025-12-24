// src/domain/test/user_test.rs
use crate::domain::{User, Username, Email, PasswordHash, UserId};

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
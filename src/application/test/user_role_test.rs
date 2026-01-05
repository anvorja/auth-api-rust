// src/domain/test/user_role_test.rs
use crate::domain::UserRole;

#[test]
fn test_user_role_from_string() {
    assert_eq!(UserRole::from_string("user").unwrap(), UserRole::User);
    assert_eq!(UserRole::from_string("User").unwrap(), UserRole::User);
    assert_eq!(UserRole::from_string("USER").unwrap(), UserRole::User);
    assert_eq!(UserRole::from_string("admin").unwrap(), UserRole::Admin);
    assert_eq!(UserRole::from_string("Admin").unwrap(), UserRole::Admin);

    assert!(UserRole::from_string("invalid").is_err());
}

#[test]
fn test_user_role_to_string() {
    assert_eq!(UserRole::User.to_string(), "user");
    assert_eq!(UserRole::Admin.to_string(), "admin");
}

#[test]
fn test_user_role_is_admin() {
    assert!(!UserRole::User.is_admin());
    assert!(UserRole::Admin.is_admin());
}

#[test]
fn test_user_role_default() {
    assert_eq!(UserRole::default(), UserRole::User);
}
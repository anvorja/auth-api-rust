// src/infrastructure/http/middleware/test/auth_test.rs
use crate::domain::{Email, PasswordHash, User, Username};
use crate::infrastructure::http::middleware::AuthenticatedUser;

#[test]
fn test_authenticated_user_clone() {
    let user = User::new(
        Username::new("testuser".to_string()).unwrap(),
        Email::new("test@example.com".to_string()).unwrap(),
        "Test".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );

    let auth_user = AuthenticatedUser(user.clone());
    let cloned = auth_user.clone();

    assert_eq!(auth_user.0.id(), cloned.0.id());
}
// src/infrastructure/http/handlers/test/user_handlers_test.rs
use crate::domain::{Email, PasswordHash, User, UserId, Username, UserRole};
use crate::infrastructure::http::middleware::AuthenticatedUser;
use crate::presentation::{ UpdateProfileRequest, AdminUpdateUsernameRequest, AdminUpdateEmailRequest,
    ChangePasswordRequest, UserResponse,
};

// ============================================================================
// TESTS UNITARIOS - HANDLERS BÁSICOS
// ============================================================================

#[test]
fn test_authenticated_user_extension() {
    let user = User::new(
        Username::new("testuser".to_string()).unwrap(),
        Email::new("test@example.com".to_string()).unwrap(),
        "Test".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );

    let auth_user = AuthenticatedUser(user.clone());
    assert_eq!(auth_user.0.id(), user.id());
}

#[test]
fn test_update_profile_request_validation() {
    use validator::Validate;

    // Request válido
    let valid_request = UpdateProfileRequest {
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Nombre vacío - inválido
    let invalid_request = UpdateProfileRequest {
        first_name: "".to_string(),
        last_name: "Doe".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Apellido muy largo - inválido
    let invalid_request = UpdateProfileRequest {
        first_name: "Jane".to_string(),
        last_name: "a".repeat(101),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_change_password_request_validation() {
    use validator::Validate;

    // Request válido
    let valid_request = ChangePasswordRequest {
        current_password: "OldPassword123!".to_string(),
        new_password: "NewPassword456!".to_string(),
        new_password_confirmation: "NewPassword456!".to_string(),
    };
    assert!(valid_request.validate().is_ok());
    assert!(valid_request.passwords_match());

    // Contraseñas no coinciden
    let request = ChangePasswordRequest {
        current_password: "OldPassword123!".to_string(),
        new_password: "NewPassword456!".to_string(),
        new_password_confirmation: "DifferentPassword789!".to_string(),
    };
    assert!(!request.passwords_match());

    // Nueva contraseña débil
    let invalid_request = ChangePasswordRequest {
        current_password: "OldPassword123!".to_string(),
        new_password: "weak".to_string(),
        new_password_confirmation: "weak".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_admin_update_username_request_validation() {
    use validator::Validate;
    use uuid::Uuid;

    // Request válido
    let valid_request = AdminUpdateUsernameRequest {
        user_id: Uuid::new_v4().to_string(),
        new_username: "newusername".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Username muy corto
    let invalid_request = AdminUpdateUsernameRequest {
        user_id: Uuid::new_v4().to_string(),
        new_username: "ab".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Username muy largo
    let invalid_request = AdminUpdateUsernameRequest {
        user_id: Uuid::new_v4().to_string(),
        new_username: "a".repeat(31),
    };
    assert!(invalid_request.validate().is_err());

    // Username con caracteres inválidos
    let invalid_request = AdminUpdateUsernameRequest {
        user_id: Uuid::new_v4().to_string(),
        new_username: "user-name".to_string(), // guión no permitido
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_admin_update_email_request_validation() {
    use validator::Validate;
    use uuid::Uuid;

    // Request válido
    let valid_request = AdminUpdateEmailRequest {
        user_id: Uuid::new_v4().to_string(),
        new_email: "newemail@example.com".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Email inválido
    let invalid_request = AdminUpdateEmailRequest {
        user_id: Uuid::new_v4().to_string(),
        new_email: "invalid-email".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    // Email vacío
    let invalid_request = AdminUpdateEmailRequest {
        user_id: Uuid::new_v4().to_string(),
        new_email: "".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

// ============================================================================
// TESTS DE LÓGICA DE NEGOCIO
// ============================================================================

#[test]
fn test_user_is_admin_check() {
    // Usuario normal
    let normal_user = User::new(
        Username::new("normaluser".to_string()).unwrap(),
        Email::new("normal@example.com".to_string()).unwrap(),
        "Normal".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );
    assert!(!normal_user.is_admin());
    assert_eq!(normal_user.role(), UserRole::User);

    // Usuario admin (necesitamos crearlo desde repository con rol admin)
    let admin_user = User::from_repository(
        UserId::new(),
        Username::new("adminuser".to_string()).unwrap(),
        Email::new("admin@example.com".to_string()).unwrap(),
        "Admin".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
        UserRole::Admin,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    assert!(admin_user.is_admin());
    assert_eq!(admin_user.role(), UserRole::Admin);
}

#[test]
fn test_user_response_from_domain() {
    let user = User::new(
        Username::new("testuser".to_string()).unwrap(),
        Email::new("test@example.com".to_string()).unwrap(),
        "Test".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );

    let response = UserResponse::from_domain(&user);

    assert_eq!(response.username, "testuser");
    assert_eq!(response.email, "test@example.com");
    assert_eq!(response.first_name, "Test");
    assert_eq!(response.last_name, "User");
    assert_eq!(response.role, "user");
    assert_eq!(response.id, user.id().value());
}

#[test]
fn test_username_validation_in_domain() {
    // Válidos
    assert!(Username::new("validuser".to_string()).is_ok());
    assert!(Username::new("user123".to_string()).is_ok());
    assert!(Username::new("user_name".to_string()).is_ok());

    // Inválidos
    assert!(Username::new("ab".to_string()).is_err()); // muy corto
    assert!(Username::new("a".repeat(31)).is_err()); // muy largo
    assert!(Username::new("123user".to_string()).is_err()); // empieza con número
    assert!(Username::new("user-name".to_string()).is_err()); // guión no permitido
}

#[test]
fn test_email_validation_in_domain() {
    // Válidos
    assert!(Email::new("user@example.com".to_string()).is_ok());
    assert!(Email::new("user.name@example.co.uk".to_string()).is_ok());

    // Inválidos
    assert!(Email::new("invalid".to_string()).is_err());
    assert!(Email::new("@example.com".to_string()).is_err());
    assert!(Email::new("user@".to_string()).is_err());
    assert!(Email::new("".to_string()).is_err());
}

#[test]
fn test_user_id_parsing() {
    // UUID válido
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let user_id = UserId::from_string(uuid_str);
    assert!(user_id.is_ok());

    // UUID inválido
    let invalid_uuid = "invalid-uuid";
    let result = UserId::from_string(invalid_uuid);
    assert!(result.is_err());
}

// ============================================================================
// TESTS DE EDGE CASES
// ============================================================================

#[test]
fn test_update_profile_with_special_characters() {
    use validator::Validate;

    // Nombres con caracteres especiales válidos
    let request = UpdateProfileRequest {
        first_name: "José".to_string(),
        last_name: "García".to_string(),
    };
    // Debería validar OK porque validator permite cualquier UTF-8
    assert!(request.validate().is_ok());
}

#[test]
fn test_username_case_sensitivity() {
    // El username se normaliza a lowercase
    let username1 = Username::new("TestUser".to_string()).unwrap();
    let username2 = Username::new("testuser".to_string()).unwrap();

    assert_eq!(username1.value(), "testuser");
    assert_eq!(username2.value(), "testuser");
    assert_eq!(username1.value(), username2.value());
}

#[test]
fn test_email_case_sensitivity() {
    // El email se normaliza a lowercase
    let email1 = Email::new("User@Example.COM".to_string()).unwrap();
    let email2 = Email::new("user@example.com".to_string()).unwrap();

    assert_eq!(email1.value(), "user@example.com");
    assert_eq!(email2.value(), "user@example.com");
    assert_eq!(email1.value(), email2.value());
}

#[test]
fn test_password_matching_edge_cases() {
    // Contraseñas idénticas
    let request = ChangePasswordRequest {
        current_password: "Current123!".to_string(),
        new_password: "New123!".to_string(),
        new_password_confirmation: "New123!".to_string(),
    };
    assert!(request.passwords_match());

    // Case sensitive - no coinciden
    let request = ChangePasswordRequest {
        current_password: "Current123!".to_string(),
        new_password: "New123!".to_string(),
        new_password_confirmation: "new123!".to_string(),
    };
    assert!(!request.passwords_match());

    // Espacios en blanco - no coinciden
    let request = ChangePasswordRequest {
        current_password: "Current123!".to_string(),
        new_password: "New123!".to_string(),
        new_password_confirmation: "New123! ".to_string(),
    };
    assert!(!request.passwords_match());
}

// ============================================================================
// TESTS DE BÚSQUEDA - SearchUserQuery
// ============================================================================

#[test]
fn test_search_user_query_deserialization() {
    // Este test verifica que la query parameter se deserialice correctamente
    let query_str = "identifier=testuser";

    // En un escenario real, axum hace esto automáticamente
    // Aquí simulamos la estructura que recibiríamos
    use serde::Deserialize;
    use serde_urlencoded;

    #[derive(Debug, Deserialize)]
    struct SearchQuery {
        identifier: String,
    }

    let query: SearchQuery = serde_urlencoded::from_str(query_str).unwrap();
    assert_eq!(query.identifier, "testuser");
}

#[test]
fn test_search_query_with_email() {
    use serde::Deserialize;
    use serde_urlencoded;

    #[derive(Debug, Deserialize)]
    struct SearchQuery {
        identifier: String,
    }

    let query_str = "identifier=user@example.com";
    let query: SearchQuery = serde_urlencoded::from_str(query_str).unwrap();
    assert_eq!(query.identifier, "user@example.com");
}

// ============================================================================
// TESTS DE RESPUESTAS
// ============================================================================

#[test]
fn test_user_response_includes_all_fields() {
    let user = User::from_repository(
        UserId::new(),
        Username::new("fulluser".to_string()).unwrap(),
        Email::new("full@example.com".to_string()).unwrap(),
        "Full".to_string(),
        "Name".to_string(),
        PasswordHash::from_hash("hash".to_string()),
        UserRole::Admin,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let response = UserResponse::from_domain(&user);

    // Verificar que todos los campos están presentes
    assert!(!response.id.to_string().is_empty());
    assert!(!response.username.is_empty());
    assert!(!response.email.is_empty());
    assert!(!response.first_name.is_empty());
    assert!(!response.last_name.is_empty());
    assert!(!response.role.is_empty());
    // created_at y updated_at son timestamps válidos
}

#[test]
fn test_user_response_role_serialization() {
    // Usuario normal
    let normal_user = User::new(
        Username::new("normaluser".to_string()).unwrap(),
        Email::new("normal@example.com".to_string()).unwrap(),
        "Normal".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
    );
    let response = UserResponse::from_domain(&normal_user);
    assert_eq!(response.role, "user");

    // Usuario admin
    let admin_user = User::from_repository(
        UserId::new(),
        Username::new("adminuser".to_string()).unwrap(),
        Email::new("admin@example.com".to_string()).unwrap(),
        "Admin".to_string(),
        "User".to_string(),
        PasswordHash::from_hash("hash".to_string()),
        UserRole::Admin,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    let response = UserResponse::from_domain(&admin_user);
    assert_eq!(response.role, "admin");
}

// ============================================================================
// TESTS DE LÍMITES Y RANGOS
// ============================================================================

#[test]
fn test_username_length_boundaries() {
    // Mínimo exacto (3 caracteres)
    assert!(Username::new("abc".to_string()).is_ok());

    // Uno menos del mínimo
    assert!(Username::new("ab".to_string()).is_err());

    // Máximo exacto (30 caracteres)
    let max_username = "a".repeat(30);
    assert!(Username::new(max_username).is_ok());

    // Uno más del máximo
    let too_long = "a".repeat(31);
    assert!(Username::new(too_long).is_err());
}

#[test]
fn test_profile_field_length_boundaries() {
    use validator::Validate;

    // Nombres en el límite superior (100 caracteres)
    let request = UpdateProfileRequest {
        first_name: "a".repeat(100),
        last_name: "b".repeat(100),
    };
    assert!(request.validate().is_ok());

    // Excediendo el límite
    let request = UpdateProfileRequest {
        first_name: "a".repeat(101),
        last_name: "Doe".to_string(),
    };
    assert!(request.validate().is_err());

    // Mínimo (1 carácter)
    let request = UpdateProfileRequest {
        first_name: "J".to_string(),
        last_name: "D".to_string(),
    };
    assert!(request.validate().is_ok());
}

#[test]
fn test_password_minimum_requirements() {
    use validator::Validate;

    // Cumple todos los requisitos
    let request = ChangePasswordRequest {
        current_password: "Old123!".to_string(),
        new_password: "Valid123!".to_string(),
        new_password_confirmation: "Valid123!".to_string(),
    };
    assert!(request.validate().is_ok());

    // Muy corta (menos de 8 caracteres)
    let request = ChangePasswordRequest {
        current_password: "Old123!".to_string(),
        new_password: "Val123!".to_string(),
        new_password_confirmation: "Val123!".to_string(),
    };
    assert!(request.validate().is_err());

    // Sin mayúscula
    let request = ChangePasswordRequest {
        current_password: "Old123!".to_string(),
        new_password: "valid123!".to_string(),
        new_password_confirmation: "valid123!".to_string(),
    };
    assert!(request.validate().is_err());

    // Sin minúscula
    let request = ChangePasswordRequest {
        current_password: "Old123!".to_string(),
        new_password: "VALID123!".to_string(),
        new_password_confirmation: "VALID123!".to_string(),
    };
    assert!(request.validate().is_err());

    // Sin número
    let request = ChangePasswordRequest {
        current_password: "Old123!".to_string(),
        new_password: "ValidPass!".to_string(),
        new_password_confirmation: "ValidPass!".to_string(),
    };
    assert!(request.validate().is_err());
}
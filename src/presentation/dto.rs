use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterUserRequest {
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,
    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub last_name: String,
    pub email: String,
}

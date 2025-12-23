use std::sync::Arc;
use std::error::Error;
use crate::domain::{user::User, repository::UserRepository};
use crate::infrastructure::security::{password::PasswordService, jwt::JwtService};

pub struct AuthUseCase {
    user_repository:  Arc<dyn UserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self { user_repository, jwt_service }
    }

    pub async fn register_user(&self, name: String, last_name: String, email: String, password: String)
        -> Result<User, Box<dyn Error + Send + Sync>>
    {
        if let Some(_) = self.user_repository.find_by_email(&email).await? {
            return Err("User already exists".into());
        }

        let password_hash = PasswordService::hash_password(&password)?;
        let new_user = User::new(name, last_name, email, password_hash);

        self.user_repository.create(&new_user).await
    }

    pub async fn login_user(&self, email: String, password: String)
        -> Result<(String, String), Box<dyn Error + Send + Sync>>
    {
        let user = self.user_repository.find_by_email(&email).await?
            .ok_or("Invalid credentials")?;

        if !PasswordService::verify_password(&password, &user.password_hash)? {
            return Err("Invalid credentials".into());
        }

        let access_token = self.jwt_service.generate_token(user.id)?;
        let refresh_token = self.jwt_service.generate_refresh_token(user.id)?;

        Ok((access_token, refresh_token))
    }

    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?.claims;
        let user_id = uuid::Uuid::parse_str(&claims.sub)?;
        let new_access_token = self.jwt_service.generate_token(user_id)?;
        Ok(new_access_token)
    }

    pub fn jwt_service(&self) -> &JwtService {
        &self.jwt_service
    }
}

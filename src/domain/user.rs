use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, last_name: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            last_name,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}

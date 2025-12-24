// V1
// src/domain/user.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Entidad de dominio: Usuario
/// Esta es una entidad PURA - NO tiene dependencias de infraestructura
/// NO usa Serde, NO usa SQLx, NO usa HTTP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    first_name: String,
    last_name: String,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Value Object: ID del usuario
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

/// Value Object: Username
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

/// Value Object: Email
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

/// Value Object: Password hasheado
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

// ============================================================================
// Implementación de User
// ============================================================================

impl User {
    /// Crea un nuevo usuario (constructor para registro)
    pub fn new(
        username: Username,
        email: Email,
        first_name: String,
        last_name: String,
        password_hash: PasswordHash,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: UserId::new(),
            username,
            email,
            first_name,
            last_name,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruye un usuario desde la base de datos
    pub fn from_repository(
        id: UserId,
        username: Username,
        email: Email,
        first_name: String,
        last_name: String,
        password_hash: PasswordHash,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            first_name,
            last_name,
            password_hash,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Actualiza el password hash
    pub fn update_password(&mut self, new_password_hash: PasswordHash) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }

    /// Actualiza información del perfil
    pub fn update_profile(&mut self, first_name: String, last_name: String) {
        self.first_name = first_name;
        self.last_name = last_name;
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Implementación de UserId
// ============================================================================

impl UserId {
    /// Genera un nuevo ID único
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Crea desde un Uuid existente
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parsea desde string
    pub fn from_string(s: &str) -> Result<Self, String> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| "ID de usuario inválido".to_string())
    }

    /// Obtiene el valor interno
    pub fn value(&self) -> Uuid {
        self.0
    }

    /// Convierte a string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Implementación de Username
// ============================================================================

impl Username {
    const MIN_LENGTH: usize = 3;
    const MAX_LENGTH: usize = 30;

    /// Crea un nuevo username con validaciones de dominio
    pub fn new(value: String) -> Result<Self, String> {
        let trimmed = value.trim().to_lowercase();

        // Validación: longitud
        if trimmed.len() < Self::MIN_LENGTH {
            return Err(format!(
                "El username debe tener al menos {} caracteres",
                Self::MIN_LENGTH
            ));
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(format!(
                "El username no puede tener más de {} caracteres",
                Self::MAX_LENGTH
            ));
        }

        // Validación: formato (alfanumérico + guión bajo)
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(
                "El username solo puede contener letras, números y guión bajo".to_string(),
            );
        }

        // Validación: debe empezar con letra
        if !trimmed.chars().next().unwrap().is_ascii_alphabetic() {
            return Err("El username debe comenzar con una letra".to_string());
        }

        Ok(Self(trimmed))
    }

    /// Obtiene el valor interno
    pub fn value(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Implementación de Email
// ============================================================================

impl Email {
    const MAX_LENGTH: usize = 255;

    /// Crea un nuevo email con validaciones de dominio
    pub fn new(value: String) -> Result<Self, String> {
        let trimmed = value.trim().to_lowercase();

        // Validación: longitud
        if trimmed.is_empty() {
            return Err("El email no puede estar vacío".to_string());
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(format!(
                "El email no puede tener más de {} caracteres",
                Self::MAX_LENGTH
            ));
        }

        // Validación: formato básico
        if !Self::is_valid_format(&trimmed) {
            return Err("Formato de email inválido".to_string());
        }

        Ok(Self(trimmed))
    }

    /// Validación básica de formato de email
    fn is_valid_format(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();

        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Local part no puede estar vacío
        if local.is_empty() || domain.is_empty() {
            return false;
        }

        // Domain debe tener al menos un punto
        if !domain.contains('.') {
            return false;
        }

        // Domain no puede empezar o terminar con punto
        if domain.starts_with('.') || domain.ends_with('.') {
            return false;
        }

        true
    }

    /// Obtiene el valor interno
    pub fn value(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Implementación de PasswordHash
// ============================================================================

impl PasswordHash {
    /// Crea desde un hash ya procesado
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Obtiene el valor del hash
    pub fn value(&self) -> &str {
        &self.0
    }
}
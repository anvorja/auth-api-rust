# 🦀 Auth API Rust

API RESTful de autenticación enterprise-grade construida con Rust, siguiendo principios SOLID y Clean Architecture.

## Características

- **Arquitectura Limpia**: Separación clara entre dominio, aplicación e infraestructura
- **Seguridad Enterprise**: JWT con access/refresh tokens, Argon2, CORS, CSP/HSTS
- **Base de Datos**: PostgreSQL con SQLx y migraciones
- **Documentación**: Swagger UI integrado con OpenAPI 3.0
- **Validaciones**: Input validation exhaustiva con `validator`
- **Tipo-seguro**: Aprovecha el sistema de tipos de Rust
- **Escalable**: Preparado para microservicios

## Stack Tecnológico

| Categoría | Tecnología |
|-----------|------------|
| Framework Web | Axum + Tower |
| Base de Datos | PostgreSQL + SQLx |
| Serialización | Serde |
| Validación | Validator |
| Seguridad | Argon2 + JWT |
| Documentación | utoipa + Swagger UI |
| Config | dotenvy + envy |
| Logging | tracing |

## Prerrequisitos

- Rust 1.75+ (edition 2021)
- PostgreSQL 14+
- Cargo

## Instalación

### 1. Clonar el repositorio

```bash
git clone https://github.com/anvorja/auth-api-rust/
cd auth-api-rust
```

### 2. Configurar base de datos

```bash
# Crear base de datos
createdb usuarios_rust_db

# O usando psql
psql -U postgres
CREATE DATABASE usuarios_rust_db;
\q
```

### 3. Configurar variables de entorno

```bash
# Copiar template de configuración
cp .env.example .env

# Editar .env con tus credenciales
nano .env
```


```bash
# Generar secretos seguros
openssl rand -base64 64
```

### 4. Ejecutar migraciones

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
cargo run --bin seed_dev
```

### Ejecutar migraciones para testing
```
bash scripts/setup_test_db.sh
```

### 5. Ejecutar la aplicación

```bash
# Desarrollo
cargo run --bin auth-api-rust
```
### Ejecutar todos los tests ignorados (DB):

```bash
cargo test -- --ignored
```

### Ejecutar un test específico:
```bash
cargo test test_login_user -- --ignored --nocapture
```

### Ejecutar todos los tests de auth_usecase:
```bash
cargo test application::test::auth_usecase_test -- --ignored
```
### Limpiar la DB antes de ejecutar:
```
psql -h localhost -U postgres -d usuarios_rust_db_test -c "TRUNCATE TABLE users RESTART IDENTITY CASCADE;"
```


## Estructura del Proyecto

```
auth-api-rust/
├── .env                   
├── .env.example          
├── Cargo.toml
├── migrations/          
│   └── 0001_create_users.sql
├── src/
│   ├── main.rs           
│   ├── app.rs            # Construcción del router
│   ├── state.rs         
│   ├── error.rs         
│   ├── config/           # Configuración centralizada
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── domain/           # Entidades y lógica de negocio
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── application/     
│   │   ├── mod.rs
│   │   └── auth_usecase.rs
│   ├── infrastructure/  
│   │   ├── mod.rs
│   │   ├── db.rs
│   │   ├── repositories/
│   │   │   └── user_repository_sqlx.rs
│   │   ├── security/
│   │   │   ├── jwt.rs
│   │   │   └── password.rs
│   │   └── http/
│   │       ├── routes.rs
│   │       ├── handlers/
│   │       │   ├── auth.rs
│   │       │   └── health.rs
│   │       └── middleware/
│   │           ├── auth.rs
│   │           ├── security_headers.rs
│   │           └── cors.rs
│   ├── presentation/    
│   │   ├── mod.rs
│   │   └── dto.rs
│   ├── docs/         
│   │   └── openapi.rs
│   └── bin/            
        └── seed_dev.rs
```

## Endpoints

### Autenticación

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| POST | `/api/v1/auth/register` | Registro de nuevo usuario |
| POST | `/api/v1/auth/login` | Inicio de sesión |
| POST | `/api/v1/auth/refresh` | Renovar access token |
| POST | `/api/v1/auth/logout` | Cerrar sesión |

### Salud y Documentación

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/swagger-ui` | Documentación Swagger UI |

## Seguridad

### Implementaciones

- **JWT**: Access tokens (15 min) + Refresh tokens (7 días)
- **Cookies HttpOnly**: Protección contra XSS
- **Argon2**: Hashing de passwords (resistente a GPU/ASIC)
- **CORS**: Configuración restrictiva
- **CSP/HSTS**: Headers de seguridad
- **SQL Injection**: Prevención con bind parameters
- **Input Validation**: Validación exhaustiva con `validator`
- **Rate Limiting**: Protección contra ataques de fuerza bruta

### Configuración de Seguridad

```env
# JWT
JWT_ACCESS_SECRET=<64-char-secret>
JWT_REFRESH_SECRET=<64-char-secret>
JWT_COOKIE_SECURE=true          # HTTPS en producción
JWT_COOKIE_HTTP_ONLY=true       # Protección XSS
JWT_COOKIE_SAME_SITE=Strict     # Protección CSRF

# CORS
CORS_ALLOWED_ORIGINS=https://mydomain.com
CORS_ALLOW_CREDENTIALS=true

# Argon2
ARGON2_MEMORY_COST=65536        # 64 MB
ARGON2_TIME_COST=3              # Iteraciones
ARGON2_PARALLELISM=4            # Threads
```

## Testing

```bash
# Ejecutar tests
cargo test

# Con coverage
cargo tarpaulin --out Html

# Tests de integración
cargo test --test '*'
```

## Seed de Datos

```bash
# Crear usuarios de prueba
cargo run --bin seed_dev
```

## Swagger UI

Una vez la aplicación esté corriendo:

```
http://localhost:9090/api/v1/swagger-ui/
```

## Arquitectura

### Principios SOLID

| Principio | Implementación |
|-----------|----------------|
| **SRP** | Cada módulo tiene una responsabilidad única |
| **OCP** | Extensible sin modificar código existente |
| **LSP** | Traits para abstracciones (repositorios) |
| **ISP** | Interfaces segregadas y específicas |
| **DIP** | Use cases dependen de traits, no implementaciones |

### Clean Architecture

```
┌─────────────────────────────────────────┐
│  Infrastructure (HTTP, DB, JWT, etc)    │ <- Detalles técnicos
├─────────────────────────────────────────┤
│  Presentation (DTOs, Validations)       │ <- Contratos HTTP
├─────────────────────────────────────────┤
│  Application (Use Cases)                │ <- Lógica de aplicación
├─────────────────────────────────────────┤
│  Domain (Entities, Business Rules)      │ <- Núcleo del negocio
└─────────────────────────────────────────┘
```

**Regla de dependencia**: Las capas internas NO conocen las externas.
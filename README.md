# 🦀 Auth API Rust

API RESTful de autenticación enterprise-grade construida con Rust, siguiendo principios SOLID, Clean Architecture y arquitectura de microservicios stateless.

## Características

- **Arquitectura Limpia**: Separación clara entre dominio, aplicación e infraestructura
- **Microservicio Stateless**: Bearer Token con Refresh Strategy para escalabilidad horizontal
- **Seguridad Enterprise**: JWT (access/refresh tokens), Argon2id, CORS por entorno, CSP/HSTS
- **Base de Datos**: PostgreSQL con SQLx y migraciones versionadas
- **Documentación**: Swagger UI integrado con OpenAPI 3.0 y autenticación Bearer
- **Validaciones**: Input validation exhaustiva con `validator`
- **Tipo-seguro**: Aprovecha el sistema de tipos de Rust para prevenir errores
- **Testing: Suite**: completa de tests unitarios e integración con base de datos
- **12-Factor App**: Configuración completa por variables de entorno

## Stack Tecnológico

| Categoría | Tecnología | Versión |
|-----------|------------|---------|
| **Framework Web** | Axum + Tower | 0.8.0 |
| **Base de Datos** | PostgreSQL + SQLx | 0.8.6 |
| **Serialización** | Serde | 1.0 |
| **Validación** | Validator | 0.20.0 |
| **Seguridad** | Argon2id + JWT | 0.5 / 9 |
| **Documentación** | utoipa + Swagger UI | 5.4.0 |
| **Configuración** | dotenvy + envy | 0.15 |
| **Logging** | tracing | 0.1 |

## Prerrequisitos

- **Rust** 1.75+ (edition 2024)
- **PostgreSQL** 14+
- **Cargo** (incluido con Rust)
- **SQLx CLI** (para migraciones)


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

# Crear base de datos de testing
createdb usuarios_rust_db_test

# O usando psql
psql -U postgres
CREATE DATABASE usuarios_rust_db;
CREATE DATABASE usuarios_rust_db_test;
\q
```

### 3. Configurar variables de entorno

```bash
# Copiar template de configuración y editarlo con sus respectivos valores
cp .env.example .env
```

**Configuración mínima requerida:**

```bash
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=9090

# Database
DATABASE_URL=postgresql://postgres:password@host:port/usuarios_rust_db

JWT_SECRET=
JWT_ACCESS_TOKEN_EXPIRY=
JWT_REFRESH_TOKEN_EXPIRY=

# CORS - Desarrollo
ALLOWED_ORIGINS=http://localhost:9090,http://localhost:3000

# Environment
ENVIRONMENT=
RUST_LOG=
```

### 4. Instalar SQLx CL y ejecutar migraciones

```bash
# Instalar SQLx CLI (si no está instalado)
cargo install sqlx-cli --no-default-features --features postgres

# Ejecutar migraciones en base de datos principal
sqlx migrate run
```

### Ejecutar migraciones para testing
```
bash scripts/setup_test_db.sh
```

```
# 1. Setup inicial (Una sola vez por máquina de desarrollo)
bash scripts/setup_test_db.sh

# 2. Antes de correr tests (Antes de cada test suite run)
bash scripts/clean_test_db.sh

# 3. Correr tests (cada test crea sus datos)
cargo test -- --ignored

# Resultado: DB limpia al final
```
ó manualmente (para debugging):
```
DATABASE_URL="postgresql://postgres:password@host:port/usuarios_rust_db_test" sqlx migrate run
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
│   ├── app.rs            
│   ├── state.rs         
│   ├── error.rs         
│   ├── config/           
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── domain/           
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

### Health y Documentación

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/swagger-ui` | Documentación Swagger UI |

## Seguridad

### Implementaciones

- **JWT**: Access tokens (15 min) + Refresh tokens (7 días)
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
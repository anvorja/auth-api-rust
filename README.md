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

| Categoría         | Tecnología          | Versión |
|-------------------|---------------------|---------|
| **Framework Web** | Axum + Tower        | 0.8.0   |
| **Base de Datos** | PostgreSQL + SQLx   | 0.8.6   |
| **Serialización** | Serde               | 1.0     |
| **Validación**    | Validator           | 0.20.0  |
| **Seguridad**     | Argon2id + JWT      | 0.5 / 9 |
| **Documentación** | utoipa + Swagger UI | 5.4.0   |
| **Configuración** | dotenvy + envy      | 0.15    |
| **Logging**       | tracing             | 0.1     |

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
sudo -u postgres psql 
```

```bash
# desarrollo
CREATE DATABASE usuarios_rust_db; 
# testing
CREATE DATABASE usuarios_rust_db_test;
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

```bash
# =============================================================================
# NOTAS DE CONFIGURACIÓN
# =============================================================================
#
# SERVER_HOST:
#   - Development/Test: 127.0.0.1 (solo localhost)
#   - Production: 0.0.0.0 (todas las interfaces - REQUERIDO en Render)
#
# ENVIRONMENT:
#   - development: CORS permisivo, CSP relajado para Swagger
#   - production: CORS restrictivo, CSP estricto, HSTS habilitado
#   - test: Configuración optimizada para tests
#
# ALLOWED_ORIGINS:
#   - Development: Permite HTTP localhost (para dev)
#   - Production: SOLO HTTPS (seguridad)
#
# JWT_SECRET:
#   - Mínimo 32 caracteres
#   - En producción: usar secrets manager o Render env vars
#
# DATABASE_URL:
#   - Development: localhost
#   - Production: Railway/Render Postgres
#   - Test: base de datos separada (usuarios_rust_db_test)
#
# =============================================================================
```

### 4. Instalar SQLx CL y ejecutar migraciones

```bash
# Instalar SQLx CLI (si no está instalado)
cargo install sqlx-cli --no-default-features --features postgres

# ó instalar herramientas de desarrollo 
make install-tools
```

```bash
# Ejecutar migraciones en base de datos principal
sqlx migrate run

# o comandos Makefile
make migrate
```

### Ejecutar migraciones para testing
```
bash scripts/setup_test_db.sh
```

ó manualmente (para debugging):
```
DATABASE_URL="postgresql://postgres:password@host:port/usuarios_rust_db_test" sqlx migrate run
```

### 5. Poblar base de datos con datos de prueba (opcional)
```bash
# Crear usuarios de ejemplo para desarrollo
make seed
```

### 6. Ejecutar la aplicación

```bash
make run
```

ó
```bash
cargo run --bin auth-api-rust
```
#### Desarrollo con auto-reload (cargo-watch)
```bash
# Si hay cambios en .env toca reiniciar aplicación
make dev
```

### 7. Build

#### Modo release (Compilar para producción)

```bash
# Build optimizado
make build-release
```

```bash
# Ejecutar release
./target/release/auth-api-rust
```

**La API estará disponible en:** `http://localhost:9090`

---

## Documentación

### Swagger UI

Una vez la aplicación esté corriendo, accede a la documentación interactiva:

```
http://localhost:9090/api/v1/swagger-ui/
```

### OpenAPI JSON

Especificación OpenAPI disponible en:

```
http://localhost:9090/api/v1/openapi.json
```

---

## Endpoints

### Autenticación

| Método | Endpoint                | Descripción                            | Autenticación |
|--------|-------------------------|----------------------------------------|---------------|
| POST   | `/api/v1/auth/register` | Registro de nuevo usuario              | ❌             |
| POST   | `/api/v1/auth/login`    | Inicio de sesión                       | ❌             |
| POST   | `/api/v1/auth/refresh`  | Renovar access token con refresh token | ❌             |
| POST   | `/api/v1/auth/logout`   | Cerrar sesión (client-side)            | ❌             |

### Usuarios (Protegidos)

| Método | Endpoint                        | Descripción                            | Autenticación |
|--------|---------------------------------|----------------------------------------|---------------|
| GET    | `/api/v1/users/profile`         | Obtener perfil del usuario autenticado | 🔒            |
| POST   | `/api/v1/users/change-password` | Cambiar contraseña                     | 🔒            |

### Health y monitoreo

| Método | Endpoint         | Descripción                  | Autenticación |
|--------|------------------|------------------------------|---------------|
| GET    | `/api/v1/health` | Health check (DB + servicio) | ❌             |


#### Validaciones de Password

- Mínimo 8 caracteres
- Al menos una mayúscula
- Al menos una minúscula
- Al menos un número
- Máximo 128 caracteres

---

## Seguridad

### Arquitectura de Autenticación

Esta API usa **Bearer Token con Refresh Strategy** - arquitectura stateless moderna para microservicios.


### Implementaciones de Seguridad

| Feature              | Implementación                     | Propósito                        |
|----------------------|------------------------------------|----------------------------------|
| **JWT**              | Access (15 min) + Refresh (7 días) | Autenticación stateless          |
| **Password Hashing** | Argon2id (19 MiB, 2 iterations)    | Resistente a GPU/ASIC            |
| **SQL Injection**    | SQLx bind parameters               | Prevención automática            |
| **XSS**              | Input sanitization + validation    | Prevención de scripts maliciosos |
| **CSRF**             | Bearer tokens (no cookies)         | Inmune por diseño                |
| **CORS**             | Configuración por entorno          | HTTP (dev) / HTTPS (prod)        |
| **Security Headers** | CSP, HSTS, X-Frame-Options         | Defensa en profundidad           |
| **Input Validation** | validator crate                    | Validación exhaustiva            |


### Configuración de Seguridad

#### Desarrollo (`.env`)

```bash
ENVIRONMENT=development
ALLOWED_ORIGINS=http://localhost:9090,http://localhost:3000
JWT_ACCESS_TOKEN_EXPIRY=900      # 15 minutos
JWT_REFRESH_TOKEN_EXPIRY=604800  # 7 días
```

#### Producción (variables de entorno)

```bash
ENVIRONMENT=production
ALLOWED_ORIGINS=https://api.midominio.com,https://app.midominio.com
JWT_SECRET=${JWT_SECRET_FROM_VAULT}  # Desde un secrets manager
JWT_ACCESS_TOKEN_EXPIRY=900
JWT_REFRESH_TOKEN_EXPIRY=604800
```


## Testing

### Ejecutar tests

```bash
# Tests rápidos (sin base de datos)
cargo test

# Todos los tests (rápidos + DB)
make test-all
```

### Limpiar la DB antes de ejecutar:
```
make test-db-clean
```
### Coverage

```bash
# Generar reporte HTML
make coverage-all
```

### Otros comandos

```bash
# Verificar compilación sin ejecutar
cargo check

# Formatear código
cargo fmt

# ó 
make format

# Linter (clippy exigente: ve warnings como errores)
cargo clippy -- -D warnings

# Ver estado de migraciones
sqlx migrate info

# Crear nueva migración
sqlx migrate add nombre_descriptivo

# Revertir última migración
sqlx migrate revert

# Reset completo (desarrollo)
dropdb usuarios_rust_db && createdb usuarios_rust_db && sqlx migrate run
```


## Estructura del Proyecto

```
auth-api-rust/
├── .env                   
├── .env.example          
├── Cargo.toml
├── Makefile
├── README.md 
├── migrations/          
│   └── 20251223_create_users.sql
└── src/
    ├── main.rs           
    ├── app.rs   
    ├── error.rs    
    │
    ├── test/
    │   ├── mod.rs
    │   ├── app_test.rs
    │   └── error_test.rs                
    │     
    ├── config/           
    │   ├── mod.rs
    │   └── settings.rs
    │   └── test/
    │       ├── mod.rs
    │       └── settings_test.rs    
    │    
    ├── domain/           
    │   ├── mod.rs
    │   └── user.rs
    │   └── test/
    │       ├── mod.rs
    │       └── user_test.rs    
    │        
    ├── application/     
    │   ├── mod.rs
    │   └── auth_usecase.rs
    │   └── test/
    │       ├── mod.rs
    │       └── auth_usecase_test.rs    
    │        
    ├── infrastructure/  
    │   ├── mod.rs
    │   ├── db.rs
    │   │    
    │   ├── test/                  # Tests de infraestructura base
    │   │   ├── mod.rs
    │   │   └── db_test.rs    
    │   │     
    │   ├── repositories/
    │   │   ├── mod.rs
    │   │   └── user_repository_sqlx.rs
    │   │   └── test/
    │   │       ├── mod.rs
    │   │       └── user_repository_sqlx_test.rs    
    │   │    
    │   ├── security/       
    │   │   ├── mod.rs
    │   │   ├── jwt.rs          
    │   │   └── password.rs   
    │   │   └── test/
    │   │       ├── mod.rs
    │   │       ├── jwt_test.rs
    │   │       └── password_test.rs     
    │   │     
    │   └── http/               
    │       ├── mod.rs
    │       ├── routes.rs
    │       │
    │       ├── test/
    │       │   ├── mod.rs
    │       │   └── routes_test.rs
    │       │       
    │       ├── handlers/      
    │       │   ├── mod.rs
    │       │   ├── auth.rs
    │       │   ├── user.rs
    │       │   └── health.rs
    │       │
    │       └── middleware/
    │           ├── mod.rs
    │           ├── auth.rs
    │           ├── cors.rs
    │           └── security_headers.rs
    │           └── test/
    │               ├── mod.rs
    │               ├── auth_test.rs
    │               ├── cors_test.rs
    │               ├── security_headers_test.rs
    │               └── test_helpers.rs
    │    
    ├── presentation/    
    │   ├── mod.rs
    │   └── dto.rs
    │   └── test/
    │       ├── mod.rs
    │       └── dto_test.rs
    │     
    └── bin/            
        └── seed_dev.rs
```

## Arquitectura

### Principios SOLID

| Principio | Implementación                                    |
|-----------|---------------------------------------------------|
| **SRP**   | Cada módulo tiene una responsabilidad única       |
| **OCP**   | Extensible sin modificar código existente         |
| **LSP**   | Traits para abstracciones (repositorios)          |
| **ISP**   | Interfaces segregadas y específicas               |
| **DIP**   | Use cases dependen de traits, no implementaciones |

### Clean Architecture + DDD

```
┌────────────────────────────────────────────────┐
│  Infrastructure (HTTP, DB, JWT, etc)           │ ← Detalles técnicos
├────────────────────────────────────────────────┤   (reemplazables)
│  Presentation (DTOs, Validations)              │ ← Contratos HTTP
├────────────────────────────────────────────────┤
│  Application (Use Cases)                       │ ← Lógica de aplicación
├────────────────────────────────────────────────┤
│  Domain (Entities, Business Rules)             │ ← Núcleo del negocio
└────────────────────────────────────────────────┘   (independiente)
```

**Regla de dependencia**: Las capas internas NO conocen las externas.
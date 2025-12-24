# Migraciones de Base de Datos

Este directorio contiene las migraciones de SQLx para la base de datos PostgreSQL.

## Requisitos

- PostgreSQL 12 o superior
- SQLx CLI instalado: `cargo install sqlx-cli --no-default-features --features postgres`

## Configuración Inicial

1. Crear la base de datos:
```bash
createdb usuarios_rust_db
```

O con psql:
```sql
CREATE DATABASE usuarios_rust_db;
```

2. Configurar la URL de la base de datos en `.env`:
```
DATABASE_URL=postgresql://postgres:superapostgres@localhost:5432/usuarios_rust_db
```

## Ejecutar Migraciones

### Aplicar todas las migraciones pendientes:
```bash
sqlx migrate run
```

### Poblar base de datos con datos seed
```
cargo run --bin seed_dev
```

### Correr aplicación
```
cargo run --bin auth-api-rust
```

### Revertir la última migración:
```bash
sqlx migrate revert
```

### Ver el estado de las migraciones:
```bash
sqlx migrate info
```

## Crear Nueva Migración

```bash
sqlx migrate add <nombre_descriptivo>
```

Ejemplo:
```bash
sqlx migrate add add_user_roles
```

## Migraciones Existentes

### 20251223_create_users.sql
Crea la tabla principal `users` con:
- UUID como primary key
- Username único (3-30 caracteres, lowercase)
- Email único
- Password hash (Argon2id)
- Timestamps automáticos (created_at, updated_at)
- Constraints de validación
- Índices para performance
- Trigger para actualizar `updated_at`

## Estructura de la Tabla Users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Constraints

- `users_username_key`: Username debe ser único
- `users_email_key`: Email debe ser único
- `users_username_length`: Username entre 3 y 30 caracteres
- `users_email_length`: Email entre 5 y 255 caracteres
- `users_username_format`: Username solo lowercase, alfanumérico + underscore, empieza con letra
- `users_password_hash_not_empty`: Password hash no puede estar vacío

## Índices

- `idx_users_username`: Búsqueda rápida por username
- `idx_users_email`: Búsqueda rápida por email
- `idx_users_created_at`: Ordenamiento por fecha de creación

## Verificación

Para verificar que las migraciones se aplicaron correctamente:

```bash
psql -U postgres -d usuarios_rust_db -c "\dt"
psql -U postgres -d usuarios_rust_db -c "\d users"
```

## Rollback

Si necesitas revertir todas las migraciones:

```bash
sqlx migrate revert
sqlx migrate revert
# ... repetir hasta revertir todas
```

O eliminar y recrear la base de datos:

```bash
dropdb usuarios_rust_db
createdb usuarios_rust_db
sqlx migrate run
```
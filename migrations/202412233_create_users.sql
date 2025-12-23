-- Migración: Crear tabla users
-- Descripción: Tabla principal de usuarios con autenticación

-- Habilitar extensión UUID (si no está habilitada)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Crear tabla users
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                       username VARCHAR(30) NOT NULL,
                       email VARCHAR(255) NOT NULL,
                       first_name VARCHAR(100) NOT NULL,
                       last_name VARCHAR(100) NOT NULL,
                       password_hash TEXT NOT NULL,
                       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
                       CONSTRAINT users_username_key UNIQUE (username),
                       CONSTRAINT users_email_key UNIQUE (email),
                       CONSTRAINT users_username_length CHECK (LENGTH(username) >= 3 AND LENGTH(username) <= 30),
                       CONSTRAINT users_email_length CHECK (LENGTH(email) >= 5 AND LENGTH(email) <= 255),
                       CONSTRAINT users_first_name_length CHECK (LENGTH(first_name) >= 1 AND LENGTH(first_name) <= 100),
                       CONSTRAINT users_last_name_length CHECK (LENGTH(last_name) >= 1 AND LENGTH(last_name) <= 100),
                       CONSTRAINT users_username_format CHECK (username ~ '^[a-z][a-z0-9_]{2,29}$'),
    CONSTRAINT users_password_hash_not_empty CHECK (LENGTH(password_hash) > 0)
);

-- Índices para performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at DESC);

-- Función para actualizar updated_at automáticamente
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger para updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comentarios en la tabla y columnas (documentación)
COMMENT ON TABLE users IS 'Tabla de usuarios del sistema de autenticación';
COMMENT ON COLUMN users.id IS 'Identificador único UUID';
COMMENT ON COLUMN users.username IS 'Nombre de usuario único (3-30 caracteres, lowercase)';
COMMENT ON COLUMN users.email IS 'Email único del usuario';
COMMENT ON COLUMN users.first_name IS 'Nombre del usuario';
COMMENT ON COLUMN users.last_name IS 'Apellido del usuario';
COMMENT ON COLUMN users.password_hash IS 'Password hasheado con Argon2id';
COMMENT ON COLUMN users.created_at IS 'Fecha de creación del registro';
COMMENT ON COLUMN users.updated_at IS 'Fecha de última actualización (se actualiza automáticamente)';
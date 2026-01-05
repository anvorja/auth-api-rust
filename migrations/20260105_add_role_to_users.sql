-- Agregar columna role a la tabla users
ALTER TABLE users
    ADD COLUMN role VARCHAR(20) NOT NULL DEFAULT 'user';

-- Agregar constraint para validar roles
ALTER TABLE users
    ADD CONSTRAINT users_role_check CHECK (role IN ('user', 'admin'));

-- Índice para búsquedas por rol
CREATE INDEX idx_users_role ON users(role);

-- Comentario en la columna
COMMENT ON COLUMN users.role IS 'Rol del usuario (user o admin)';
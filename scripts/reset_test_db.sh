#!/bin/bash
# scripts/reset_test_db.sh
# Reset completo (útil si migraciones cambiaron)

set -e

DB_NAME="usuarios_rust_db_test"
DB_URL="postgresql://postgres:superapostgres@localhost:5432/$DB_NAME"

echo "⚙ Reseteando base de datos de tests..."

# 1. Drop
dropdb $DB_NAME 2>/dev/null || true

# 2. Create
createdb $DB_NAME

# 3. Migrate
DATABASE_URL=$DB_URL sqlx migrate run

echo "✓ DB de tests reseteada completamente"
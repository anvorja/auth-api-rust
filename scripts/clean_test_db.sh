#!/bin/bash
# scripts/clean_test_db.sh
# Limpia todos los datos de la base de datos de tests

set -e

DB_NAME="usuarios_rust_db_test"

echo "🧹 Limpiando base de datos de tests..."

# Intentar diferentes métodos de autenticación
if psql -h localhost -U postgres -d $DB_NAME -c "TRUNCATE TABLE users RESTART IDENTITY CASCADE;" 2>/dev/null; then
    echo "✅ Base de datos limpiada (método: host localhost)"
elif sudo -u postgres psql -d $DB_NAME -c "TRUNCATE TABLE users RESTART IDENTITY CASCADE;" 2>/dev/null; then
    echo "✅ Base de datos limpiada (método: sudo postgres)"
elif psql -d $DB_NAME -c "TRUNCATE TABLE users RESTART IDENTITY CASCADE;" 2>/dev/null; then
    echo "✅ Base de datos limpiada (método: usuario actual)"
else
    echo "❌ Error: No se pudo limpiar la base de datos"
    echo ""
    echo "Intenta manualmente uno de estos comandos:"
    echo "  1. psql -h localhost -U postgres -d $DB_NAME -c \"TRUNCATE TABLE users RESTART IDENTITY CASCADE;\""
    echo "  2. sudo -u postgres psql -d $DB_NAME -c \"TRUNCATE TABLE users RESTART IDENTITY CASCADE;\""
    echo "  3. psql -d $DB_NAME -c \"TRUNCATE TABLE users RESTART IDENTITY CASCADE;\""
    exit 1
fi

echo ""
echo "🧪 Ahora puedes ejecutar:"
echo "   cargo test -- --ignored"
#!/bin/bash
# scripts/setup_test_db.sh
# Setup simple para base de datos de tests

set -e

DB_NAME="usuarios_rust_db_test"
DB_URL="postgresql://postgres:superapostgres@localhost:5432/$DB_NAME"

echo "⚙ Configurando base de datos de tests..."
echo ""

# 1. Crear DB si no existe
createdb $DB_NAME 2>/dev/null && echo "✓ Base de datos creada" || echo "✓ Base de datos ya existe"
echo ""

# 2. Aplicar migraciones
echo "⚙ Aplicando migraciones..."
export DATABASE_URL=$DB_URL
sqlx migrate run

# 3. Verificar
echo ""
echo "✓ Base de datos de tests lista!"
echo ""
echo "📊 Información:"
echo "   Database: $DB_NAME"
echo "   URL:      $DB_URL"
echo ""
echo " Ejecutar tests:"
echo "   cargo test --ignored"
echo ""
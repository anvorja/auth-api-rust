#!/bin/bash
# Poblar DB de desarrollo (wrapper)

set -e

echo "⚙ Verificando el entorno..."

# Verificar que estamos en desarrollo
if [ "$ENVIRONMENT" = "production" ]; then
    echo "❌ ERROR: No ejecutar seed en producción"
    exit 1
fi

# Ejecutar seed
cargo run --bin seed_dev
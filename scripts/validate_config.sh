#!/bin/bash
# V1
# scripts/validate_config.sh
# Script para validar que la configuración esté correcta antes de ejecutar la app

set -e

echo "🔍 Validating configuration..."

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Función para verificar variable de entorno
check_env_var() {
    local var_name=$1
    local var_value=$(grep "^${var_name}=" .env 2>/dev/null | cut -d '=' -f2-)
    
    if [ -z "$var_value" ]; then
        echo -e "${RED}✗${NC} Missing: $var_name"
        return 1
    else
        echo -e "${GREEN}✓${NC} Found: $var_name"
        return 0
    fi
}

# Función para verificar longitud mínima
check_min_length() {
    local var_name=$1
    local min_length=$2
    local var_value=$(grep "^${var_name}=" .env 2>/dev/null | cut -d '=' -f2-)
    
    if [ ${#var_value} -lt $min_length ]; then
        echo -e "${RED}✗${NC} $var_name is too short (minimum: $min_length characters)"
        return 1
    fi
    return 0
}

# Función para verificar que no contenga valores por defecto
check_not_default() {
    local var_name=$1
    local var_value=$(grep "^${var_name}=" .env 2>/dev/null | cut -d '=' -f2-)
    
    if [[ "$var_value" == *"CHANGE"* ]] || [[ "$var_value" == *"change"* ]]; then
        echo -e "${YELLOW}⚠${NC}  $var_name contains default value - change before production!"
        return 0
    fi
    return 0
}

# Verificar que existe .env
if [ ! -f .env ]; then
    echo -e "${RED}✗${NC} .env file not found!"
    echo "  Run: cp .env.example .env"
    exit 1
fi

echo ""
echo "📋 Checking required variables..."

# Variables críticas
ERRORS=0

check_env_var "DATABASE_URL" || ERRORS=$((ERRORS + 1))
check_env_var "JWT_ACCESS_SECRET" || ERRORS=$((ERRORS + 1))
check_env_var "JWT_REFRESH_SECRET" || ERRORS=$((ERRORS + 1))
check_env_var "SERVER_PORT" || ERRORS=$((ERRORS + 1))

echo ""
echo "🔐 Checking security settings..."

check_min_length "JWT_ACCESS_SECRET" 32 || ERRORS=$((ERRORS + 1))
check_min_length "JWT_REFRESH_SECRET" 32 || ERRORS=$((ERRORS + 1))

echo ""
echo "⚠️  Checking for default values..."

check_not_default "JWT_ACCESS_SECRET"
check_not_default "JWT_REFRESH_SECRET"
check_not_default "DATABASE_PASSWORD"

echo ""

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✓ Configuration validation passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Configuration validation failed with $ERRORS error(s)${NC}"
    exit 1
fi

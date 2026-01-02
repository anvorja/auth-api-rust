## Makefile - Auth API Rust

- Testing organizado: tests rápidos, con DB, y específicos

- Gestión de base de datos: migraciones, seeding, reset


### Instalación Rápida

```bash
# Configuración inicial del proyecto
make setup
```

```bash
# Instalar herramientas de desarrollo
make install-tools
```

```bash
# Verificar que todo esté correcto
make validate
```

### Comandos Principales

```
# Mostrar ayuda completa
make help

# Compilación
make build          # Compilar en modo desarrollo
make build-release  # Compilar para producción

# Ejecución
make run           # Ejecutar la aplicación
make dev           # Modo desarrollo con auto-reload

# Testing general
make test          # Tests rápidos (sin DB)
make test-db       # Tests con base de datos
make test-all      # Todos los tests (rápidos + DB)
make test-verbose  # Tests con output detallado

# Base de datos
make migrate       # Ejecutar migraciones
make migrate-undo  # Revertir última migración
make seed          # Poblar DB con datos de prueba
make db-reset      # Resetear DB completamente (¡CUIDADO!)

# Calidad de código
make format        # Formatear código
make lint          # Ejecutar clippy (linter)
make fix           # Auto-arreglar problemas
make audit         # Auditar dependencias
```

### Testing Avanzado

```bash
# Mostrar ayuda específica de testing
make test-help
```

### Ejecutar Tests Específicos

```
# Por módulo completo
make test-one MODULE=application::test::auth_usecase_test
make test-one MODULE=domain::test::user_test
make test-one MODULE=infrastructure::security::test::jwt_test

# Por patrón
make test-one PATTERN=password
make test-one PATTERN=jwt
make test-one PATTERN=auth_usecase

# Test individual
make test-one TEST=application::test::auth_usecase_test::test_register_user
```

### Listar Tests Disponibles

```
# Todos los tests
make test-list

# Tests de un módulo específico
make test-list-module MODULE=application::test::auth_usecase_test

# Tests por patrón
make test-list-pattern PATTERN=password
```

### Otros Comandos de Testing

```
# Generar reporte de coverage (requiere cargo-tarpaulin)
make coverage

# Modo watch (auto-reload al cambiar código)
make test-watch
```

### Operaciones con Base de Datos

#### Base de Datos Principal

```
# Crear base de datos
make db-create

# Eliminar base de datos (¡confirmación requerida!)
make db-drop

# Reset completo: drop → create → migrate → seed
make db-reset

# Migraciones
make migrate
make migrate-undo
make migrate-status
```

#### Base de Datos Testing

```
# Setup inicial (solo una vez)
make test-db-setup

# Limpiar datos de tests
make test-db-clean

# Ejecutar tests con DB (limpia automáticamente)
make test-db
```

### Calidad de Código

```
# Formatear código con rustfmt
make format

# Ejecutar clippy (linter de Rust)
make lint

# Auto-arreglar problemas detectables
make fix

# Limpiar archivos de compilación
make clean

# Generar documentación
make doc

# Verificar que el código compila
make check

# Auditar dependencias por vulnerabilidades
make audit

# Actualizar dependencias
make update
```

### Herramientas de Desarrollo

```
# Instalar todas las herramientas recomendadas
make install-tools

# Herramientas incluidas:
# - cargo-watch      (auto-reload)
# - cargo-tarpaulin  (coverage)
# - cargo-audit      (vulnerabilidades)
# - sqlx-cli         (migraciones PostgreSQL)

# Mostrar estructura del proyecto
make tree
```
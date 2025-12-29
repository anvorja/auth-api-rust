# Makefile para Auth API Rust
# Comandos útiles para desarrollo y despliegue

.PHONY: help setup check build run test clean migrate seed validate test-db test-all test-db-clean test-db-setup

# Colores
GREEN  := \033[0;32m
YELLOW := \033[1;33m
RED    := \033[0;31m
BLUE   := \033[0;34m
NC     := \033[0m # No Color

help: ## Mostrar ayuda
	@echo "$(GREEN)Auth API Rust - Available Commands$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort |awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(BLUE)Tip: Use 'make help-test' para ayuda de testing$(NC)"

setup: ## Configuración inicial del proyecto
	@echo "$(GREEN)Setting up project...$(NC)"
	@if [ ! -f .env ]; then \
		echo "$(YELLOW)Creating .env from template...$(NC)"; \
		cp .env.example .env; \
		echo "$(RED)⚠️  Please edit .env with your actual values$(NC)"; \
	else \
		echo "$(GREEN)✓ .env already exists$(NC)"; \
	fi
	@echo "$(GREEN)Installing SQLx CLI...$(NC)"
	cargo install sqlx-cli --no-default-features --features postgres
	@echo "$(GREEN)✓ Setup complete$(NC)"

validate: ## Validar configuración
	@echo "$(GREEN)Validating configuration...$(NC)"
	@./scripts/validate_config.sh

check: ## Verificar que el código compila
	@echo "$(GREEN)Checking code...$(NC)"
	@cargo check --all-features

build: ## Compilar el proyecto
	@echo "$(GREEN)Building project...$(NC)"
	@cargo build

build-release: ## Compilar para producción (optimizado)
	@echo "$(GREEN)Building release version...$(NC)"
	@cargo build --release

run: validate ## Ejecutar la aplicación
	@echo "$(GREEN)Running application...$(NC)"
	@cargo run

dev: ## Ejecutar con auto-reload (requiere cargo-watch)
	@echo "$(GREEN)Running in watch mode...$(NC)"
	@cargo watch -x run

# ============================================================================
# TESTS
# ============================================================================

test: ## Ejecutar tests rápidos (sin DB) -  durante desarrollo
	@echo "$(GREEN)Running fast tests (no DB)...$(NC)"
	@cargo test
	@echo ""
	@echo "$(BLUE)Para tests con DB usar: make test-db$(NC)"

test-verbose: ## Tests con output detallado
	@echo "$(GREEN)Running tests (verbose)...$(NC)"
	@cargo test -- --nocapture --test-threads=1

coverage: ## Generar reporte de coverage (requiere cargo-tarpaulin)
	@echo "$(GREEN)Generating coverage report...$(NC)"
	@cargo tarpaulin --out Html --output-dir coverage --exclude-files 'src/bin/*'

# ============================================================================
# DATABASE OPERATIONS
# ============================================================================

migrate: ## Ejecutar migraciones de base de datos
	@echo "$(GREEN)Running database migrations...$(NC)"
	@sqlx migrate run

migrate-undo: ## Revertir última migración
	@echo "$(YELLOW)Reverting last migration...$(NC)"
	@sqlx migrate revert

migrate-status: ## Ver estado de migraciones
	@sqlx migrate info

seed: ## Poblar base de datos con datos de prueba
	@echo "$(GREEN)Seeding database...$(NC)"
	@cargo run --bin seed_dev

db-create: ## Crear base de datos
	@echo "$(GREEN)Creating database...$(NC)"
	@createdb usuarios_rust_db || echo "$(YELLOW)Database might already exist$(NC)"

db-drop: ## Eliminar base de datos (¡CUIDADO!)
	@echo "$(RED)Dropping database...$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		dropdb usuarios_rust_db; \
		echo "$(GREEN)Database dropped$(NC)"; \
	fi

db-reset: db-drop db-create migrate seed ## Resetear base de datos completamente

# ============================================================================
# TESTS CON BASE DE DATOS
# ============================================================================

# Variables para testing
TEST_DB_NAME := usuarios_rust_db_test
DB_PASSWORD := superapostgres

test-db-setup: ## Configurar base de datos de tests (una sola vez)
	@echo "$(GREEN)Setting up test database...$(NC)"
	@echo "$(YELLOW)Creating test database if not exists...$(NC)"
	@PGPASSWORD=$(DB_PASSWORD) psql -h localhost -U postgres -d postgres -c "CREATE DATABASE $(TEST_DB_NAME);" 2>/dev/null || echo "$(YELLOW)Database already exists$(NC)"
	@echo "$(GREEN)Running migrations on test database...$(NC)"
	@DATABASE_URL="postgresql://postgres:$(DB_PASSWORD)@localhost:5432/$(TEST_DB_NAME)" sqlx migrate run
	@echo "$(GREEN)Test database ready$(NC)"

test-db-clean: ## Limpiar base de datos de tests
	@echo "$(YELLOW)Cleaning test database...$(NC)"
	@PGPASSWORD=$(DB_PASSWORD) psql -h localhost -U postgres -d $(TEST_DB_NAME) -c "TRUNCATE TABLE users RESTART IDENTITY CASCADE;" 2>/dev/null || true
	@echo "$(GREEN)Test database cleaned$(NC)"

test-db: test-db-clean ## Ejecutar tests con DB (limpia DB antes)
	@echo "$(GREEN)Running database tests...$(NC)"
	@cargo test -- --ignored --test-threads=1
	@echo "$(GREEN)✓ Database tests completed$(NC)"

test-all: ## Ejecutar TODOS los tests (rápidos + DB)
	@echo "$(GREEN)Running all tests...$(NC)"
	@echo ""
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(BLUE)  Part 1: Fast tests (no DB)$(NC)"
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@cargo test
	@echo ""
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@echo "$(BLUE)  Part 2: Database tests$(NC)"
	@echo "$(BLUE)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(NC)"
	@$(MAKE) test-db-clean > /dev/null 2>&1
	@cargo test -- --ignored --test-threads=1
	@echo ""
	@echo "$(GREEN)✓ All tests completed successfully!$(NC)"

test-watch: ## Ejecutar tests en modo watch
	@echo "$(GREEN)Running tests in watch mode...$(NC)"
	@cargo watch -x test

test-one: ## Ejecutar un test específico (use TEST=nombre_test)
	@if [ -z "$(TEST)" ]; then \
		echo "$(RED)Error: Specify test name with TEST=nombre_test$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)Running test: $(TEST)$(NC)"
	@cargo test $(TEST) -- --nocapture

test-help: ## Ayuda de comandos de testing
	@echo "$(GREEN)════════════════════════════════════════$(NC)"
	@echo "$(GREEN)    Guía de Testing - Auth API Rust    $(NC)"
	@echo "$(GREEN)════════════════════════════════════════$(NC)"
	@echo ""
	@echo "$(YELLOW)Comandos Principales:$(NC)"
	@echo ""
	@echo "  $(BLUE)make test$(NC)"
	@echo "    → Tests rápidos (sin DB)"
	@echo "    → Ideal durante desarrollo"
	@echo "    → ~73 tests en <5 segundos"
	@echo ""
	@echo "  $(BLUE)make test-db$(NC)"
	@echo "    → Tests con base de datos"
	@echo "    → Requiere PostgreSQL corriendo"
	@echo "    → ~10 tests de integración"
	@echo "    → Limpia DB automáticamente antes"
	@echo ""
	@echo "  $(BLUE)make test-all$(NC)"
	@echo "    → TODOS los tests (rápidos + DB)"
	@echo "    → ~83 tests totales"
	@echo "    → Usa antes de commits importantes"
	@echo ""
	@echo "$(YELLOW)Comandos Auxiliares:$(NC)"
	@echo ""
	@echo "  $(BLUE)make test-db-setup$(NC)"
	@echo "    → Setup inicial DB de tests (una sola vez)"
	@echo ""
	@echo "  $(BLUE)make test-db-clean$(NC)"
	@echo "    → Limpiar DB de tests manualmente"
	@echo ""
	@echo "  $(BLUE)make test-one TEST=nombre$(NC)"
	@echo "    → Ejecutar un test específico"
	@echo "    → Ejemplo: make test-one TEST=test_register_user"
	@echo ""
	@echo "  $(BLUE)make test-watch$(NC)"
	@echo "    → Tests en modo watch (auto-reload)"
	@echo ""
	@echo "  $(BLUE)make coverage$(NC)"
	@echo "    → Generar reporte de coverage HTML"
	@echo ""
	@echo "$(YELLOW)Workflow Recomendado:$(NC)"
	@echo ""
	@echo "  1. $(GREEN)make test-db-setup$(NC)     (primera vez solamente)"
	@echo "  2. $(GREEN)make test$(NC)              (durante desarrollo)"
	@echo "  3. $(GREEN)make test-db$(NC)           (antes de commit)"
	@echo "  4. $(GREEN)make test-all$(NC)          (antes de push/PR)"
	@echo ""
	@echo "$(YELLOW)Troubleshooting:$(NC)"
	@echo ""
	@echo "  • Tests con DB fallan?"
	@echo "    → Verifica PostgreSQL: pg_isready"
	@echo "    → Recrea DB: make test-db-setup"
	@echo ""
	@echo "  • Tests lentos?"
	@echo "    → Usa make test (sin DB)"
	@echo "    → Tests con DB son más lentos (esperado)"
	@echo ""
	@echo "$(GREEN)════════════════════════════════════════$(NC)"

# ============================================================================
# CODE QUALITY
# ============================================================================

clean: ## Limpiar archivos de compilación
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	@cargo clean

format: ## Formatear código con rustfmt
	@echo "$(GREEN)Formatting code...$(NC)"
	@cargo fmt --all

lint: ## Ejecutar clippy (linter de Rust)
	@echo "$(GREEN)Running clippy...$(NC)"
	@cargo clippy --all-features -- -D warnings

fix: ## Auto-arreglar problemas de clippy y formato
	@echo "$(GREEN)Auto-fixing issues...$(NC)"
	@cargo clippy --fix --allow-dirty --allow-staged
	@cargo fmt --all

doc: ## Generar documentación
	@echo "$(GREEN)Generating documentation...$(NC)"
	@cargo doc --no-deps --open

# ============================================================================
# TOOLS
# ============================================================================

install-tools: ## Instalar herramientas de desarrollo
	@echo "$(GREEN)Installing development tools...$(NC)"
	@cargo install cargo-watch
	@cargo install cargo-tarpaulin
	@cargo install cargo-audit
	@cargo install sqlx-cli --no-default-features --features postgres
	@echo "$(GREEN)✓ Tools installed$(NC)"

audit: ## Auditar dependencias por vulnerabilidades
	@echo "$(GREEN)Auditing dependencies...$(NC)"
	@cargo audit

update: ## Actualizar dependencias
	@echo "$(GREEN)Updating dependencies...$(NC)"
	@cargo update

tree: ## Mostrar estructura del proyecto
	@echo "$(GREEN)Project structure:$(NC)"
	@tree -I 'target|node_modules' -L 3

.DEFAULT_GOAL := help
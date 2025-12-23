# V1
# Makefile para Auth API Rust
# Comandos útiles para desarrollo y despliegue

.PHONY: help setup check build run test clean migrate seed validate

# Colores
GREEN  := \033[0;32m
YELLOW := \033[1;33m
RED    := \033[0;31m
NC     := \033[0m # No Color

help: ## Mostrar esta ayuda
	@echo "$(GREEN)Auth API Rust - Available Commands$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}'

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

test: ## Ejecutar todos los tests
	@echo "$(GREEN)Running tests...$(NC)"
	@cargo test

test-verbose: ## Ejecutar tests con output detallado
	@echo "$(GREEN)Running tests (verbose)...$(NC)"
	@cargo test -- --nocapture --test-threads=1

coverage: ## Generar reporte de coverage (requiere cargo-tarpaulin)
	@echo "$(GREEN)Generating coverage report...$(NC)"
	@cargo tarpaulin --out Html --output-dir coverage

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

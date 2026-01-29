# AgentTrace Makefile
# ==================

.PHONY: all build test clean dev setup help

# Default target
all: build

# ==================
# Setup
# ==================

setup: ## Install all dependencies
	@echo "📦 Setting up development environment..."
	@cd crates/agenttrace-core && cargo fetch
	@cd sdks/python && poetry install
	@cd dashboard && pnpm install
	@echo "✅ Setup complete!"

setup-db: ## Initialize database with migrations
	@echo "🗄️  Setting up database..."
	docker-compose up -d timescaledb redis
	@sleep 5  # Wait for DB to be ready
	PGPASSWORD=agenttrace_dev psql -h localhost -U agenttrace -d agenttrace -f migrations/001_initial_schema.sql
	@echo "✅ Database ready!"

# ==================
# Build
# ==================

build: build-rust build-python build-dashboard ## Build all components
	@echo "✅ All components built!"

build-rust: ## Build Rust collector
	@echo "🦀 Building Rust collector..."
	cd crates/agenttrace-core && cargo build --release

build-python: ## Build Python SDK
	@echo "🐍 Building Python SDK..."
	cd sdks/python && poetry build

build-dashboard: ## Build web dashboard
	@echo "⚛️  Building web dashboard..."
	cd dashboard && pnpm build

# ==================
# Development
# ==================

dev: ## Start all services in development mode
	@echo "🚀 Starting development environment..."
	docker-compose up -d
	@echo "Starting collector..."
	cd crates/agenttrace-core && cargo run -- serve &
	@echo "Starting dashboard..."
	cd dashboard && pnpm dev &
	@echo ""
	@echo "✅ Development environment running!"
	@echo "   - Collector: http://localhost:8080"
	@echo "   - Dashboard: http://localhost:3000"
	@echo "   - TimescaleDB: localhost:5432"
	@echo "   - Redis: localhost:6379"

dev-collector: ## Run collector in dev mode with hot reload
	cd crates/agenttrace-core && cargo watch -x 'run -- serve'

dev-dashboard: ## Run dashboard in dev mode
	cd dashboard && pnpm dev

dev-tui: ## Run TUI dashboard
	cd crates/agenttrace-core && cargo run -- dashboard

# ==================
# Testing
# ==================

test: test-rust test-python test-dashboard ## Run all tests
	@echo "✅ All tests passed!"

test-rust: ## Run Rust tests
	@echo "🦀 Running Rust tests..."
	cd crates/agenttrace-core && cargo test

test-python: ## Run Python tests
	@echo "🐍 Running Python tests..."
	cd sdks/python && poetry run pytest -v

test-dashboard: ## Run dashboard tests
	@echo "⚛️  Running dashboard tests..."
	cd dashboard && pnpm test

test-integration: ## Run integration tests (requires Docker)
	@echo "🔗 Running integration tests..."
	./scripts/integration-tests.sh

# ==================
# Linting & Formatting
# ==================

lint: lint-rust lint-python lint-dashboard ## Lint all code
	@echo "✅ All linting passed!"

lint-rust: ## Lint Rust code
	cd crates/agenttrace-core && cargo clippy -- -D warnings

lint-python: ## Lint Python code
	cd sdks/python && poetry run ruff check .

lint-dashboard: ## Lint dashboard code
	cd dashboard && pnpm lint

fmt: fmt-rust fmt-python fmt-dashboard ## Format all code
	@echo "✅ All code formatted!"

fmt-rust: ## Format Rust code
	cd crates/agenttrace-core && cargo fmt

fmt-python: ## Format Python code
	cd sdks/python && poetry run ruff format .

fmt-dashboard: ## Format dashboard code
	cd dashboard && pnpm format

# ==================
# Database
# ==================

db-up: ## Start database containers
	docker-compose up -d timescaledb redis

db-down: ## Stop database containers
	docker-compose down

db-reset: ## Reset database (WARNING: deletes all data)
	docker-compose down -v
	docker-compose up -d timescaledb redis
	@sleep 5
	PGPASSWORD=agenttrace_dev psql -h localhost -U agenttrace -d agenttrace -f migrations/001_initial_schema.sql

db-shell: ## Open database shell
	PGPASSWORD=agenttrace_dev psql -h localhost -U agenttrace -d agenttrace

db-seed: ## Seed database with sample data
	@echo "🌱 Seeding database with sample data..."
	cd crates/agenttrace-core && cargo run -- db seed

# ==================
# Release
# ==================

release: ## Create a release build
	@echo "📦 Creating release build..."
	cd crates/agenttrace-core && cargo build --release
	cd sdks/python && poetry build
	cd dashboard && pnpm build
	@echo "✅ Release build complete!"

release-docker: ## Build Docker images
	@echo "🐳 Building Docker images..."
	docker build -t agenttrace:latest .
	docker build -t agenttrace-dashboard:latest ./dashboard

# ==================
# Documentation
# ==================

docs: ## Build documentation
	@echo "📚 Building documentation..."
	cd docs && mdbook build

docs-serve: ## Serve documentation locally
	cd docs && mdbook serve

# ==================
# Utilities
# ==================

clean: ## Clean all build artifacts
	@echo "🧹 Cleaning build artifacts..."
	cd crates/agenttrace-core && cargo clean
	cd sdks/python && rm -rf dist/ .pytest_cache/
	cd dashboard && rm -rf dist/ node_modules/.cache/
	@echo "✅ Clean complete!"

logs: ## Show logs from all containers
	docker-compose logs -f

logs-collector: ## Show collector logs
	cd crates/agenttrace-core && RUST_LOG=debug cargo run -- serve

stats: ## Show database statistics
	@echo "📊 Database statistics:"
	@PGPASSWORD=agenttrace_dev psql -h localhost -U agenttrace -d agenttrace -c "\
		SELECT 'Traces' as table_name, COUNT(*) as count FROM traces \
		UNION ALL \
		SELECT 'Spans', COUNT(*) FROM spans \
		UNION ALL \
		SELECT 'Alert Rules', COUNT(*) FROM alert_rules;"

# ==================
# Help
# ==================

help: ## Show this help message
	@echo "AgentTrace Development Commands"
	@echo "==============================="
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

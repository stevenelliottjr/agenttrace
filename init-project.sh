#!/bin/bash
# init-project.sh — Initialize the AgentTrace project structure

set -e

echo "🚀 Initializing AgentTrace project structure..."

# Create root directory structure
mkdir -p collector/src/{server,pipeline,storage,models,utils}
mkdir -p cli/src
mkdir -p tui/src
mkdir -p sdk/python/src/agenttrace/{integrations,exporters}
mkdir -p sdk/python/tests
mkdir -p api/src
mkdir -p api/tests
mkdir -p dashboard/src/{app,components,hooks,lib,stores}
mkdir -p dashboard/src/components/{ui,charts,traces,sessions,common}
mkdir -p dashboard/src/app/{sessions,traces,metrics,settings}
mkdir -p proto
mkdir -p migrations
mkdir -p docker
mkdir -p docs
mkdir -p examples
mkdir -p scripts

# Create Rust workspace Cargo.toml
cat > Cargo.toml << 'CARGO'
[workspace]
resolver = "2"
members = [
    "collector",
    "cli",
    "tui",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/yourusername/agenttrace"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
tonic = "0.10"
prost = "0.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4.4", features = ["derive"] }
config = "0.14"
thiserror = "1.0"
anyhow = "1.0"
bytes = "1.5"
CARGO

# Create collector Cargo.toml
cat > collector/Cargo.toml << 'CARGO'
[package]
name = "agenttrace-collector"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "agenttrace-collector"
path = "src/main.rs"

[dependencies]
tokio.workspace = true
tonic.workspace = true
prost.workspace = true
serde.workspace = true
serde_json.workspace = true
sqlx.workspace = true
redis.workspace = true
uuid.workspace = true
chrono.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
config.workspace = true
thiserror.workspace = true
anyhow.workspace = true
bytes.workspace = true
dashmap = "5.5"
parking_lot = "0.12"
metrics = "0.22"
tiktoken-rs = "0.5"

[build-dependencies]
tonic-build = "0.10"
CARGO

# Create CLI Cargo.toml
cat > cli/Cargo.toml << 'CARGO'
[package]
name = "agenttrace-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "agenttrace"
path = "src/main.rs"

[dependencies]
tokio.workspace = true
clap.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
reqwest = { version = "0.11", features = ["json"] }
tabled = "0.15"
indicatif = "0.17"
console = "0.15"
dialoguer = "0.11"
CARGO

# Create TUI Cargo.toml
cat > tui/Cargo.toml << 'CARGO'
[package]
name = "agenttrace-tui"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "agenttrace-tui"
path = "src/main.rs"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
ratatui = "0.25"
crossterm = "0.27"
tui-textarea = "0.4"
reqwest = { version = "0.11", features = ["json"] }
CARGO

# Create Python SDK pyproject.toml
cat > sdk/python/pyproject.toml << 'PYPROJECT'
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "agenttrace"
version = "0.1.0"
description = "Observability SDK for AI Agents"
readme = "README.md"
license = "MIT"
requires-python = ">=3.11"
authors = [
    { name = "Your Name", email = "you@example.com" }
]
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
]
dependencies = [
    "grpcio>=1.60.0",
    "grpcio-tools>=1.60.0",
    "protobuf>=4.25.0",
    "pydantic>=2.5.0",
    "httpx>=0.26.0",
    "structlog>=24.1.0",
    "tiktoken>=0.5.0",
]

[project.optional-dependencies]
anthropic = ["anthropic>=0.18.0"]
openai = ["openai>=1.10.0"]
langchain = ["langchain>=0.1.0"]
dev = [
    "pytest>=7.4.0",
    "pytest-asyncio>=0.23.0",
    "ruff>=0.1.0",
    "mypy>=1.8.0",
]

[tool.hatch.build.targets.wheel]
packages = ["src/agenttrace"]

[tool.ruff]
line-length = 100
target-version = "py311"

[tool.ruff.lint]
select = ["E", "F", "I", "N", "W", "UP"]

[tool.mypy]
python_version = "3.11"
strict = true
PYPROJECT

# Create API pyproject.toml
cat > api/pyproject.toml << 'PYPROJECT'
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "agenttrace-api"
version = "0.1.0"
description = "REST API for AgentTrace"
license = "MIT"
requires-python = ">=3.11"
dependencies = [
    "fastapi>=0.109.0",
    "uvicorn[standard]>=0.27.0",
    "pydantic>=2.5.0",
    "sqlalchemy>=2.0.0",
    "asyncpg>=0.29.0",
    "redis>=5.0.0",
    "structlog>=24.1.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "pytest-asyncio>=0.23.0",
    "httpx>=0.26.0",
    "ruff>=0.1.0",
]

[tool.hatch.build.targets.wheel]
packages = ["src"]

[tool.ruff]
line-length = 100
target-version = "py311"
PYPROJECT

# Create dashboard package.json
cat > dashboard/package.json << 'PACKAGE'
{
  "name": "agenttrace-dashboard",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "14.1.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tanstack/react-query": "^5.17.0",
    "recharts": "^2.10.0",
    "zustand": "^4.4.0",
    "date-fns": "^3.2.0",
    "lucide-react": "^0.309.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0"
  },
  "devDependencies": {
    "typescript": "^5.3.0",
    "@types/node": "^20.11.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0",
    "eslint": "^8.56.0",
    "eslint-config-next": "14.1.0"
  }
}
PACKAGE

# Create docker-compose.yml
cat > docker-compose.yml << 'DOCKER'
version: '3.8'

services:
  collector:
    build:
      context: ./collector
      dockerfile: Dockerfile
    ports:
      - "4317:4317"
      - "4318:4318"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://agenttrace:agenttrace@timescaledb:5432/agenttrace
      - REDIS_URL=redis://redis:6379
    depends_on:
      timescaledb:
        condition: service_healthy
      redis:
        condition: service_started

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://agenttrace:agenttrace@timescaledb:5432/agenttrace
      - REDIS_URL=redis://redis:6379
    depends_on:
      - timescaledb
      - redis

  dashboard:
    build:
      context: ./dashboard
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://localhost:8080
    depends_on:
      - api

  timescaledb:
    image: timescale/timescaledb:latest-pg15
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=agenttrace
      - POSTGRES_PASSWORD=agenttrace
      - POSTGRES_DB=agenttrace
    volumes:
      - timescale_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U agenttrace"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  timescale_data:
  redis_data:
DOCKER

# Create placeholder files
echo "// Entry point - see SPEC.md for implementation details" > collector/src/main.rs
echo "// Entry point - see SPEC.md for implementation details" > cli/src/main.rs
echo "// Entry point - see SPEC.md for implementation details" > tui/src/main.rs
echo '"""AgentTrace Python SDK"""' > sdk/python/src/agenttrace/__init__.py
echo '"""AgentTrace API"""' > api/src/__init__.py

# Create .gitignore
cat > .gitignore << 'GITIGNORE'
# Rust
/target/
Cargo.lock

# Python
__pycache__/
*.py[cod]
*$py.class
.venv/
*.egg-info/
dist/
build/
.ruff_cache/
.mypy_cache/

# Node
node_modules/
.next/
.turbo/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# Database
*.db
*.sqlite

# Logs
*.log
logs/
GITIGNORE

# Create LICENSE
cat > LICENSE << 'LICENSE'
MIT License

Copyright (c) 2025 Your Name

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
LICENSE

echo "✅ Project structure initialized!"
echo ""
echo "Next steps:"
echo "  1. Read SPEC.md for the full technical specification"
echo "  2. Read CLAUDE.md for development guidelines"
echo "  3. Start with the collector: cd collector && cargo build"
echo ""
echo "📁 Structure created:"
find . -type d | head -30

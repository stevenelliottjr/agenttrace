# Contributing to AgentTrace

Thank you for your interest in contributing to AgentTrace! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Be kind, constructive, and professional in all interactions.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, Python version)
   - Relevant logs or screenshots

### Suggesting Features

1. Check existing issues and discussions
2. Use the feature request template
3. Explain the use case and why it would benefit users

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit with clear messages
6. Push to your fork
7. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75+
- Python 3.11+
- Node.js 20+
- Docker and Docker Compose
- PostgreSQL client (for migrations)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/stevenelliottjr/agenttrace.git
cd agenttrace

# Start infrastructure
docker-compose up -d timescaledb redis

# Build Rust components
cargo build

# Install Python SDK in dev mode
cd sdk/python
pip install -e ".[dev]"

# Install dashboard dependencies
cd dashboard
pnpm install
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests
cd sdk/python && pytest

# Dashboard tests
cd dashboard && pnpm test
```

### Code Style

#### Rust
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix all warnings
- Use `thiserror` for error types
- Use `tracing` for logging

#### Python
- Run `ruff check --fix .` and `ruff format .`
- Use type hints everywhere
- Use Pydantic for data models

#### TypeScript/React
- Run `pnpm lint` and fix issues
- Use TypeScript strictly (no `any`)
- Use functional components with hooks

## Project Structure

```
agenttrace/
├── crates/
│   └── agenttrace-core/    # Rust collector, CLI, TUI
├── sdk/
│   └── python/             # Python SDK
├── api/                    # FastAPI server
├── dashboard/              # React dashboard
├── migrations/             # Database migrations
└── proto/                  # Protocol buffer definitions
```

## Architecture Guidelines

### Performance

The collector is performance-critical. When contributing:
- Avoid allocations in hot paths
- Use zero-copy parsing where possible
- Benchmark changes that touch the pipeline
- Target < 1ms overhead per span

### Privacy

AgentTrace is local-first:
- Never add external network calls without explicit opt-in
- Never collect telemetry
- Be mindful of what gets logged

### Compatibility

- Maintain backward compatibility for the SDK API
- Use semantic versioning for breaking changes
- Document migration paths for breaking changes

## Commit Messages

Use clear, descriptive commit messages:

```
feat(collector): add UDP receiver for high-throughput ingestion

- Implement UDP socket listener on configurable port
- Add batch processing for incoming spans
- Update configuration to support UDP settings

Closes #123
```

Prefixes:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

## Getting Help

- Open a GitHub Discussion for questions
- Check existing issues and documentation
- Join our community chat (coming soon)

## Recognition

Contributors are recognized in:
- GitHub contributors list
- Release notes for significant contributions
- CONTRIBUTORS.md file (for major contributors)

Thank you for helping make AgentTrace better!

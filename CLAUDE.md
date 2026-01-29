# CLAUDE.md — AgentTrace Development Guide

> This file provides context and instructions for Claude Code when working on the AgentTrace project.

## Project Overview

AgentTrace is a "Datadog for AI Agents" — a high-performance observability platform that provides real-time telemetry, cost attribution, and reasoning chain visualization for AI agent systems.

**Read the full specification in `SPEC.md` before starting any work.**

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                      AgentTrace Stack                            │
├─────────────────────────────────────────────────────────────────┤
│  Python SDK → Collector (Rust) → TimescaleDB + Redis            │
│                    ↓                                             │
│            API (FastAPI) → Dashboard (React)                     │
└─────────────────────────────────────────────────────────────────┘
```

## Repository Structure

```
agenttrace/
├── CLAUDE.md                    # This file
├── SPEC.md                      # Full specification
├── Cargo.toml                   # Rust workspace
├── collector/                   # Rust collector service
│   ├── Cargo.toml
│   └── src/
├── cli/                         # Rust CLI
│   ├── Cargo.toml
│   └── src/
├── tui/                         # Rust TUI
│   ├── Cargo.toml
│   └── src/
├── sdk/
│   └── python/                  # Python SDK
│       ├── pyproject.toml
│       └── src/agenttrace/
├── api/                         # FastAPI server
│   ├── pyproject.toml
│   └── src/
├── dashboard/                   # React dashboard
│   ├── package.json
│   └── src/
├── proto/                       # Protocol buffer definitions
│   └── agenttrace.proto
├── migrations/                  # Database migrations
├── docker/                      # Docker files
├── docs/                        # Documentation
└── examples/                    # Usage examples
```

## Development Commands

### Rust (Collector, CLI, TUI)

```bash
# Build all Rust components
cargo build --release

# Run collector in development
cargo run -p agenttrace-collector

# Run CLI
cargo run -p agenttrace-cli -- <command>

# Run tests
cargo test

# Format and lint
cargo fmt
cargo clippy -- -D warnings
```

### Python SDK & API

```bash
# Install SDK in development mode
cd sdk/python
pip install -e ".[dev]"

# Run API server
cd api
uvicorn src.main:app --reload --port 8080

# Run tests
pytest

# Format and lint
ruff check --fix .
ruff format .
```

### Dashboard

```bash
cd dashboard
pnpm install
pnpm dev        # Development server on :3000
pnpm build      # Production build
pnpm lint       # Lint check
```

### Docker

```bash
# Start full stack
docker-compose up -d

# View logs
docker-compose logs -f collector

# Stop
docker-compose down
```

## Key Design Decisions

### 1. Performance is Non-Negotiable

The collector must add < 1ms latency per span. This means:

- Use zero-copy parsing where possible
- Batch writes to TimescaleDB
- Use UDP for high-throughput scenarios
- Avoid allocations in hot paths

### 2. OpenTelemetry-Inspired, Not OpenTelemetry

We borrow concepts (spans, traces, attributes) but don't implement the full OTel spec. Our protocol is simpler and AI-agent-specific:

- First-class token usage tracking
- Cost calculation built-in
- Reasoning chain concepts (not just generic spans)
- Simpler attribute model

### 3. Local-First, Cloud-Optional

Everything runs locally by default. No cloud account required. This is critical for:

- Privacy (agent traces may contain sensitive data)
- Developer experience (works offline)
- Performance (no network latency for traces)

### 4. Beautiful Defaults

The dashboard and TUI should be visually impressive out of the box. No configuration required for good-looking visualizations.

## Code Style Guidelines

### Rust

- Use `thiserror` for error types
- Use `anyhow` for application errors
- Prefer `async` for I/O operations
- Use `tracing` for logging (not `log`)
- Follow Rust API guidelines

```rust
// Good: descriptive error types
#[derive(thiserror::Error, Debug)]
pub enum CollectorError {
    #[error("failed to connect to database: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    
    #[error("invalid span: {reason}")]
    InvalidSpan { reason: String },
}

// Good: structured logging
tracing::info!(
    span_id = %span.span_id,
    duration_us = span.duration_us,
    "span processed"
);
```

### Python

- Use type hints everywhere
- Use Pydantic for data models
- Use `structlog` for logging
- Follow Google Python style guide

```python
# Good: typed and documented
async def process_span(span: Span) -> ProcessedSpan:
    """Process a raw span and enrich with computed fields.
    
    Args:
        span: The raw span from the agent
        
    Returns:
        Enriched span with duration and cost calculated
    """
    ...
```

### TypeScript/React

- Use functional components with hooks
- Use TypeScript strictly (no `any`)
- Use TanStack Query for data fetching
- Use Zustand for global state (minimal)

```typescript
// Good: typed props and hooks
interface TraceWaterfallProps {
  traceId: string;
  onSpanSelect?: (spanId: string) => void;
}

export function TraceWaterfall({ traceId, onSpanSelect }: TraceWaterfallProps) {
  const { data: trace, isLoading } = useTrace(traceId);
  // ...
}
```

## Testing Strategy

### Unit Tests

- Rust: `#[test]` modules in each file
- Python: `pytest` with fixtures
- TypeScript: Vitest for components

### Integration Tests

- Test collector → database pipeline
- Test SDK → collector communication
- Test API → database queries

### End-to-End Tests

- Full stack with docker-compose
- Simulate agent workloads
- Verify dashboard renders correctly

## Common Tasks

### Adding a New Span Type

1. Add variant to `SpanType` enum in `collector/src/models/span.rs`
2. Add corresponding variant in `proto/agenttrace.proto`
3. Update Python SDK `models.py`
4. Add icon/color in dashboard `SpanTypeIcon.tsx`

### Adding a New LLM Provider

1. Add pricing to `collector/src/pipeline/cost_calculator.rs`
2. Add auto-instrumentation in `sdk/python/src/agenttrace/integrations/`
3. Update documentation

### Adding a New Dashboard Chart

1. Create component in `dashboard/src/components/charts/`
2. Create corresponding API endpoint in `api/`
3. Add to relevant page

## Performance Benchmarks

Run benchmarks before submitting PRs that touch hot paths:

```bash
# Rust benchmarks
cargo bench -p agenttrace-collector

# Load testing
./scripts/load-test.sh
```

Target metrics:
- Span ingestion: > 50,000 spans/second
- p99 latency: < 5ms
- Memory usage: < 100MB for 1M spans

## Debugging Tips

### Collector Issues

```bash
# Enable debug logging
RUST_LOG=agenttrace_collector=debug cargo run -p agenttrace-collector

# Check gRPC connectivity
grpcurl -plaintext localhost:4317 agenttrace.v1.AgentTraceCollector/Health
```

### SDK Issues

```python
# Enable debug mode
import agenttrace
agenttrace.configure(debug=True)

# Use console exporter for testing
from agenttrace.exporters.console import ConsoleExporter
tracer = AgentTrace(exporter=ConsoleExporter())
```

### Database Issues

```sql
-- Check recent spans
SELECT * FROM spans ORDER BY start_time DESC LIMIT 10;

-- Check continuous aggregate status
SELECT * FROM timescaledb_information.continuous_aggregates;

-- Check chunk compression
SELECT * FROM timescaledb_information.compressed_chunk_stats;
```

## Release Process

1. Update version in all `Cargo.toml` and `pyproject.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create git tag `v0.x.y`
5. GitHub Actions builds and publishes:
   - Rust binaries to GitHub Releases
   - Python package to PyPI
   - Docker images to GHCR

## Questions?

If you're unsure about implementation details:

1. Check `SPEC.md` for the authoritative design
2. Look at existing similar code in the codebase
3. Prioritize simplicity and performance
4. When in doubt, keep it local-first and privacy-preserving

## Priority Order for Implementation

1. **Collector core** — Without this, nothing works
2. **Python SDK** — Most agents are Python
3. **API server** — Dashboard needs data
4. **Dashboard** — The demo-able part
5. **CLI** — Quality of life
6. **TUI** — Nice to have

Focus on getting a working end-to-end trace before polishing any single component.

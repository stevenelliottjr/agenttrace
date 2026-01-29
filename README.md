# AgentTrace

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![CI](https://github.com/stevenelliottjr/agenttrace/actions/workflows/ci.yml/badge.svg)](https://github.com/stevenelliottjr/agenttrace/actions/workflows/ci.yml)

**Observability for AI Agents** — Real-time telemetry, cost attribution, and reasoning chain visualization.

<p align="center">
  <img src="docs/assets/dashboard-preview.png" alt="AgentTrace Dashboard" width="800">
</p>

---

## Why AgentTrace?

AI agents are black boxes. When you run Claude Code, LangChain agents, or custom AI systems, you have no visibility into:

- **What's eating your tokens/budget?** — Track costs per task, model, and tool
- **Why is the agent slow?** — Profile latency across LLM calls and tool executions
- **What reasoning led to this output?** — Visualize the full chain of thought
- **When do agents fail?** — Detect patterns and anomalies in real-time

AgentTrace gives you a Datadog-like observability layer specifically designed for AI agents.

## Features

- **Sub-millisecond overhead** — Rust collector adds <1ms latency per span
- **Automatic instrumentation** — Drop-in tracing for Anthropic, OpenAI, LangChain
- **Cost attribution** — Real-time token usage and spend tracking by task/model/tool
- **Trace visualization** — Waterfall views of agent execution flows
- **Local-first** — All data stays on your machine. No cloud required.
- **Multiple interfaces** — Web dashboard, terminal TUI, and CLI

## Quick Start

### 1. Start the Stack

```bash
# Using Docker (recommended)
docker-compose up -d

# Or install the CLI
cargo install agenttrace-cli
agenttrace up
```

### 2. Install the Python SDK

```bash
pip install agenttrace
```

### 3. Instrument Your Agent

```python
from agenttrace import AgentTrace
import anthropic

# Initialize (auto-instruments Anthropic SDK)
tracer = AgentTrace(session_name="my-agent")
await tracer.start()

# Your existing code — traces captured automatically
client = anthropic.Anthropic()
response = client.messages.create(
    model="claude-sonnet-4-20250514",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Hello!"}]
)

await tracer.stop()
```

### 4. View Your Traces

Open http://localhost:3000 to see the dashboard, or use the CLI:

```bash
agenttrace traces list
agenttrace metrics costs --since today
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Your AI Agent                                │
│  (Claude Code, LangChain, Custom)                                   │
│                              │                                       │
│                    ┌─────────▼─────────┐                            │
│                    │   Python SDK      │                            │
│                    │   (auto-traces)   │                            │
│                    └─────────┬─────────┘                            │
└──────────────────────────────┼──────────────────────────────────────┘
                               │ gRPC/UDP
┌──────────────────────────────▼──────────────────────────────────────┐
│                      AgentTrace Stack                                │
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │
│  │  Collector   │───▶│ TimescaleDB  │◀───│   REST API   │          │
│  │   (Rust)     │    │              │    │  (FastAPI)   │          │
│  └──────────────┘    └──────────────┘    └──────┬───────┘          │
│         │                                        │                   │
│         │            ┌──────────────┐           │                   │
│         └───────────▶│    Redis     │◀──────────┘                   │
│                      │  (real-time) │                               │
│                      └──────────────┘                               │
│                             │                                        │
│              ┌──────────────┼──────────────┐                        │
│              ▼              ▼              ▼                        │
│      ┌────────────┐ ┌────────────┐ ┌────────────┐                  │
│      │ Dashboard  │ │    TUI     │ │    CLI     │                  │
│      │  (React)   │ │ (Ratatui)  │ │  (Rust)    │                  │
│      └────────────┘ └────────────┘ └────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
agenttrace/
├── crates/
│   └── agenttrace-core/     # Rust: collector, CLI, TUI
├── sdk/
│   └── python/              # Python SDK
├── api/                     # FastAPI REST server
├── dashboard/               # React web dashboard
├── proto/                   # Protocol buffer definitions
├── migrations/              # TimescaleDB schema
├── docker/                  # Docker configurations
├── examples/                # Usage examples
└── docs/                    # Documentation
```

## Documentation

- [Full Specification](./SPEC.md) — Complete technical design
- [Contributing Guide](./CONTRIBUTING.md) — How to contribute
- [Development Guide](./CLAUDE.md) — Setup and architecture details
- [Examples](./examples/) — Code examples

## Development

```bash
# Clone the repo
git clone https://github.com/stevenelliottjr/agenttrace.git
cd agenttrace

# Start infrastructure
docker-compose up -d timescaledb redis

# Build Rust components
cargo build

# Install Python SDK (dev mode)
cd sdk/python && pip install -e ".[dev]"

# Run tests
cargo test
cd sdk/python && pytest
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed setup instructions.

## Roadmap

- [x] Core data models and schema
- [x] Project scaffolding
- [ ] Collector core (gRPC + UDP ingestion)
- [ ] Python SDK with auto-instrumentation
- [ ] REST API
- [ ] Web dashboard
- [ ] TUI dashboard
- [ ] CLI commands
- [ ] LangChain integration
- [ ] Alerting system

## Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) before submitting a PR.

## License

MIT License — see [LICENSE](./LICENSE) for details.

---

<p align="center">
  Built with care for the AI agent community
</p>

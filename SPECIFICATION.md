# AgentTrace — Observability Platform for AI Agents

## Executive Summary

AgentTrace is an open-source observability platform designed specifically for AI agents. It provides real-time monitoring, debugging, and cost analysis for agentic AI systems. Think "Datadog for AI agents" — a production-grade solution that fills a critical gap in the AI tooling ecosystem.

**Target Users:**
- Teams deploying Claude Code, LangChain, CrewAI, AutoGPT, or custom agents
- Enterprises needing cost attribution and compliance tracking
- Developers debugging complex multi-agent systems

**Key Differentiators:**
- Sub-microsecond overhead Rust telemetry collector
- Beautiful TUI and web dashboard
- MCP server for native Claude Code integration
- Hierarchical trace visualization for nested agent calls
- Predictive cost forecasting with anomaly detection

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           AgentTrace System                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │   Python     │    │   Node.js    │    │      Rust Collector      │  │
│  │     SDK      │───▶│     SDK      │───▶│   (agenttrace-core)     │  │
│  │              │    │              │    │                          │  │
│  │  - OpenAI    │    │  - LangChain │    │  - UDP/gRPC ingestion   │  │
│  │  - Anthropic │    │  - Vercel AI │    │  - Ring buffer          │  │
│  │  - LiteLLM   │    │  - Custom    │    │  - Batch processing     │  │
│  └──────────────┘    └──────────────┘    │  - Zero-copy parsing    │  │
│                                          └───────────┬──────────────┘  │
│                                                      │                  │
│                    ┌─────────────────────────────────┼─────────────┐   │
│                    │                                 ▼             │   │
│                    │         ┌──────────────────────────┐         │   │
│                    │         │      Message Queue       │         │   │
│                    │         │        (Redis)           │         │   │
│                    │         └────────────┬─────────────┘         │   │
│                    │                      │                        │   │
│                    │    ┌─────────────────┼─────────────────┐     │   │
│                    │    ▼                 ▼                 ▼     │   │
│  ┌──────────────┐ │ ┌──────┐        ┌──────────┐      ┌───────┐  │   │
│  │     TUI      │ │ │Traces│        │ Metrics  │      │Alerts │  │   │
│  │  Dashboard   │ │ │Writer│        │Aggregator│      │Engine │  │   │
│  │   (Ratatui)  │ │ └──┬───┘        └────┬─────┘      └───┬───┘  │   │
│  └──────────────┘ │    │                 │                │      │   │
│                    │    ▼                 ▼                ▼      │   │
│  ┌──────────────┐ │ ┌─────────────────────────────────────────┐  │   │
│  │     Web      │ │ │              TimescaleDB                │  │   │
│  │  Dashboard   │◀┼─│    (PostgreSQL + time-series ext)       │  │   │
│  │   (React)    │ │ └─────────────────────────────────────────┘  │   │
│  └──────────────┘ │                                              │   │
│                    │         Storage Layer                       │   │
│                    └─────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                         MCP Server                                │  │
│  │   - Native Claude Code integration                                │  │
│  │   - Real-time metrics queries                                     │  │
│  │   - Cost analysis tools                                           │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Rust Collector (`agenttrace-core`)

The high-performance telemetry collector written in Rust.

**Crate Structure:**
```
agenttrace-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs              # CLI entry point
│   ├── collector/
│   │   ├── mod.rs
│   │   ├── udp.rs           # UDP receiver
│   │   ├── grpc.rs          # gRPC receiver
│   │   └── buffer.rs        # Lock-free ring buffer
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── span.rs          # Span parsing
│   │   ├── event.rs         # Event parsing
│   │   └── metrics.rs       # Metrics extraction
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── timescale.rs     # TimescaleDB writer
│   │   └── redis.rs         # Redis pub/sub
│   ├── aggregator/
│   │   ├── mod.rs
│   │   ├── rollup.rs        # Time-based rollups
│   │   └── anomaly.rs       # Anomaly detection
│   ├── api/
│   │   ├── mod.rs
│   │   ├── rest.rs          # REST API (Axum)
│   │   └── websocket.rs     # Real-time streaming
│   └── tui/
│       ├── mod.rs
│       ├── app.rs           # Application state
│       ├── dashboard.rs     # Main dashboard view
│       ├── traces.rs        # Trace explorer
│       ├── costs.rs         # Cost analysis
│       └── alerts.rs        # Alerts view
```

**Key Dependencies (Cargo.toml):**
```toml
[package]
name = "agenttrace"
version = "0.1.0"
edition = "2021"
description = "Observability platform for AI agents"
license = "Apache-2.0"
repository = "https://github.com/yourusername/agenttrace"

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
deadpool-redis = "0.14"

# gRPC
tonic = "0.10"
prost = "0.12"

# TUI
ratatui = "0.26"
crossterm = "0.27"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
clap = { version = "4.4", features = ["derive"] }
config = "0.14"
bytes = "1.5"
parking_lot = "0.12"

[build-dependencies]
tonic-build = "0.10"
```

### 2. Python SDK (`agenttrace-py`)

Zero-config instrumentation for Python AI frameworks.

**Package Structure:**
```
agenttrace-py/
├── pyproject.toml
├── src/
│   └── agenttrace/
│       ├── __init__.py
│       ├── client.py          # Core client
│       ├── span.py            # Span management
│       ├── context.py         # Context propagation
│       ├── exporters/
│       │   ├── __init__.py
│       │   ├── udp.py         # UDP exporter
│       │   ├── grpc.py        # gRPC exporter
│       │   └── console.py     # Debug exporter
│       ├── integrations/
│       │   ├── __init__.py
│       │   ├── openai.py      # OpenAI auto-instrumentation
│       │   ├── anthropic.py   # Anthropic auto-instrumentation
│       │   ├── litellm.py     # LiteLLM auto-instrumentation
│       │   ├── langchain.py   # LangChain auto-instrumentation
│       │   └── crewai.py      # CrewAI auto-instrumentation
│       ├── processors/
│       │   ├── __init__.py
│       │   ├── batch.py       # Batch processor
│       │   └── sampling.py    # Sampling processor
│       └── utils/
│           ├── __init__.py
│           ├── tokens.py      # Token counting
│           └── costs.py       # Cost calculation
```

**Example Usage:**
```python
import agenttrace
from agenttrace.integrations import openai, anthropic

# Auto-instrument with one line
agenttrace.init(
    service_name="my-agent",
    endpoint="localhost:4317",
    environment="production"
)

# Or manual instrumentation
from agenttrace import trace, span

@trace
def my_agent_function():
    with span("planning", attributes={"step": "initial"}):
        # Agent logic here
        pass
```

### 3. Node.js SDK (`agenttrace-js`)

For JavaScript/TypeScript agent frameworks.

**Package Structure:**
```
agenttrace-js/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── client.ts
│   ├── span.ts
│   ├── context.ts
│   ├── exporters/
│   │   ├── index.ts
│   │   ├── udp.ts
│   │   └── grpc.ts
│   └── integrations/
│       ├── index.ts
│       ├── vercel-ai.ts
│       ├── langchain.ts
│       └── openai.ts
```

### 4. Web Dashboard (React)

Modern, responsive dashboard for trace visualization.

**Tech Stack:**
- React 18 with TypeScript
- Tailwind CSS for styling
- Recharts for visualizations
- TanStack Query for data fetching
- Zustand for state management

**Component Structure:**
```
dashboard/
├── package.json
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── Layout.tsx
│   │   ├── traces/
│   │   │   ├── TraceList.tsx
│   │   │   ├── TraceDetail.tsx
│   │   │   ├── TraceTimeline.tsx
│   │   │   └── SpanTree.tsx
│   │   ├── metrics/
│   │   │   ├── TokenUsage.tsx
│   │   │   ├── CostChart.tsx
│   │   │   ├── LatencyChart.tsx
│   │   │   └── ErrorRate.tsx
│   │   ├── agents/
│   │   │   ├── AgentList.tsx
│   │   │   ├── AgentDetail.tsx
│   │   │   └── AgentComparison.tsx
│   │   └── common/
│   │       ├── Card.tsx
│   │       ├── Table.tsx
│   │       └── Chart.tsx
│   ├── hooks/
│   │   ├── useTraces.ts
│   │   ├── useMetrics.ts
│   │   └── useWebSocket.ts
│   ├── stores/
│   │   ├── traceStore.ts
│   │   └── filterStore.ts
│   └── api/
│       ├── client.ts
│       └── types.ts
```

### 5. MCP Server

Native integration with Claude Code and other MCP-compatible clients.

**Tools Provided:**
- `agenttrace_query_traces` - Search and filter traces
- `agenttrace_get_metrics` - Get aggregated metrics
- `agenttrace_analyze_costs` - Cost breakdown analysis
- `agenttrace_detect_anomalies` - Find unusual patterns
- `agenttrace_compare_runs` - Compare agent performance

---

## Database Schema

### TimescaleDB Tables

```sql
-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Traces table (parent of spans)
CREATE TABLE traces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trace_id VARCHAR(32) NOT NULL UNIQUE,
    service_name VARCHAR(255) NOT NULL,
    environment VARCHAR(50) DEFAULT 'production',
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms DOUBLE PRECISION,
    status VARCHAR(20) DEFAULT 'in_progress',
    root_span_id VARCHAR(32),
    total_tokens_in INTEGER DEFAULT 0,
    total_tokens_out INTEGER DEFAULT 0,
    total_cost_usd DECIMAL(10, 6) DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    tags VARCHAR(100)[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_traces_trace_id ON traces(trace_id);
CREATE INDEX idx_traces_service_name ON traces(service_name);
CREATE INDEX idx_traces_started_at ON traces(started_at DESC);
CREATE INDEX idx_traces_tags ON traces USING GIN(tags);

-- Spans table (hypertable for time-series optimization)
CREATE TABLE spans (
    id UUID DEFAULT gen_random_uuid(),
    span_id VARCHAR(32) NOT NULL,
    trace_id VARCHAR(32) NOT NULL,
    parent_span_id VARCHAR(32),
    operation_name VARCHAR(255) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    span_kind VARCHAR(20) DEFAULT 'internal',
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms DOUBLE PRECISION,
    status VARCHAR(20) DEFAULT 'ok',
    status_message TEXT,
    
    -- AI-specific fields
    model_name VARCHAR(100),
    model_provider VARCHAR(50),
    tokens_in INTEGER,
    tokens_out INTEGER,
    cost_usd DECIMAL(10, 6),
    
    -- Tool usage
    tool_name VARCHAR(100),
    tool_input JSONB,
    tool_output JSONB,
    
    -- Attributes and events
    attributes JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    
    PRIMARY KEY (span_id, started_at)
);

-- Convert to hypertable for time-series optimization
SELECT create_hypertable('spans', 'started_at', chunk_time_interval => INTERVAL '1 day');

CREATE INDEX idx_spans_trace_id ON spans(trace_id, started_at DESC);
CREATE INDEX idx_spans_parent_span_id ON spans(parent_span_id);
CREATE INDEX idx_spans_operation_name ON spans(operation_name);
CREATE INDEX idx_spans_model_name ON spans(model_name);
CREATE INDEX idx_spans_tool_name ON spans(tool_name);

-- Events table for span events
CREATE TABLE span_events (
    id UUID DEFAULT gen_random_uuid(),
    span_id VARCHAR(32) NOT NULL,
    trace_id VARCHAR(32) NOT NULL,
    name VARCHAR(255) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    attributes JSONB DEFAULT '{}',
    PRIMARY KEY (id, timestamp)
);

SELECT create_hypertable('span_events', 'timestamp', chunk_time_interval => INTERVAL '1 day');

-- Metrics aggregation table (materialized for fast queries)
CREATE TABLE metrics_hourly (
    time TIMESTAMPTZ NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    model_name VARCHAR(100),
    operation_name VARCHAR(255),
    
    -- Counters
    request_count BIGINT DEFAULT 0,
    error_count BIGINT DEFAULT 0,
    
    -- Token metrics
    tokens_in_sum BIGINT DEFAULT 0,
    tokens_out_sum BIGINT DEFAULT 0,
    tokens_in_avg DOUBLE PRECISION,
    tokens_out_avg DOUBLE PRECISION,
    
    -- Cost metrics
    cost_sum DECIMAL(12, 6) DEFAULT 0,
    cost_avg DECIMAL(10, 6),
    
    -- Latency metrics (percentiles)
    latency_p50_ms DOUBLE PRECISION,
    latency_p95_ms DOUBLE PRECISION,
    latency_p99_ms DOUBLE PRECISION,
    latency_avg_ms DOUBLE PRECISION,
    
    PRIMARY KEY (time, service_name, model_name, operation_name)
);

SELECT create_hypertable('metrics_hourly', 'time', chunk_time_interval => INTERVAL '1 day');

-- Continuous aggregate for real-time metrics
CREATE MATERIALIZED VIEW metrics_realtime
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', started_at) AS bucket,
    service_name,
    model_name,
    COUNT(*) as request_count,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
    SUM(tokens_in) as tokens_in_sum,
    SUM(tokens_out) as tokens_out_sum,
    SUM(cost_usd) as cost_sum,
    AVG(duration_ms) as latency_avg_ms,
    percentile_cont(0.50) WITHIN GROUP (ORDER BY duration_ms) as latency_p50_ms,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) as latency_p95_ms,
    percentile_cont(0.99) WITHIN GROUP (ORDER BY duration_ms) as latency_p99_ms
FROM spans
WHERE started_at > NOW() - INTERVAL '24 hours'
GROUP BY bucket, service_name, model_name;

-- Alerts configuration
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    service_name VARCHAR(255),
    condition_type VARCHAR(50) NOT NULL,  -- threshold, anomaly, rate_change
    metric VARCHAR(100) NOT NULL,
    operator VARCHAR(10) NOT NULL,        -- gt, lt, eq, gte, lte
    threshold DOUBLE PRECISION,
    window_minutes INTEGER DEFAULT 5,
    severity VARCHAR(20) DEFAULT 'warning',
    enabled BOOLEAN DEFAULT true,
    notification_channels JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alert history
CREATE TABLE alert_events (
    id UUID DEFAULT gen_random_uuid(),
    rule_id UUID REFERENCES alert_rules(id),
    triggered_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    severity VARCHAR(20),
    message TEXT,
    metric_value DOUBLE PRECISION,
    threshold_value DOUBLE PRECISION,
    metadata JSONB DEFAULT '{}',
    PRIMARY KEY (id, triggered_at)
);

SELECT create_hypertable('alert_events', 'triggered_at', chunk_time_interval => INTERVAL '7 days');

-- Model pricing reference table
CREATE TABLE model_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    model_name VARCHAR(100) NOT NULL,
    input_cost_per_1k DECIMAL(10, 6) NOT NULL,
    output_cost_per_1k DECIMAL(10, 6) NOT NULL,
    effective_date DATE NOT NULL,
    deprecated_date DATE,
    UNIQUE(provider, model_name, effective_date)
);

-- Insert common model pricing
INSERT INTO model_pricing (provider, model_name, input_cost_per_1k, output_cost_per_1k, effective_date) VALUES
('anthropic', 'claude-3-5-sonnet-20241022', 0.003, 0.015, '2024-10-22'),
('anthropic', 'claude-3-5-haiku-20241022', 0.001, 0.005, '2024-10-22'),
('anthropic', 'claude-3-opus-20240229', 0.015, 0.075, '2024-02-29'),
('anthropic', 'claude-sonnet-4-20250514', 0.003, 0.015, '2025-05-14'),
('anthropic', 'claude-opus-4-5-20251101', 0.015, 0.075, '2025-11-01'),
('openai', 'gpt-4o', 0.005, 0.015, '2024-05-13'),
('openai', 'gpt-4o-mini', 0.00015, 0.0006, '2024-07-18'),
('openai', 'gpt-4-turbo', 0.01, 0.03, '2024-04-09'),
('openai', 'o1', 0.015, 0.06, '2024-12-05'),
('openai', 'o1-mini', 0.003, 0.012, '2024-09-12');
```

---

## API Specification

### REST API Endpoints

```yaml
openapi: 3.0.0
info:
  title: AgentTrace API
  version: 1.0.0
  description: Observability API for AI agents

paths:
  /api/v1/traces:
    get:
      summary: List traces
      parameters:
        - name: service_name
          in: query
          schema:
            type: string
        - name: start_time
          in: query
          schema:
            type: string
            format: date-time
        - name: end_time
          in: query
          schema:
            type: string
            format: date-time
        - name: status
          in: query
          schema:
            type: string
            enum: [ok, error, in_progress]
        - name: min_duration_ms
          in: query
          schema:
            type: number
        - name: limit
          in: query
          schema:
            type: integer
            default: 50
        - name: offset
          in: query
          schema:
            type: integer
            default: 0
      responses:
        '200':
          description: List of traces
          content:
            application/json:
              schema:
                type: object
                properties:
                  traces:
                    type: array
                    items:
                      $ref: '#/components/schemas/Trace'
                  total:
                    type: integer
                  has_more:
                    type: boolean

  /api/v1/traces/{trace_id}:
    get:
      summary: Get trace details with all spans
      parameters:
        - name: trace_id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Trace with spans
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TraceDetail'

  /api/v1/spans:
    post:
      summary: Ingest spans (batch)
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                spans:
                  type: array
                  items:
                    $ref: '#/components/schemas/SpanInput'
      responses:
        '202':
          description: Spans accepted

  /api/v1/metrics:
    get:
      summary: Get aggregated metrics
      parameters:
        - name: service_name
          in: query
          schema:
            type: string
        - name: model_name
          in: query
          schema:
            type: string
        - name: start_time
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: end_time
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: granularity
          in: query
          schema:
            type: string
            enum: [1m, 5m, 15m, 1h, 1d]
            default: 5m
      responses:
        '200':
          description: Metrics timeseries
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MetricsResponse'

  /api/v1/costs:
    get:
      summary: Get cost breakdown
      parameters:
        - name: service_name
          in: query
          schema:
            type: string
        - name: group_by
          in: query
          schema:
            type: string
            enum: [service, model, operation, day, hour]
        - name: start_time
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: end_time
          in: query
          required: true
          schema:
            type: string
            format: date-time
      responses:
        '200':
          description: Cost breakdown
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CostBreakdown'

  /api/v1/alerts:
    get:
      summary: List alert rules
      responses:
        '200':
          description: Alert rules
    post:
      summary: Create alert rule
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AlertRuleInput'

  /api/v1/health:
    get:
      summary: Health check
      responses:
        '200':
          description: System health

components:
  schemas:
    Trace:
      type: object
      properties:
        id:
          type: string
          format: uuid
        trace_id:
          type: string
        service_name:
          type: string
        started_at:
          type: string
          format: date-time
        ended_at:
          type: string
          format: date-time
        duration_ms:
          type: number
        status:
          type: string
        total_tokens_in:
          type: integer
        total_tokens_out:
          type: integer
        total_cost_usd:
          type: number
        error_count:
          type: integer
        tags:
          type: array
          items:
            type: string

    TraceDetail:
      allOf:
        - $ref: '#/components/schemas/Trace'
        - type: object
          properties:
            spans:
              type: array
              items:
                $ref: '#/components/schemas/Span'

    Span:
      type: object
      properties:
        span_id:
          type: string
        trace_id:
          type: string
        parent_span_id:
          type: string
        operation_name:
          type: string
        service_name:
          type: string
        started_at:
          type: string
          format: date-time
        ended_at:
          type: string
          format: date-time
        duration_ms:
          type: number
        status:
          type: string
        model_name:
          type: string
        model_provider:
          type: string
        tokens_in:
          type: integer
        tokens_out:
          type: integer
        cost_usd:
          type: number
        tool_name:
          type: string
        tool_input:
          type: object
        tool_output:
          type: object
        attributes:
          type: object
        events:
          type: array

    SpanInput:
      type: object
      required:
        - span_id
        - trace_id
        - operation_name
        - started_at
      properties:
        span_id:
          type: string
        trace_id:
          type: string
        parent_span_id:
          type: string
        operation_name:
          type: string
        service_name:
          type: string
        started_at:
          type: string
          format: date-time
        ended_at:
          type: string
          format: date-time
        status:
          type: string
        model_name:
          type: string
        model_provider:
          type: string
        tokens_in:
          type: integer
        tokens_out:
          type: integer
        tool_name:
          type: string
        tool_input:
          type: object
        tool_output:
          type: object
        attributes:
          type: object

    MetricsResponse:
      type: object
      properties:
        buckets:
          type: array
          items:
            type: object
            properties:
              timestamp:
                type: string
                format: date-time
              request_count:
                type: integer
              error_count:
                type: integer
              tokens_in_sum:
                type: integer
              tokens_out_sum:
                type: integer
              cost_sum:
                type: number
              latency_avg_ms:
                type: number
              latency_p95_ms:
                type: number

    CostBreakdown:
      type: object
      properties:
        total_cost:
          type: number
        breakdown:
          type: array
          items:
            type: object
            properties:
              key:
                type: string
              cost:
                type: number
              tokens_in:
                type: integer
              tokens_out:
                type: integer
              request_count:
                type: integer

    AlertRuleInput:
      type: object
      required:
        - name
        - condition_type
        - metric
        - operator
        - threshold
      properties:
        name:
          type: string
        description:
          type: string
        service_name:
          type: string
        condition_type:
          type: string
          enum: [threshold, anomaly, rate_change]
        metric:
          type: string
        operator:
          type: string
          enum: [gt, lt, eq, gte, lte]
        threshold:
          type: number
        window_minutes:
          type: integer
        severity:
          type: string
          enum: [info, warning, critical]
        enabled:
          type: boolean
```

---

## CLI Commands

```bash
# Installation
cargo install agenttrace

# Start the collector
agenttrace serve --config config.toml

# Start TUI dashboard
agenttrace dashboard

# Start web dashboard server
agenttrace web --port 3000

# Query traces
agenttrace traces list --service my-agent --last 1h
agenttrace traces show <trace_id>
agenttrace traces export <trace_id> --format json

# View metrics
agenttrace metrics --service my-agent --last 24h
agenttrace costs --group-by model --last 7d

# Manage alerts
agenttrace alerts list
agenttrace alerts create --name "High Error Rate" --metric error_rate --threshold 0.05
agenttrace alerts test <rule_id>

# Database management
agenttrace db migrate
agenttrace db rollback
agenttrace db seed  # Seed with sample data for testing

# Development
agenttrace dev  # Run all services in dev mode
```

---

## Configuration

### config.toml

```toml
[server]
host = "0.0.0.0"
http_port = 8080
grpc_port = 4317
udp_port = 4318

[database]
url = "postgres://agenttrace:password@localhost:5432/agenttrace"
max_connections = 20
min_connections = 5

[redis]
url = "redis://localhost:6379"
max_connections = 10

[collector]
batch_size = 100
batch_timeout_ms = 1000
buffer_size = 10000

[aggregator]
rollup_interval_minutes = 5
retention_days = 30

[tui]
refresh_rate_ms = 1000
default_time_range = "1h"

[web]
cors_origins = ["http://localhost:3000"]
static_files = "./dashboard/dist"

[alerting]
check_interval_seconds = 30
notification_cooldown_minutes = 5

[logging]
level = "info"
format = "json"  # or "pretty"
```

---

## Development Roadmap

### Phase 1: Core Infrastructure (Weeks 1-2)
- [ ] Initialize Rust workspace with Cargo
- [ ] Set up TimescaleDB schema and migrations
- [ ] Implement basic collector with UDP receiver
- [ ] Create span/trace data models
- [ ] Basic REST API with Axum
- [ ] Docker Compose for local development

### Phase 2: Python SDK (Week 3)
- [ ] Core client and span management
- [ ] OpenAI auto-instrumentation
- [ ] Anthropic auto-instrumentation
- [ ] Batch exporter with retry logic
- [ ] PyPI package setup

### Phase 3: TUI Dashboard (Week 4)
- [ ] Ratatui application scaffolding
- [ ] Real-time trace list view
- [ ] Trace detail with span tree
- [ ] Metrics charts (sparklines)
- [ ] Keyboard navigation

### Phase 4: Web Dashboard (Weeks 5-6)
- [ ] React application setup
- [ ] Trace explorer with filtering
- [ ] Interactive span timeline
- [ ] Cost analysis dashboard
- [ ] Real-time WebSocket updates

### Phase 5: Advanced Features (Week 7)
- [ ] Alert rules engine
- [ ] Anomaly detection (basic)
- [ ] MCP server implementation
- [ ] Node.js SDK

### Phase 6: Polish & Launch (Week 8)
- [ ] Documentation site
- [ ] Installation scripts
- [ ] Performance optimization
- [ ] Integration tests
- [ ] GitHub release automation

---

## File Structure

```
agenttrace/
├── README.md
├── LICENSE
├── CLAUDE.md                    # Claude Code instructions
├── docker-compose.yml
├── Makefile
│
├── crates/
│   └── agenttrace-core/        # Main Rust crate
│       ├── Cargo.toml
│       ├── build.rs
│       ├── proto/
│       │   └── trace.proto
│       └── src/
│           └── ...
│
├── sdks/
│   ├── python/                  # Python SDK
│   │   ├── pyproject.toml
│   │   ├── README.md
│   │   └── src/
│   │       └── agenttrace/
│   │           └── ...
│   │
│   └── node/                    # Node.js SDK
│       ├── package.json
│       ├── tsconfig.json
│       └── src/
│           └── ...
│
├── dashboard/                   # React web dashboard
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── src/
│       └── ...
│
├── mcp/                         # MCP server
│   ├── package.json
│   └── src/
│       └── index.ts
│
├── migrations/                  # Database migrations
│   ├── 001_initial_schema.sql
│   └── ...
│
├── scripts/
│   ├── setup.sh
│   ├── dev.sh
│   └── release.sh
│
└── docs/
    ├── getting-started.md
    ├── architecture.md
    ├── api-reference.md
    └── sdk-guides/
        ├── python.md
        └── node.md
```

---

## Design Principles

1. **Zero-config by default** — Works out of the box with sensible defaults
2. **Progressive disclosure** — Simple for basic use, powerful when needed
3. **Performance first** — Sub-microsecond overhead on instrumented code
4. **Developer experience** — Beautiful TUI, clear error messages, great docs
5. **Open and extensible** — Standard protocols (OTLP-compatible), plugin architecture
6. **Privacy-aware** — Easy to redact sensitive data, local-first option

---

## Success Metrics

- Collector handles 100K+ spans/second on commodity hardware
- SDK adds <100μs overhead per instrumented call
- TUI renders at 60fps with 10K traces loaded
- Web dashboard loads in <2s on 3G
- Zero data loss under normal operation

---

## Next Steps

1. Review this specification
2. Run `claude code` with the CLAUDE.md file
3. Start with Phase 1 infrastructure
4. Iterate based on real usage

This specification is designed to be directly usable as context for Claude Code development sessions.

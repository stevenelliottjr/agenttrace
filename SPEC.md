# AgentTrace — Observability Platform for AI Agents

> "Datadog for AI Agents" — Real-time telemetry, cost attribution, and reasoning chain visualization for AI agent systems.

---

## Table of Contents

1. [Vision & Goals](#vision--goals)
2. [Architecture Overview](#architecture-overview)
3. [Technology Stack](#technology-stack)
4. [Core Components](#core-components)
5. [Data Models](#data-models)
6. [API Design](#api-design)
7. [SDK Design](#sdk-design)
8. [CLI Design](#cli-design)
9. [Dashboard Design](#dashboard-design)
10. [Deployment](#deployment)
11. [Development Roadmap](#development-roadmap)

---

## Vision & Goals

### Problem Statement

AI agents are black boxes. Developers deploying Claude Code, LangChain agents, AutoGPT, CrewAI, and custom agent systems have no visibility into:

- **Cost**: Which tool calls burn tokens? What's the cost per task?
- **Performance**: Where are the bottlenecks? Which LLM calls are slow?
- **Reliability**: When do agents fail? What patterns precede failures?
- **Reasoning**: What chain of thought led to this output?
- **Comparison**: How does agent v2 compare to v1?

### Solution

AgentTrace provides a unified observability layer for AI agents:

```
┌─────────────────────────────────────────────────────────────────┐
│                        AgentTrace                                │
├─────────────────────────────────────────────────────────────────┤
│  [Traces]  [Metrics]  [Costs]  [Alerts]  [Playground]           │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Agent Session: claude-code-abc123                       │    │
│  │  Duration: 2m 34s | Tokens: 45,231 | Cost: $0.89        │    │
│  │                                                          │    │
│  │  ▼ Task: "Refactor authentication module"               │    │
│  │    ├─ 🔍 Read file: src/auth/login.ts (234ms)           │    │
│  │    ├─ 🤔 Reasoning: "Need to extract JWT logic..." (1.2s)│   │
│  │    ├─ 📝 Edit file: src/auth/jwt.ts (456ms)             │    │
│  │    ├─ 🔍 Read file: src/auth/middleware.ts (123ms)      │    │
│  │    ├─ 🤔 Reasoning: "Middleware needs updating..." (0.8s)│   │
│  │    ├─ 📝 Edit file: src/auth/middleware.ts (567ms)      │    │
│  │    └─ ✅ Verification: Tests passed (3.2s)              │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Zero-Config Start**: `pip install agenttrace && agenttrace up` gets you running
2. **Sub-Millisecond Overhead**: Rust collector adds <1ms latency to agent operations
3. **Privacy-First**: All data stays local by default; cloud sync is opt-in
4. **Universal Compatibility**: Works with any agent framework via OpenTelemetry-inspired protocol
5. **Beautiful by Default**: TUI and web dashboard that developers actually want to use

### Success Metrics

- Trace overhead < 1ms per span
- Support 10,000+ spans/second on modest hardware
- Cold start to first trace < 30 seconds
- Dashboard renders 100k spans smoothly

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              User's Machine                                 │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐       │
│  │  Claude Code    │     │  LangChain      │     │  Custom Agent   │       │
│  │  + agenttrace   │     │  + agenttrace   │     │  + agenttrace   │       │
│  │    Python SDK   │     │    Python SDK   │     │    Python SDK   │       │
│  └────────┬────────┘     └────────┬────────┘     └────────┬────────┘       │
│           │                       │                       │                 │
│           └───────────────────────┼───────────────────────┘                 │
│                                   │                                         │
│                                   ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     AgentTrace Collector (Rust)                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │   │
│  │  │ UDP Receiver │  │ gRPC Server  │  │ Unix Socket  │               │   │
│  │  │ (high perf)  │  │ (rich data)  │  │ (local only) │               │   │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │   │
│  │         └─────────────────┼─────────────────┘                        │   │
│  │                           ▼                                          │   │
│  │  ┌────────────────────────────────────────────────────────────────┐ │   │
│  │  │                    Processing Pipeline                          │ │   │
│  │  │  [Decode] → [Enrich] → [Aggregate] → [Cost Calculate] → [Store]│ │   │
│  │  └────────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                         │
│           ┌───────────────────────┼───────────────────────┐                 │
│           ▼                       ▼                       ▼                 │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐       │
│  │   TimescaleDB   │     │     Redis       │     │   File Export   │       │
│  │  (time-series)  │     │  (real-time)    │     │  (JSON/Parquet) │       │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘       │
│           │                       │                                         │
│           └───────────────────────┘                                         │
│                       │                                                     │
│                       ▼                                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Query & Display Layer                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │   │
│  │  │  REST API    │  │  TUI (Ratatui)│  │ Web Dashboard│               │   │
│  │  │  (FastAPI)   │  │  (Rust)      │  │  (React)     │               │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Language | Responsibility |
|-----------|----------|----------------|
| **Collector** | Rust | High-performance telemetry ingestion, processing pipeline |
| **Python SDK** | Python | Agent instrumentation, decorators, context managers |
| **Node SDK** | TypeScript | For JS/TS agents (future) |
| **TUI** | Rust (Ratatui) | Terminal-based real-time monitoring |
| **API Server** | Python (FastAPI) | REST API for dashboard and integrations |
| **Dashboard** | React + TypeScript | Web-based visualization and analysis |
| **CLI** | Rust | `agenttrace` command-line tool |

---

## Technology Stack

### Core Infrastructure

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Collector Core** | Rust 1.75+ | Sub-millisecond latency, memory safety, async with Tokio |
| **Database** | TimescaleDB 2.x | Time-series optimized PostgreSQL, compression, continuous aggregates |
| **Cache/Streaming** | Redis 7.x | Real-time span streaming, session state, pub/sub |
| **API Server** | Python 3.11+ / FastAPI | Rapid development, excellent async, Pydantic validation |
| **Dashboard** | React 18 + TypeScript | Component reuse, strong typing, Recharts for viz |
| **TUI** | Rust + Ratatui | Native performance, consistent with collector |

### Rust Crates

```toml
# Cargo.toml dependencies
[dependencies]
tokio = { version = "1.35", features = ["full"] }
tonic = "0.10"                    # gRPC
prost = "0.12"                    # Protocol Buffers
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
ratatui = "0.25"                  # TUI framework
crossterm = "0.27"                # Terminal manipulation
clap = { version = "4.4", features = ["derive"] }
config = "0.14"                   # Configuration management
thiserror = "1.0"
anyhow = "1.0"
bytes = "1.5"
dashmap = "5.5"                   # Concurrent hashmap for caching
parking_lot = "0.12"              # Fast synchronization primitives
metrics = "0.22"                  # Internal metrics
tiktoken-rs = "0.5"               # Token counting
```

### Python Dependencies

```toml
# pyproject.toml
[project]
dependencies = [
    "fastapi>=0.109.0",
    "uvicorn[standard]>=0.27.0",
    "pydantic>=2.5.0",
    "sqlalchemy>=2.0.0",
    "asyncpg>=0.29.0",
    "redis>=5.0.0",
    "httpx>=0.26.0",
    "opentelemetry-api>=1.22.0",
    "tiktoken>=0.5.0",
    "structlog>=24.1.0",
    "typer>=0.9.0",
    "rich>=13.7.0",
]
```

### Frontend Dependencies

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "typescript": "^5.3.0",
    "@tanstack/react-query": "^5.17.0",
    "recharts": "^2.10.0",
    "tailwindcss": "^3.4.0",
    "@radix-ui/react-*": "latest",
    "lucide-react": "^0.309.0",
    "zustand": "^4.4.0",
    "date-fns": "^3.2.0"
  }
}
```

---

## Core Components

### 1. Collector (Rust)

The collector is the heart of AgentTrace — a high-performance telemetry ingestion and processing system.

#### Directory Structure

```
agenttrace-collector/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   ├── config.rs               # Configuration management
│   ├── server/
│   │   ├── mod.rs
│   │   ├── grpc.rs             # gRPC server (tonic)
│   │   ├── udp.rs              # UDP receiver for high-throughput
│   │   └── unix_socket.rs      # Unix domain socket
│   ├── pipeline/
│   │   ├── mod.rs
│   │   ├── decoder.rs          # Span decoding
│   │   ├── enricher.rs         # Add computed fields
│   │   ├── aggregator.rs       # Real-time aggregations
│   │   ├── cost_calculator.rs  # Token/cost calculation
│   │   └── processor.rs        # Pipeline orchestration
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── timescale.rs        # TimescaleDB writer
│   │   ├── redis.rs            # Redis streaming
│   │   └── file_export.rs      # JSON/Parquet export
│   ├── models/
│   │   ├── mod.rs
│   │   ├── span.rs             # Core span model
│   │   ├── trace.rs            # Trace (collection of spans)
│   │   ├── session.rs          # Agent session
│   │   └── metrics.rs          # Aggregated metrics
│   └── utils/
│       ├── mod.rs
│       └── token_counter.rs    # tiktoken integration
├── proto/
│   └── agenttrace.proto        # Protocol buffer definitions
└── tests/
    ├── integration/
    └── benchmarks/
```

#### Core Span Model

```rust
// src/models/span.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique identifier for this span
    pub span_id: Uuid,
    
    /// Trace ID - groups all spans in a single agent execution
    pub trace_id: Uuid,
    
    /// Parent span ID (None for root spans)
    pub parent_span_id: Option<Uuid>,
    
    /// Session ID - groups traces for a single agent instance
    pub session_id: Uuid,
    
    /// Human-readable name (e.g., "llm_call", "tool_execution", "reasoning")
    pub name: String,
    
    /// Span type for categorization and UI rendering
    pub span_type: SpanType,
    
    /// When the span started
    pub start_time: DateTime<Utc>,
    
    /// When the span ended (None if still running)
    pub end_time: Option<DateTime<Utc>>,
    
    /// Duration in microseconds (computed)
    pub duration_us: Option<i64>,
    
    /// Status of the span
    pub status: SpanStatus,
    
    /// Structured attributes
    pub attributes: SpanAttributes,
    
    /// Events that occurred during the span
    pub events: Vec<SpanEvent>,
    
    /// Cost information (computed)
    pub cost: Option<CostInfo>,
    
    /// Token usage (for LLM calls)
    pub token_usage: Option<TokenUsage>,
    
    /// Agent/framework metadata
    pub metadata: SpanMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanType {
    /// Root span for entire agent task
    Task,
    /// LLM API call
    LlmCall,
    /// Tool/function execution
    ToolExecution,
    /// Reasoning/thinking step
    Reasoning,
    /// Memory retrieval
    MemoryRetrieval,
    /// Memory storage
    MemoryStore,
    /// File read operation
    FileRead,
    /// File write operation
    FileWrite,
    /// External API call (non-LLM)
    ApiCall,
    /// Code execution
    CodeExecution,
    /// Verification/testing step
    Verification,
    /// Custom span type
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    /// Span is still running
    Running,
    /// Span completed successfully
    Ok,
    /// Span completed with error
    Error { message: String, code: Option<String> },
    /// Span was cancelled
    Cancelled,
    /// Span timed out
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAttributes {
    /// LLM-specific attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmAttributes>,
    
    /// Tool-specific attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolAttributes>,
    
    /// File operation attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileAttributes>,
    
    /// Custom key-value attributes
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAttributes {
    /// Model identifier (e.g., "claude-3-opus-20240229")
    pub model: String,
    
    /// Provider (e.g., "anthropic", "openai")
    pub provider: String,
    
    /// Temperature setting
    pub temperature: Option<f32>,
    
    /// Max tokens setting
    pub max_tokens: Option<i32>,
    
    /// Input prompt (truncated for storage)
    pub prompt_preview: Option<String>,
    
    /// Output response (truncated for storage)
    pub response_preview: Option<String>,
    
    /// Whether thinking/reasoning was enabled
    pub thinking_enabled: Option<bool>,
    
    /// Stop reason
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAttributes {
    /// Tool name (e.g., "bash", "read_file", "web_search")
    pub tool_name: String,
    
    /// Tool input (JSON)
    pub input: Option<serde_json::Value>,
    
    /// Tool output (truncated)
    pub output_preview: Option<String>,
    
    /// Whether tool succeeded
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributes {
    /// File path
    pub path: String,
    
    /// Operation type
    pub operation: FileOperation,
    
    /// Bytes read/written
    pub bytes: Option<i64>,
    
    /// Lines affected (for text files)
    pub lines: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event attributes
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub input_tokens: i32,
    
    /// Output/completion tokens
    pub output_tokens: i32,
    
    /// Total tokens
    pub total_tokens: i32,
    
    /// Cache read tokens (Anthropic-specific)
    pub cache_read_tokens: Option<i32>,
    
    /// Cache creation tokens (Anthropic-specific)
    pub cache_creation_tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Cost in USD
    pub total_usd: f64,
    
    /// Input token cost
    pub input_cost_usd: f64,
    
    /// Output token cost
    pub output_cost_usd: f64,
    
    /// Pricing model used
    pub pricing_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanMetadata {
    /// Agent framework (e.g., "claude-code", "langchain", "crewai")
    pub framework: Option<String>,
    
    /// Framework version
    pub framework_version: Option<String>,
    
    /// SDK version
    pub sdk_version: String,
    
    /// Hostname
    pub hostname: Option<String>,
    
    /// User-defined tags
    pub tags: Vec<String>,
}
```

#### gRPC Protocol Definition

```protobuf
// proto/agenttrace.proto

syntax = "proto3";

package agenttrace.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

service AgentTraceCollector {
  // Stream spans to the collector
  rpc StreamSpans(stream SpanBatch) returns (StreamSpansResponse);
  
  // Send a single span (for simpler integrations)
  rpc SendSpan(Span) returns (SendSpanResponse);
  
  // Start a new session
  rpc StartSession(StartSessionRequest) returns (StartSessionResponse);
  
  // End a session
  rpc EndSession(EndSessionRequest) returns (EndSessionResponse);
  
  // Health check
  rpc Health(HealthRequest) returns (HealthResponse);
}

message SpanBatch {
  repeated Span spans = 1;
}

message Span {
  string span_id = 1;
  string trace_id = 2;
  optional string parent_span_id = 3;
  string session_id = 4;
  string name = 5;
  SpanType span_type = 6;
  google.protobuf.Timestamp start_time = 7;
  optional google.protobuf.Timestamp end_time = 8;
  SpanStatus status = 9;
  SpanAttributes attributes = 10;
  repeated SpanEvent events = 11;
  optional TokenUsage token_usage = 12;
  SpanMetadata metadata = 13;
}

enum SpanType {
  SPAN_TYPE_UNSPECIFIED = 0;
  SPAN_TYPE_TASK = 1;
  SPAN_TYPE_LLM_CALL = 2;
  SPAN_TYPE_TOOL_EXECUTION = 3;
  SPAN_TYPE_REASONING = 4;
  SPAN_TYPE_MEMORY_RETRIEVAL = 5;
  SPAN_TYPE_MEMORY_STORE = 6;
  SPAN_TYPE_FILE_READ = 7;
  SPAN_TYPE_FILE_WRITE = 8;
  SPAN_TYPE_API_CALL = 9;
  SPAN_TYPE_CODE_EXECUTION = 10;
  SPAN_TYPE_VERIFICATION = 11;
  SPAN_TYPE_CUSTOM = 100;
}

message SpanStatus {
  StatusCode code = 1;
  optional string message = 2;
  optional string error_code = 3;
}

enum StatusCode {
  STATUS_CODE_UNSPECIFIED = 0;
  STATUS_CODE_RUNNING = 1;
  STATUS_CODE_OK = 2;
  STATUS_CODE_ERROR = 3;
  STATUS_CODE_CANCELLED = 4;
  STATUS_CODE_TIMEOUT = 5;
}

message SpanAttributes {
  optional LlmAttributes llm = 1;
  optional ToolAttributes tool = 2;
  optional FileAttributes file = 3;
  google.protobuf.Struct custom = 4;
}

message LlmAttributes {
  string model = 1;
  string provider = 2;
  optional float temperature = 3;
  optional int32 max_tokens = 4;
  optional string prompt_preview = 5;
  optional string response_preview = 6;
  optional bool thinking_enabled = 7;
  optional string stop_reason = 8;
}

message ToolAttributes {
  string tool_name = 1;
  optional google.protobuf.Struct input = 2;
  optional string output_preview = 3;
  bool success = 4;
}

message FileAttributes {
  string path = 1;
  FileOperation operation = 2;
  optional int64 bytes = 3;
  optional int32 lines = 4;
}

enum FileOperation {
  FILE_OPERATION_UNSPECIFIED = 0;
  FILE_OPERATION_READ = 1;
  FILE_OPERATION_WRITE = 2;
  FILE_OPERATION_CREATE = 3;
  FILE_OPERATION_DELETE = 4;
  FILE_OPERATION_MOVE = 5;
  FILE_OPERATION_COPY = 6;
}

message SpanEvent {
  string name = 1;
  google.protobuf.Timestamp timestamp = 2;
  google.protobuf.Struct attributes = 3;
}

message TokenUsage {
  int32 input_tokens = 1;
  int32 output_tokens = 2;
  int32 total_tokens = 3;
  optional int32 cache_read_tokens = 4;
  optional int32 cache_creation_tokens = 5;
}

message SpanMetadata {
  optional string framework = 1;
  optional string framework_version = 2;
  string sdk_version = 3;
  optional string hostname = 4;
  repeated string tags = 5;
}

message StartSessionRequest {
  string session_id = 1;
  optional string name = 2;
  SpanMetadata metadata = 3;
}

message StartSessionResponse {
  bool success = 1;
}

message EndSessionRequest {
  string session_id = 1;
}

message EndSessionResponse {
  bool success = 1;
  SessionSummary summary = 2;
}

message SessionSummary {
  int64 total_spans = 1;
  int64 total_tokens = 2;
  double total_cost_usd = 3;
  int64 duration_ms = 4;
}

message StreamSpansResponse {
  int32 accepted = 1;
  int32 rejected = 2;
}

message SendSpanResponse {
  bool success = 1;
}

message HealthRequest {}

message HealthResponse {
  bool healthy = 1;
  string version = 2;
}
```

#### Processing Pipeline

```rust
// src/pipeline/processor.rs

use tokio::sync::mpsc;
use crate::models::Span;
use crate::storage::{TimescaleWriter, RedisStreamer};

pub struct PipelineConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub enable_cost_calculation: bool,
    pub enable_real_time_streaming: bool,
}

pub struct Pipeline {
    config: PipelineConfig,
    decoder: Decoder,
    enricher: Enricher,
    aggregator: Aggregator,
    cost_calculator: CostCalculator,
    timescale: TimescaleWriter,
    redis: Option<RedisStreamer>,
}

impl Pipeline {
    pub async fn new(config: PipelineConfig) -> anyhow::Result<Self> {
        // Initialize components...
    }
    
    pub async fn process(&self, span: Span) -> anyhow::Result<()> {
        // 1. Decode (already done by gRPC layer)
        
        // 2. Enrich with computed fields
        let span = self.enricher.enrich(span).await?;
        
        // 3. Calculate costs if LLM span
        let span = if self.config.enable_cost_calculation {
            self.cost_calculator.calculate(span).await?
        } else {
            span
        };
        
        // 4. Update real-time aggregations
        self.aggregator.update(&span).await?;
        
        // 5. Store in TimescaleDB
        self.timescale.write(span.clone()).await?;
        
        // 6. Stream to Redis for real-time dashboards
        if self.config.enable_real_time_streaming {
            if let Some(redis) = &self.redis {
                redis.publish(&span).await?;
            }
        }
        
        Ok(())
    }
}
```

#### Cost Calculator

```rust
// src/pipeline/cost_calculator.rs

use crate::models::{Span, CostInfo, SpanType};
use std::collections::HashMap;

pub struct CostCalculator {
    /// Pricing per 1M tokens (input, output)
    pricing: HashMap<String, (f64, f64)>,
}

impl CostCalculator {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();
        
        // Anthropic pricing (as of Jan 2025)
        pricing.insert("claude-3-opus".to_string(), (15.0, 75.0));
        pricing.insert("claude-3-5-sonnet".to_string(), (3.0, 15.0));
        pricing.insert("claude-3-5-haiku".to_string(), (0.80, 4.0));
        pricing.insert("claude-sonnet-4".to_string(), (3.0, 15.0));
        pricing.insert("claude-opus-4".to_string(), (15.0, 75.0));
        
        // OpenAI pricing
        pricing.insert("gpt-4-turbo".to_string(), (10.0, 30.0));
        pricing.insert("gpt-4o".to_string(), (5.0, 15.0));
        pricing.insert("gpt-4o-mini".to_string(), (0.15, 0.60));
        
        Self { pricing }
    }
    
    pub async fn calculate(&self, mut span: Span) -> anyhow::Result<Span> {
        if !matches!(span.span_type, SpanType::LlmCall) {
            return Ok(span);
        }
        
        let Some(token_usage) = &span.token_usage else {
            return Ok(span);
        };
        
        let Some(llm_attrs) = &span.attributes.llm else {
            return Ok(span);
        };
        
        // Find pricing by model prefix
        let (input_price, output_price) = self.pricing
            .iter()
            .find(|(model, _)| llm_attrs.model.contains(model.as_str()))
            .map(|(_, prices)| *prices)
            .unwrap_or((0.0, 0.0));
        
        let input_cost = (token_usage.input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (token_usage.output_tokens as f64 / 1_000_000.0) * output_price;
        
        span.cost = Some(CostInfo {
            total_usd: input_cost + output_cost,
            input_cost_usd: input_cost,
            output_cost_usd: output_cost,
            pricing_model: llm_attrs.model.clone(),
        });
        
        Ok(span)
    }
}
```

### 2. Python SDK

The SDK provides a Pythonic interface for instrumenting agents.

#### Directory Structure

```
agenttrace-python/
├── pyproject.toml
├── src/
│   └── agenttrace/
│       ├── __init__.py
│       ├── client.py           # Core client
│       ├── context.py          # Context management
│       ├── decorators.py       # @trace decorators
│       ├── integrations/
│       │   ├── __init__.py
│       │   ├── anthropic.py    # Anthropic SDK auto-instrumentation
│       │   ├── openai.py       # OpenAI SDK auto-instrumentation
│       │   ├── langchain.py    # LangChain auto-instrumentation
│       │   └── litellm.py      # LiteLLM auto-instrumentation
│       ├── exporters/
│       │   ├── __init__.py
│       │   ├── grpc.py         # gRPC exporter
│       │   ├── http.py         # HTTP fallback
│       │   └── console.py      # Console exporter for debugging
│       └── models.py           # Pydantic models
└── tests/
```

#### Core Client

```python
# src/agenttrace/client.py

from __future__ import annotations

import asyncio
import atexit
import uuid
from contextvars import ContextVar
from datetime import datetime, timezone
from typing import Any, Optional, Callable
from functools import wraps

import structlog
from pydantic import BaseModel

from .models import Span, SpanType, SpanStatus, SpanAttributes, TokenUsage
from .exporters.grpc import GrpcExporter

logger = structlog.get_logger()

# Context variable for current span
_current_span: ContextVar[Optional[Span]] = ContextVar("current_span", default=None)
_current_trace: ContextVar[Optional[str]] = ContextVar("current_trace", default=None)

class AgentTrace:
    """Main AgentTrace client for instrumenting AI agents."""
    
    _instance: Optional[AgentTrace] = None
    
    def __init__(
        self,
        *,
        collector_url: str = "localhost:4317",
        session_name: Optional[str] = None,
        framework: Optional[str] = None,
        auto_instrument: bool = True,
        tags: Optional[list[str]] = None,
    ):
        self.session_id = str(uuid.uuid4())
        self.session_name = session_name
        self.framework = framework
        self.tags = tags or []
        
        self._exporter = GrpcExporter(collector_url)
        self._spans: list[Span] = []
        self._started = False
        
        if auto_instrument:
            self._setup_auto_instrumentation()
        
        # Register cleanup
        atexit.register(self._cleanup)
        
        AgentTrace._instance = self
    
    @classmethod
    def get_instance(cls) -> Optional[AgentTrace]:
        """Get the global AgentTrace instance."""
        return cls._instance
    
    def _setup_auto_instrumentation(self):
        """Set up automatic instrumentation for common libraries."""
        try:
            from .integrations.anthropic import instrument_anthropic
            instrument_anthropic(self)
        except ImportError:
            pass
        
        try:
            from .integrations.openai import instrument_openai
            instrument_openai(self)
        except ImportError:
            pass
    
    async def start(self):
        """Start the session."""
        if self._started:
            return
        
        await self._exporter.connect()
        await self._exporter.start_session(
            session_id=self.session_id,
            name=self.session_name,
            framework=self.framework,
            tags=self.tags,
        )
        self._started = True
        logger.info("AgentTrace session started", session_id=self.session_id)
    
    async def stop(self):
        """Stop the session and flush remaining spans."""
        if not self._started:
            return
        
        await self._exporter.end_session(self.session_id)
        await self._exporter.close()
        self._started = False
        logger.info("AgentTrace session stopped", session_id=self.session_id)
    
    def _cleanup(self):
        """Cleanup handler for atexit."""
        if self._started:
            asyncio.run(self.stop())
    
    def start_span(
        self,
        name: str,
        span_type: SpanType = SpanType.CUSTOM,
        attributes: Optional[dict[str, Any]] = None,
        tags: Optional[list[str]] = None,
    ) -> SpanContext:
        """Start a new span and return a context manager."""
        parent_span = _current_span.get()
        current_trace = _current_trace.get()
        
        span = Span(
            span_id=str(uuid.uuid4()),
            trace_id=current_trace or str(uuid.uuid4()),
            parent_span_id=parent_span.span_id if parent_span else None,
            session_id=self.session_id,
            name=name,
            span_type=span_type,
            start_time=datetime.now(timezone.utc),
            status=SpanStatus.RUNNING,
            attributes=SpanAttributes(**(attributes or {})),
            metadata={
                "framework": self.framework,
                "sdk_version": "0.1.0",
                "tags": (self.tags or []) + (tags or []),
            },
        )
        
        return SpanContext(self, span)
    
    async def _send_span(self, span: Span):
        """Send a completed span to the collector."""
        if self._started:
            await self._exporter.send_span(span)


class SpanContext:
    """Context manager for spans."""
    
    def __init__(self, client: AgentTrace, span: Span):
        self.client = client
        self.span = span
        self._token: Any = None
        self._trace_token: Any = None
    
    def __enter__(self) -> Span:
        self._token = _current_span.set(self.span)
        if not _current_trace.get():
            self._trace_token = _current_trace.set(self.span.trace_id)
        return self.span
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.span.end_time = datetime.now(timezone.utc)
        self.span.duration_us = int(
            (self.span.end_time - self.span.start_time).total_seconds() * 1_000_000
        )
        
        if exc_type is not None:
            self.span.status = SpanStatus.ERROR
            self.span.status_message = str(exc_val)
        else:
            self.span.status = SpanStatus.OK
        
        _current_span.reset(self._token)
        if self._trace_token:
            _current_trace.reset(self._trace_token)
        
        # Send span asynchronously
        asyncio.create_task(self.client._send_span(self.span))
        
        return False
    
    async def __aenter__(self) -> Span:
        return self.__enter__()
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        return self.__exit__(exc_type, exc_val, exc_tb)
    
    def set_attribute(self, key: str, value: Any):
        """Set a custom attribute on the span."""
        self.span.attributes.custom[key] = value
    
    def add_event(self, name: str, attributes: Optional[dict] = None):
        """Add an event to the span."""
        self.span.events.append({
            "name": name,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "attributes": attributes or {},
        })
    
    def set_token_usage(
        self,
        input_tokens: int,
        output_tokens: int,
        cache_read_tokens: Optional[int] = None,
        cache_creation_tokens: Optional[int] = None,
    ):
        """Set token usage for LLM spans."""
        self.span.token_usage = TokenUsage(
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            total_tokens=input_tokens + output_tokens,
            cache_read_tokens=cache_read_tokens,
            cache_creation_tokens=cache_creation_tokens,
        )


# Convenience functions
def get_tracer() -> Optional[AgentTrace]:
    """Get the global AgentTrace instance."""
    return AgentTrace.get_instance()


def trace(
    name: Optional[str] = None,
    span_type: SpanType = SpanType.CUSTOM,
    attributes: Optional[dict] = None,
):
    """Decorator to trace a function."""
    def decorator(func: Callable) -> Callable:
        span_name = name or func.__name__
        
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            tracer = get_tracer()
            if not tracer:
                return await func(*args, **kwargs)
            
            with tracer.start_span(span_name, span_type, attributes) as span:
                return await func(*args, **kwargs)
        
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            tracer = get_tracer()
            if not tracer:
                return func(*args, **kwargs)
            
            with tracer.start_span(span_name, span_type, attributes) as span:
                return func(*args, **kwargs)
        
        if asyncio.iscoroutinefunction(func):
            return async_wrapper
        return sync_wrapper
    
    return decorator
```

#### Anthropic Auto-Instrumentation

```python
# src/agenttrace/integrations/anthropic.py

from __future__ import annotations

import functools
from typing import TYPE_CHECKING

from ..models import SpanType, LlmAttributes

if TYPE_CHECKING:
    from ..client import AgentTrace


def instrument_anthropic(tracer: AgentTrace):
    """Auto-instrument the Anthropic SDK."""
    try:
        import anthropic
    except ImportError:
        return
    
    original_create = anthropic.Anthropic.messages.create
    original_async_create = anthropic.AsyncAnthropic.messages.create
    
    @functools.wraps(original_create)
    def traced_create(self, *args, **kwargs):
        with tracer.start_span(
            name="anthropic.messages.create",
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm": LlmAttributes(
                    model=kwargs.get("model", "unknown"),
                    provider="anthropic",
                    temperature=kwargs.get("temperature"),
                    max_tokens=kwargs.get("max_tokens"),
                    thinking_enabled=kwargs.get("thinking", {}).get("type") == "enabled",
                ).model_dump()
            }
        ) as span:
            response = original_create(self, *args, **kwargs)
            
            # Extract token usage from response
            if hasattr(response, "usage"):
                span.set_token_usage(
                    input_tokens=response.usage.input_tokens,
                    output_tokens=response.usage.output_tokens,
                    cache_read_tokens=getattr(response.usage, "cache_read_input_tokens", None),
                    cache_creation_tokens=getattr(response.usage, "cache_creation_input_tokens", None),
                )
            
            return response
    
    # Similar implementation for async version...
    
    # Monkey-patch
    anthropic.Anthropic.messages.create = traced_create
```

#### Usage Example

```python
# Example usage in an agent

import asyncio
from agenttrace import AgentTrace, trace, SpanType

async def main():
    # Initialize tracer
    tracer = AgentTrace(
        session_name="code-review-agent",
        framework="custom",
        tags=["production", "code-review"],
    )
    await tracer.start()
    
    try:
        # Automatic instrumentation captures Anthropic calls
        from anthropic import Anthropic
        client = Anthropic()
        
        # Manual instrumentation for custom logic
        with tracer.start_span("review_task", SpanType.TASK) as task_span:
            # Read file
            with tracer.start_span("read_source", SpanType.FILE_READ) as read_span:
                read_span.set_attribute("file.path", "src/main.py")
                content = open("src/main.py").read()
            
            # LLM call (auto-instrumented)
            response = client.messages.create(
                model="claude-sonnet-4-20250514",
                max_tokens=4096,
                messages=[{"role": "user", "content": f"Review this code:\n{content}"}],
            )
            
            # Process response
            with tracer.start_span("process_review", SpanType.REASONING):
                review = response.content[0].text
                # ... process review
    
    finally:
        await tracer.stop()

if __name__ == "__main__":
    asyncio.run(main())
```

---

## Data Models

### Database Schema (TimescaleDB)

```sql
-- migrations/001_initial_schema.sql

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    name TEXT,
    framework TEXT,
    framework_version TEXT,
    hostname TEXT,
    tags TEXT[],
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    total_spans BIGINT DEFAULT 0,
    total_tokens BIGINT DEFAULT 0,
    total_cost_usd DECIMAL(12, 6) DEFAULT 0,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_sessions_started_at ON sessions (started_at DESC);
CREATE INDEX idx_sessions_framework ON sessions (framework);
CREATE INDEX idx_sessions_tags ON sessions USING GIN (tags);

-- Traces table
CREATE TABLE traces (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    name TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_us BIGINT,
    status TEXT NOT NULL,
    total_spans INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    total_cost_usd DECIMAL(12, 6) DEFAULT 0,
    root_span_id UUID,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_traces_session_id ON traces (session_id);
CREATE INDEX idx_traces_started_at ON traces (started_at DESC);
CREATE INDEX idx_traces_status ON traces (status);

-- Spans table (hypertable for time-series optimization)
CREATE TABLE spans (
    id UUID NOT NULL,
    trace_id UUID NOT NULL,
    parent_span_id UUID,
    session_id UUID NOT NULL,
    name TEXT NOT NULL,
    span_type TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_us BIGINT,
    status TEXT NOT NULL,
    status_message TEXT,
    
    -- Token usage
    input_tokens INT,
    output_tokens INT,
    total_tokens INT,
    cache_read_tokens INT,
    cache_creation_tokens INT,
    
    -- Cost
    total_cost_usd DECIMAL(12, 6),
    input_cost_usd DECIMAL(12, 6),
    output_cost_usd DECIMAL(12, 6),
    
    -- Attributes (denormalized for query performance)
    model TEXT,
    provider TEXT,
    tool_name TEXT,
    file_path TEXT,
    
    -- Full attributes as JSONB
    attributes JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps for TimescaleDB
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (id, start_time)
);

-- Convert to hypertable for time-series optimization
SELECT create_hypertable('spans', 'start_time', chunk_time_interval => INTERVAL '1 day');

-- Indexes for common queries
CREATE INDEX idx_spans_trace_id ON spans (trace_id, start_time DESC);
CREATE INDEX idx_spans_session_id ON spans (session_id, start_time DESC);
CREATE INDEX idx_spans_span_type ON spans (span_type, start_time DESC);
CREATE INDEX idx_spans_model ON spans (model, start_time DESC) WHERE model IS NOT NULL;
CREATE INDEX idx_spans_tool_name ON spans (tool_name, start_time DESC) WHERE tool_name IS NOT NULL;
CREATE INDEX idx_spans_status ON spans (status, start_time DESC);

-- Continuous aggregates for dashboard metrics
CREATE MATERIALIZED VIEW hourly_metrics
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', start_time) AS bucket,
    session_id,
    span_type,
    model,
    COUNT(*) AS span_count,
    SUM(total_tokens) AS total_tokens,
    SUM(total_cost_usd) AS total_cost_usd,
    AVG(duration_us) AS avg_duration_us,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_us) AS p95_duration_us,
    COUNT(*) FILTER (WHERE status = 'error') AS error_count
FROM spans
GROUP BY bucket, session_id, span_type, model
WITH NO DATA;

-- Refresh policy for continuous aggregate
SELECT add_continuous_aggregate_policy('hourly_metrics',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

-- Daily aggregates for cost tracking
CREATE MATERIALIZED VIEW daily_costs
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', start_time) AS bucket,
    session_id,
    model,
    provider,
    SUM(input_tokens) AS total_input_tokens,
    SUM(output_tokens) AS total_output_tokens,
    SUM(total_cost_usd) AS total_cost_usd,
    COUNT(*) AS llm_call_count
FROM spans
WHERE span_type = 'llm_call'
GROUP BY bucket, session_id, model, provider
WITH NO DATA;

SELECT add_continuous_aggregate_policy('daily_costs',
    start_offset => INTERVAL '2 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day');

-- Compression policy for older data
SELECT add_compression_policy('spans', INTERVAL '7 days');

-- Retention policy (optional, configurable)
-- SELECT add_retention_policy('spans', INTERVAL '90 days');
```

---

## API Design

### REST API (FastAPI)

```python
# api/main.py

from fastapi import FastAPI, Query, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime, timedelta
from typing import Optional
import asyncpg

app = FastAPI(
    title="AgentTrace API",
    version="0.1.0",
    description="REST API for AgentTrace observability platform"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# --- Sessions ---

@app.get("/api/v1/sessions")
async def list_sessions(
    limit: int = Query(50, le=100),
    offset: int = 0,
    framework: Optional[str] = None,
    since: Optional[datetime] = None,
):
    """List all sessions with optional filtering."""
    pass

@app.get("/api/v1/sessions/{session_id}")
async def get_session(session_id: str):
    """Get session details including summary statistics."""
    pass

@app.get("/api/v1/sessions/{session_id}/traces")
async def get_session_traces(
    session_id: str,
    limit: int = Query(50, le=100),
    offset: int = 0,
):
    """Get all traces for a session."""
    pass

# --- Traces ---

@app.get("/api/v1/traces/{trace_id}")
async def get_trace(trace_id: str):
    """Get trace details with all spans."""
    pass

@app.get("/api/v1/traces/{trace_id}/spans")
async def get_trace_spans(trace_id: str):
    """Get all spans for a trace in hierarchical order."""
    pass

@app.get("/api/v1/traces/{trace_id}/waterfall")
async def get_trace_waterfall(trace_id: str):
    """Get trace data formatted for waterfall visualization."""
    pass

# --- Spans ---

@app.get("/api/v1/spans")
async def query_spans(
    session_id: Optional[str] = None,
    trace_id: Optional[str] = None,
    span_type: Optional[str] = None,
    model: Optional[str] = None,
    status: Optional[str] = None,
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
    limit: int = Query(100, le=1000),
    offset: int = 0,
):
    """Query spans with flexible filtering."""
    pass

@app.get("/api/v1/spans/{span_id}")
async def get_span(span_id: str):
    """Get full span details including all attributes."""
    pass

# --- Metrics ---

@app.get("/api/v1/metrics/summary")
async def get_metrics_summary(
    session_id: Optional[str] = None,
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
):
    """Get summary metrics (total tokens, cost, spans, etc.)."""
    pass

@app.get("/api/v1/metrics/costs")
async def get_cost_metrics(
    session_id: Optional[str] = None,
    group_by: str = Query("day", regex="^(hour|day|week|model|provider)$"),
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
):
    """Get cost metrics with flexible grouping."""
    pass

@app.get("/api/v1/metrics/performance")
async def get_performance_metrics(
    session_id: Optional[str] = None,
    span_type: Optional[str] = None,
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
):
    """Get performance metrics (latency percentiles, throughput)."""
    pass

@app.get("/api/v1/metrics/errors")
async def get_error_metrics(
    session_id: Optional[str] = None,
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
):
    """Get error rate and failure patterns."""
    pass

# --- Real-time ---

@app.websocket("/api/v1/ws/spans")
async def websocket_spans(websocket):
    """WebSocket endpoint for real-time span streaming."""
    pass

# --- Export ---

@app.get("/api/v1/export/spans")
async def export_spans(
    format: str = Query("json", regex="^(json|csv|parquet)$"),
    session_id: Optional[str] = None,
    since: Optional[datetime] = None,
    until: Optional[datetime] = None,
):
    """Export spans in various formats."""
    pass
```

---

## CLI Design

```
agenttrace - Observability for AI Agents

USAGE:
    agenttrace <COMMAND>

COMMANDS:
    up          Start the AgentTrace stack (collector + dashboard)
    down        Stop the AgentTrace stack
    status      Show status of AgentTrace services
    logs        View logs from collector or dashboard
    
    sessions    List and manage sessions
    traces      Query and inspect traces
    spans       Query spans
    
    metrics     View metrics and cost summaries
    export      Export data to JSON/CSV/Parquet
    
    config      Manage configuration
    version     Show version information
    help        Print help

EXAMPLES:
    # Start AgentTrace with default settings
    agenttrace up
    
    # Start with custom port
    agenttrace up --port 4318
    
    # View recent sessions
    agenttrace sessions list --limit 10
    
    # Get cost summary for today
    agenttrace metrics costs --since today
    
    # Stream spans in real-time
    agenttrace spans stream --session abc123
    
    # Export session data
    agenttrace export --session abc123 --format parquet -o data.parquet
```

### CLI Implementation

```rust
// cli/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agenttrace")]
#[command(about = "Observability for AI Agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the AgentTrace stack
    Up {
        /// Port for the collector gRPC server
        #[arg(long, default_value = "4317")]
        grpc_port: u16,
        
        /// Port for the HTTP API
        #[arg(long, default_value = "8080")]
        http_port: u16,
        
        /// Port for the dashboard
        #[arg(long, default_value = "3000")]
        dashboard_port: u16,
        
        /// Run in background
        #[arg(short, long)]
        detach: bool,
    },
    
    /// Stop the AgentTrace stack
    Down,
    
    /// Show status
    Status,
    
    /// View logs
    Logs {
        /// Service to view logs for
        #[arg(default_value = "all")]
        service: String,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Session commands
    Sessions {
        #[command(subcommand)]
        command: SessionCommands,
    },
    
    /// Trace commands
    Traces {
        #[command(subcommand)]
        command: TraceCommands,
    },
    
    /// Span commands
    Spans {
        #[command(subcommand)]
        command: SpanCommands,
    },
    
    /// Metrics commands
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
    
    /// Export data
    Export {
        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
        
        /// Session ID to export
        #[arg(long)]
        session: Option<String>,
        
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// List sessions
    List {
        #[arg(long, default_value = "10")]
        limit: u32,
    },
    
    /// Show session details
    Show { session_id: String },
    
    /// Delete a session
    Delete { session_id: String },
}

#[derive(Subcommand)]
enum TraceCommands {
    /// List traces
    List {
        #[arg(long)]
        session: Option<String>,
        
        #[arg(long, default_value = "10")]
        limit: u32,
    },
    
    /// Show trace details
    Show { trace_id: String },
    
    /// Show trace as tree
    Tree { trace_id: String },
}

#[derive(Subcommand)]
enum SpanCommands {
    /// Query spans
    Query {
        #[arg(long)]
        session: Option<String>,
        
        #[arg(long)]
        trace: Option<String>,
        
        #[arg(long)]
        span_type: Option<String>,
        
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    
    /// Stream spans in real-time
    Stream {
        #[arg(long)]
        session: Option<String>,
    },
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// Show summary metrics
    Summary {
        #[arg(long)]
        session: Option<String>,
    },
    
    /// Show cost breakdown
    Costs {
        #[arg(long)]
        session: Option<String>,
        
        #[arg(long, default_value = "day")]
        group_by: String,
        
        #[arg(long)]
        since: Option<String>,
    },
    
    /// Show performance metrics
    Performance {
        #[arg(long)]
        session: Option<String>,
    },
}
```

---

## Dashboard Design

### Component Structure

```
dashboard/
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx                    # Overview/Home
│   │   ├── sessions/
│   │   │   ├── page.tsx                # Session list
│   │   │   └── [id]/
│   │   │       └── page.tsx            # Session detail
│   │   ├── traces/
│   │   │   ├── page.tsx                # Trace search
│   │   │   └── [id]/
│   │   │       └── page.tsx            # Trace waterfall
│   │   ├── metrics/
│   │   │   ├── page.tsx                # Metrics overview
│   │   │   ├── costs/page.tsx          # Cost analysis
│   │   │   └── performance/page.tsx    # Performance analysis
│   │   └── settings/
│   │       └── page.tsx                # Configuration
│   ├── components/
│   │   ├── ui/                         # Radix-based primitives
│   │   ├── charts/
│   │   │   ├── CostChart.tsx
│   │   │   ├── LatencyChart.tsx
│   │   │   ├── TokenUsageChart.tsx
│   │   │   └── SpanTypeDistribution.tsx
│   │   ├── traces/
│   │   │   ├── TraceWaterfall.tsx
│   │   │   ├── SpanTree.tsx
│   │   │   └── SpanDetail.tsx
│   │   ├── sessions/
│   │   │   ├── SessionCard.tsx
│   │   │   └── SessionList.tsx
│   │   └── common/
│   │       ├── Header.tsx
│   │       ├── Sidebar.tsx
│   │       └── SearchBar.tsx
│   ├── hooks/
│   │   ├── useTraces.ts
│   │   ├── useSessions.ts
│   │   ├── useMetrics.ts
│   │   └── useRealtime.ts
│   ├── lib/
│   │   ├── api.ts                      # API client
│   │   └── utils.ts
│   └── stores/
│       └── app.ts                      # Zustand store
```

### Key Views

#### 1. Overview Dashboard

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  AgentTrace                                    [Search...]        [Settings]│
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ Total Cost  │ │ Total Tokens│ │ Avg Latency │ │ Error Rate  │           │
│  │   $12.47    │ │   1.2M      │ │   2.3s      │ │    0.8%     │           │
│  │  ↑ 15% /24h │ │  ↑ 23% /24h │ │  ↓ 5% /24h  │ │  ↓ 12% /24h │           │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘           │
│                                                                             │
│  ┌─────────────────────────────────────┐ ┌─────────────────────────────────┐│
│  │ Token Usage (7 days)                │ │ Cost by Model                   ││
│  │                                     │ │                                 ││
│  │     ╭────────────────────╮          │ │  claude-opus-4    ████████ 67% ││
│  │    ╱                      ╲         │ │  claude-sonnet-4  ███      28% ││
│  │   ╱                        ╲        │ │  gpt-4o           █         5% ││
│  │  ╱                          ────    │ │                                 ││
│  │ ╱                                   │ │                                 ││
│  └─────────────────────────────────────┘ └─────────────────────────────────┘│
│                                                                             │
│  Recent Sessions                                                            │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ Session                    │ Duration │ Tokens  │ Cost   │ Status   │  │
│  ├──────────────────────────────────────────────────────────────────────┤  │
│  │ code-review-agent-abc123   │ 5m 23s   │ 45,231  │ $0.89  │ ● Active │  │
│  │ research-assistant-def456  │ 12m 07s  │ 128,456 │ $2.34  │ ✓ Done   │  │
│  │ bug-fix-agent-ghi789       │ 3m 45s   │ 22,100  │ $0.45  │ ✗ Error  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 2. Trace Waterfall View

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  ← Back    Trace: Refactor Authentication Module                            │
│            Session: code-review-agent-abc123                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Duration: 2m 34s    Tokens: 45,231    Cost: $0.89    Spans: 47             │
│                                                                             │
│  Timeline                                                    0s  30s  60s   │
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │                                                                        ││
│  │ ▼ task: refactor_auth                                                  ││
│  │ │████████████████████████████████████████████████████████████████████│││
│  │ │                                                                      ││
│  │ ├─ file_read: src/auth/login.ts                                       ││
│  │ │ │██│                                                                 ││
│  │ │                                                                      ││
│  │ ├─ llm_call: analyze_code (claude-opus-4)                             ││
│  │ │ │    │████████████████│  tokens: 12,456  cost: $0.24                ││
│  │ │                                                                      ││
│  │ ├─ reasoning: plan_refactor                                           ││
│  │ │ │         │█████│                                                   ││
│  │ │                                                                      ││
│  │ ├─ file_write: src/auth/jwt.ts                                        ││
│  │ │ │              │██│                                                 ││
│  │ │                                                                      ││
│  │ ├─ llm_call: generate_tests (claude-sonnet-4)                         ││
│  │ │ │                │████████│  tokens: 8,234  cost: $0.12            ││
│  │ │                                                                      ││
│  │ └─ verification: run_tests                                            ││
│  │   │                         │████████████│  ✓ passed                  ││
│  │                                                                        ││
│  └────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
│  Selected Span: llm_call: analyze_code                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │ Model: claude-opus-4-20250101                                          ││
│  │ Provider: anthropic                                                    ││
│  │ Duration: 12.4s                                                        ││
│  │ Input Tokens: 8,456  |  Output Tokens: 4,000                          ││
│  │ Cost: $0.24                                                            ││
│  │ Temperature: 0.7  |  Max Tokens: 4096                                 ││
│  │                                                                        ││
│  │ Prompt Preview:                                                        ││
│  │ ┌────────────────────────────────────────────────────────────────────┐││
│  │ │ You are a senior software engineer. Analyze this authentication   │││
│  │ │ code and identify areas for improvement...                         │││
│  │ └────────────────────────────────────────────────────────────────────┘││
│  │                                                                        ││
│  │ Response Preview:                                                      ││
│  │ ┌────────────────────────────────────────────────────────────────────┐││
│  │ │ I've analyzed the authentication code. Here are my findings:       │││
│  │ │ 1. JWT handling should be extracted to a separate module...        │││
│  │ └────────────────────────────────────────────────────────────────────┘││
│  └────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Deployment

### Docker Compose (Development)

```yaml
# docker-compose.yml

version: '3.8'

services:
  collector:
    build:
      context: ./collector
      dockerfile: Dockerfile
    ports:
      - "4317:4317"   # gRPC
      - "4318:4318"   # HTTP
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://agenttrace:agenttrace@timescaledb:5432/agenttrace
      - REDIS_URL=redis://redis:6379
    depends_on:
      - timescaledb
      - redis
    volumes:
      - ./config:/etc/agenttrace

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

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  timescale_data:
  redis_data:
```

### Single Binary Distribution

```rust
// Build configuration for single binary with embedded assets

// Embed dashboard assets
#[derive(RustEmbed)]
#[folder = "dashboard/dist/"]
struct DashboardAssets;

// Embed migrations
#[derive(RustEmbed)]
#[folder = "migrations/"]
struct Migrations;
```

---

## Development Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Set up monorepo structure
- [ ] Implement core Span model in Rust
- [ ] Create gRPC server with basic ingestion
- [ ] Set up TimescaleDB schema and migrations
- [ ] Basic Python SDK with manual instrumentation
- [ ] Simple console exporter for testing

### Phase 2: Core Collector (Weeks 3-4)
- [ ] Processing pipeline (decode → enrich → store)
- [ ] Cost calculation for major LLM providers
- [ ] Redis streaming for real-time updates
- [ ] UDP receiver for high-throughput scenarios
- [ ] Batch processing and buffering

### Phase 3: Python SDK (Weeks 5-6)
- [ ] Context managers and decorators
- [ ] Anthropic auto-instrumentation
- [ ] OpenAI auto-instrumentation
- [ ] Async support
- [ ] gRPC exporter with retry logic

### Phase 4: API & Dashboard (Weeks 7-8)
- [ ] FastAPI REST endpoints
- [ ] WebSocket for real-time streaming
- [ ] React dashboard with core views
- [ ] Trace waterfall visualization
- [ ] Cost and metrics charts

### Phase 5: CLI & Polish (Weeks 9-10)
- [ ] Full CLI implementation
- [ ] TUI with Ratatui
- [ ] Docker compose setup
- [ ] Documentation
- [ ] Example integrations

### Phase 6: Advanced Features (Future)
- [ ] LangChain integration
- [ ] Anomaly detection
- [ ] Alerting system
- [ ] Team/organization support
- [ ] Cloud sync (optional)

---

## Success Criteria

1. **Performance**: < 1ms overhead per span
2. **Reliability**: Zero data loss under normal operation
3. **Usability**: Working trace in < 5 minutes from install
4. **Compatibility**: Works with Anthropic, OpenAI, LangChain out of box
5. **Visualization**: Beautiful, responsive dashboard

---

## License

MIT License - Open source, attribution required.

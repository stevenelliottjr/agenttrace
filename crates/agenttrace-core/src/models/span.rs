//! Span data model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpanStatus {
    /// Operation completed successfully
    #[default]
    Ok,
    /// Operation failed
    Error,
    /// Status not set
    Unset,
}

/// Kind of span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpanKind {
    /// Internal operation
    #[default]
    Internal,
    /// Client-side operation
    Client,
    /// Server-side operation
    Server,
    /// Producer in messaging
    Producer,
    /// Consumer in messaging
    Consumer,
}

/// A span represents a single operation within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique identifier
    pub id: Uuid,

    /// Span ID (32-char hex)
    pub span_id: String,

    /// Trace ID this span belongs to
    pub trace_id: String,

    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,

    /// Name of the operation
    pub operation_name: String,

    /// Service that generated this span
    pub service_name: String,

    /// Kind of span
    pub span_kind: SpanKind,

    /// When the operation started
    pub started_at: DateTime<Utc>,

    /// When the operation ended (if completed)
    pub ended_at: Option<DateTime<Utc>>,

    /// Duration in milliseconds
    pub duration_ms: Option<f64>,

    /// Status of the operation
    pub status: SpanStatus,

    /// Status message (usually for errors)
    pub status_message: Option<String>,

    // AI-specific fields
    /// Model name (e.g., "gpt-4o", "claude-3-5-sonnet")
    pub model_name: Option<String>,

    /// Model provider (e.g., "openai", "anthropic")
    pub model_provider: Option<String>,

    /// Input tokens
    pub tokens_in: Option<i32>,

    /// Output tokens
    pub tokens_out: Option<i32>,

    /// Reasoning tokens (for o1-style models)
    pub tokens_reasoning: Option<i32>,

    /// Cost in USD
    pub cost_usd: Option<f64>,

    // Tool usage
    /// Tool name if this span represents a tool call
    pub tool_name: Option<String>,

    /// Tool input parameters
    pub tool_input: Option<serde_json::Value>,

    /// Tool output
    pub tool_output: Option<serde_json::Value>,

    /// Tool execution duration
    pub tool_duration_ms: Option<f64>,

    // Content previews
    /// First 500 chars of prompt
    pub prompt_preview: Option<String>,

    /// First 500 chars of completion
    pub completion_preview: Option<String>,

    /// Additional attributes
    pub attributes: serde_json::Value,

    /// Events that occurred during the span
    pub events: Vec<SpanEvent>,

    /// Links to other spans
    pub links: Vec<SpanLink>,
}

/// An event that occurred during a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,

    /// When the event occurred
    pub timestamp: DateTime<Utc>,

    /// Event attributes
    pub attributes: serde_json::Value,
}

/// A link to another span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    /// Trace ID of the linked span
    pub trace_id: String,

    /// Span ID of the linked span
    pub span_id: String,

    /// Link attributes
    pub attributes: serde_json::Value,
}

/// Input for creating a new span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInput {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service_name: Option<String>,
    pub span_kind: Option<SpanKind>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: Option<SpanStatus>,
    pub status_message: Option<String>,
    pub model_name: Option<String>,
    pub model_provider: Option<String>,
    pub tokens_in: Option<i32>,
    pub tokens_out: Option<i32>,
    pub tokens_reasoning: Option<i32>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<serde_json::Value>,
    pub prompt_preview: Option<String>,
    pub completion_preview: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub events: Option<Vec<SpanEvent>>,
}

impl Span {
    /// Calculate duration from start and end times
    pub fn calculate_duration(&mut self) {
        if let Some(ended_at) = self.ended_at {
            let duration = ended_at - self.started_at;
            self.duration_ms = Some(duration.num_milliseconds() as f64);
        }
    }

    /// Check if this span represents an LLM call
    pub fn is_llm_call(&self) -> bool {
        self.model_name.is_some()
    }

    /// Check if this span represents a tool call
    pub fn is_tool_call(&self) -> bool {
        self.tool_name.is_some()
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> i32 {
        self.tokens_in.unwrap_or(0)
            + self.tokens_out.unwrap_or(0)
            + self.tokens_reasoning.unwrap_or(0)
    }
}

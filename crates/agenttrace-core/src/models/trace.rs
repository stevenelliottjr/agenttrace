//! Trace data model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    /// Trace completed successfully
    Ok,
    /// Trace had errors
    Error,
    /// Trace is still in progress
    #[default]
    InProgress,
}

/// A trace represents a complete request/operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Unique identifier
    pub id: Uuid,

    /// Trace ID (32-char hex)
    pub trace_id: String,

    /// Service that initiated this trace
    pub service_name: String,

    /// Environment (production, staging, etc.)
    pub environment: String,

    /// When the trace started
    pub started_at: DateTime<Utc>,

    /// When the trace ended (if completed)
    pub ended_at: Option<DateTime<Utc>>,

    /// Total duration in milliseconds
    pub duration_ms: Option<f64>,

    /// Overall status
    pub status: TraceStatus,

    /// Root span ID
    pub root_span_id: Option<String>,

    // Aggregated metrics
    /// Total input tokens across all spans
    pub total_tokens_in: i32,

    /// Total output tokens across all spans
    pub total_tokens_out: i32,

    /// Total cost in USD
    pub total_cost_usd: f64,

    /// Number of errors
    pub error_count: i32,

    /// Number of spans
    pub span_count: i32,

    /// Additional metadata
    pub metadata: serde_json::Value,

    /// Tags for filtering
    pub tags: Vec<String>,

    /// When this record was created
    pub created_at: DateTime<Utc>,

    /// When this record was last updated
    pub updated_at: DateTime<Utc>,
}

/// A trace with all its spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDetail {
    /// The trace
    #[serde(flatten)]
    pub trace: Trace,

    /// All spans in this trace
    pub spans: Vec<super::Span>,
}

/// Query parameters for listing traces
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceQuery {
    /// Filter by service name
    pub service_name: Option<String>,

    /// Filter by environment
    pub environment: Option<String>,

    /// Filter by status
    pub status: Option<TraceStatus>,

    /// Filter by minimum duration
    pub min_duration_ms: Option<f64>,

    /// Filter by maximum duration
    pub max_duration_ms: Option<f64>,

    /// Filter by minimum cost
    pub min_cost: Option<f64>,

    /// Filter by start time (traces started after this time)
    pub start_time: Option<DateTime<Utc>>,

    /// Filter by end time (traces started before this time)
    pub end_time: Option<DateTime<Utc>>,

    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,

    /// Search in metadata
    pub metadata_query: Option<serde_json::Value>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Sort field
    pub sort_by: Option<String>,

    /// Sort direction (asc or desc)
    pub sort_order: Option<String>,
}

impl Trace {
    /// Get total tokens across the trace
    pub fn total_tokens(&self) -> i32 {
        self.total_tokens_in + self.total_tokens_out
    }

    /// Check if the trace has any errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if the trace is complete
    pub fn is_complete(&self) -> bool {
        self.status != TraceStatus::InProgress
    }
}

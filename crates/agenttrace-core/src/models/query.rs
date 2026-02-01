//! Query and response types shared between API and database layers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Search filter for advanced queries
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchFilter {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

/// Sort configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SortConfig {
    pub field: String,
    pub descending: bool,
}

/// Trace summary
#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub trace_id: String,
    pub root_operation: String,
    pub service_name: String,
    pub started_at: DateTime<Utc>,
    pub duration_ms: Option<f64>,
    pub span_count: i64,
    pub error_count: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
}

/// Summary metrics response
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummaryResponse {
    pub total_spans: i64,
    pub total_traces: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub error_count: i64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

/// Cost metrics by group
#[derive(Debug, Clone, Serialize)]
pub struct CostMetric {
    pub group: String,
    pub total_cost_usd: f64,
    pub total_tokens: i64,
    pub call_count: i64,
}

/// Latency metrics over time
#[derive(Debug, Clone, Serialize)]
pub struct LatencyMetric {
    pub timestamp: DateTime<Utc>,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub count: i64,
}

/// Error metrics over time
#[derive(Debug, Clone, Serialize)]
pub struct ErrorMetric {
    pub timestamp: DateTime<Utc>,
    pub error_count: i64,
    pub total_count: i64,
    pub error_rate: f64,
}

/// Error statistics for alerting
#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub error_count: i64,
    pub total: i64,
    pub sample_trace_ids: Vec<String>,
}

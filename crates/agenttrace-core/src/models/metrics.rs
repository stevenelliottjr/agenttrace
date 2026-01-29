//! Metrics data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregated metrics for a time bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsBucket {
    /// Start of the time bucket
    pub timestamp: DateTime<Utc>,

    /// Service name
    pub service_name: String,

    /// Model name (if applicable)
    pub model_name: Option<String>,

    /// Model provider (if applicable)
    pub model_provider: Option<String>,

    /// Operation name (if applicable)
    pub operation_name: Option<String>,

    // Counters
    /// Total request count
    pub request_count: i64,

    /// Error count
    pub error_count: i64,

    /// Tool call count
    pub tool_call_count: i64,

    // Token metrics
    /// Sum of input tokens
    pub tokens_in_sum: i64,

    /// Sum of output tokens
    pub tokens_out_sum: i64,

    /// Average input tokens per request
    pub tokens_in_avg: Option<f64>,

    /// Average output tokens per request
    pub tokens_out_avg: Option<f64>,

    // Cost metrics
    /// Total cost
    pub cost_sum: f64,

    /// Average cost per request
    pub cost_avg: Option<f64>,

    // Latency metrics
    /// Average latency in milliseconds
    pub latency_avg_ms: Option<f64>,

    /// Minimum latency
    pub latency_min_ms: Option<f64>,

    /// Maximum latency
    pub latency_max_ms: Option<f64>,

    /// 50th percentile latency
    pub latency_p50_ms: Option<f64>,

    /// 90th percentile latency
    pub latency_p90_ms: Option<f64>,

    /// 95th percentile latency
    pub latency_p95_ms: Option<f64>,

    /// 99th percentile latency
    pub latency_p99_ms: Option<f64>,
}

impl MetricsBucket {
    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.request_count as f64
        }
    }

    /// Calculate total tokens
    pub fn total_tokens(&self) -> i64 {
        self.tokens_in_sum + self.tokens_out_sum
    }
}

/// Query parameters for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQuery {
    /// Filter by service name
    pub service_name: Option<String>,

    /// Filter by model name
    pub model_name: Option<String>,

    /// Filter by model provider
    pub model_provider: Option<String>,

    /// Filter by operation name
    pub operation_name: Option<String>,

    /// Start time for the query
    pub start_time: DateTime<Utc>,

    /// End time for the query
    pub end_time: DateTime<Utc>,

    /// Granularity (1m, 5m, 15m, 1h, 1d)
    pub granularity: String,
}

/// Response containing metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// The query that was executed
    pub query: MetricsQuery,

    /// Metric buckets
    pub buckets: Vec<MetricsBucket>,

    /// Summary statistics
    pub summary: MetricsSummary,
}

/// Summary statistics for a metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total requests
    pub total_requests: i64,

    /// Total errors
    pub total_errors: i64,

    /// Total tokens
    pub total_tokens: i64,

    /// Total cost
    pub total_cost: f64,

    /// Overall error rate
    pub error_rate: f64,

    /// Average latency
    pub avg_latency_ms: f64,
}

/// Cost breakdown by dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Total cost
    pub total_cost: f64,

    /// Cost breakdown by the grouped dimension
    pub breakdown: Vec<CostBreakdownItem>,
}

/// Individual item in a cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdownItem {
    /// The key (model name, service name, etc.)
    pub key: String,

    /// Cost for this item
    pub cost: f64,

    /// Percentage of total
    pub percentage: f64,

    /// Input tokens
    pub tokens_in: i64,

    /// Output tokens
    pub tokens_out: i64,

    /// Request count
    pub request_count: i64,
}

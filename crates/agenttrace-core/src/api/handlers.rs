//! API handlers for the HTTP REST API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use uuid::Uuid;

use crate::collector::Pipeline;
use crate::db::{RedisPool, SpanRepository};
use crate::models::{
    Span, SpanStatus, SpanKind,
    CostMetric, ErrorMetric, LatencyMetric, MetricsSummaryResponse,
    SearchFilter, SortConfig, TraceSummary,
};

use crate::alerting::{AlertEvaluator, AlertRepository};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<Pipeline>,
    pub span_repo: SpanRepository,
    pub redis: Option<RedisPool>,
    pub alert_repo: Option<AlertRepository>,
    pub alert_evaluator: Option<Arc<AlertEvaluator>>,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Span ingestion request
#[derive(Debug, Deserialize)]
pub struct IngestSpanRequest {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service_name: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<String>,
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
}

/// Span ingestion response
#[derive(Serialize)]
pub struct IngestSpanResponse {
    pub success: bool,
    pub span_id: String,
}

/// Ingest a single span
pub async fn ingest_span(
    State(state): State<AppState>,
    Json(req): Json<IngestSpanRequest>,
) -> Result<Json<IngestSpanResponse>, (StatusCode, String)> {
    let span = convert_request_to_span(req);
    let span_id = span.span_id.clone();

    state
        .pipeline
        .submit(span)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IngestSpanResponse {
        success: true,
        span_id,
    }))
}

/// Batch ingestion request
#[derive(Debug, Deserialize)]
pub struct IngestBatchRequest {
    pub spans: Vec<IngestSpanRequest>,
}

/// Batch ingestion response
#[derive(Serialize)]
pub struct IngestBatchResponse {
    pub accepted: usize,
    pub rejected: usize,
}

/// Ingest multiple spans
pub async fn ingest_batch(
    State(state): State<AppState>,
    Json(req): Json<IngestBatchRequest>,
) -> Result<Json<IngestBatchResponse>, (StatusCode, String)> {
    let total = req.spans.len();
    let spans: Vec<Span> = req.spans.into_iter().map(convert_request_to_span).collect();

    let accepted = state
        .pipeline
        .submit_batch(spans)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IngestBatchResponse {
        accepted,
        rejected: total - accepted,
    }))
}

/// Query parameters for listing spans
#[derive(Debug, Deserialize)]
pub struct ListSpansQuery {
    pub trace_id: Option<String>,
    pub service_name: Option<String>,
    pub limit: Option<i64>,
}

/// List spans response
#[derive(Serialize)]
pub struct ListSpansResponse {
    pub spans: Vec<Span>,
    pub total: usize,
}

/// List spans
pub async fn list_spans(
    State(state): State<AppState>,
    Query(query): Query<ListSpansQuery>,
) -> Result<Json<ListSpansResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(100);

    let spans = if let Some(trace_id) = query.trace_id {
        state
            .span_repo
            .get_by_trace_id(&trace_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .span_repo
            .get_recent(limit)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let total = spans.len();
    Ok(Json(ListSpansResponse { spans, total }))
}

/// Get a single span by ID
pub async fn get_span(
    State(state): State<AppState>,
    Path(span_id): Path<Uuid>,
) -> Result<Json<Span>, (StatusCode, String)> {
    let span = state
        .span_repo
        .get_by_id(&span_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Span not found".to_string()))?;

    Ok(Json(span))
}

fn convert_request_to_span(req: IngestSpanRequest) -> Span {
    let status = match req.status.as_deref() {
        Some("ok") => SpanStatus::Ok,
        Some("error") => SpanStatus::Error,
        _ => SpanStatus::Unset,
    };

    Span {
        id: Uuid::new_v4(),
        span_id: req.span_id,
        trace_id: req.trace_id,
        parent_span_id: req.parent_span_id,
        operation_name: req.operation_name,
        service_name: req.service_name.unwrap_or_else(|| "unknown".to_string()),
        span_kind: SpanKind::Internal,
        started_at: req.started_at,
        ended_at: req.ended_at,
        duration_ms: None,
        status,
        status_message: req.status_message,
        model_name: req.model_name,
        model_provider: req.model_provider,
        tokens_in: req.tokens_in,
        tokens_out: req.tokens_out,
        tokens_reasoning: req.tokens_reasoning,
        cost_usd: None,
        tool_name: req.tool_name,
        tool_input: req.tool_input,
        tool_output: req.tool_output,
        tool_duration_ms: None,
        prompt_preview: req.prompt_preview,
        completion_preview: req.completion_preview,
        attributes: req.attributes.unwrap_or_else(|| serde_json::json!({})),
        events: vec![],
        links: vec![],
    }
}

/// Query parameters for SSE stream
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    /// Filter by trace_id (optional)
    pub trace_id: Option<String>,
    /// Channel to subscribe to: "spans", "llm", or "trace:{id}"
    pub channel: Option<String>,
}

// ============================================================================
// Search Handlers
// ============================================================================

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Free-text search query
    pub q: Option<String>,
    /// Service name filter
    pub service: Option<String>,
    /// Model name filter
    pub model: Option<String>,
    /// Status filter (ok, error)
    pub status: Option<String>,
    /// Minimum duration in ms
    pub min_duration: Option<f64>,
    /// Maximum duration in ms
    pub max_duration: Option<f64>,
    /// Minimum cost in USD
    pub min_cost: Option<f64>,
    /// Maximum cost in USD
    pub max_cost: Option<f64>,
    /// Start time (ISO 8601)
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// End time (ISO 8601)
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort order (asc, desc)
    pub sort_order: Option<String>,
    /// Maximum results
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Search response
#[derive(Serialize)]
pub struct SearchResponse {
    pub spans: Vec<Span>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Search spans with filters
/// Search spans with filters
pub async fn search_spans(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let (spans, total) = state
        .span_repo
        .search(
            query.q.as_deref(),
            query.service.as_deref(),
            query.model.as_deref(),
            query.status.as_deref(),
            query.min_duration,
            query.max_duration,
            query.min_cost,
            query.max_cost,
            query.since,
            query.until,
            query.sort_by.as_deref().unwrap_or("started_at"),
            query.sort_order.as_deref().unwrap_or("desc") == "desc",
            limit,
            offset,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SearchResponse {
        spans,
        total,
        limit,
        offset,
    }))
}

/// Advanced search request
#[derive(Debug, Deserialize)]
pub struct AdvancedSearchRequest {
    /// Filter conditions (AND)
    pub filters: Vec<SearchFilter>,
    /// Sort configuration
    pub sort: Option<SortConfig>,
    /// Pagination
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Advanced search with complex filters
pub async fn advanced_search(
    State(state): State<AppState>,
    Json(req): Json<AdvancedSearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let limit = req.limit.unwrap_or(50).min(1000);
    let offset = req.offset.unwrap_or(0);

    let (spans, total) = state
        .span_repo
        .advanced_search(&req.filters, req.sort.as_ref(), limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SearchResponse {
        spans,
        total,
        limit,
        offset,
    }))
}

// ============================================================================
// Trace Handlers
// ============================================================================

/// List traces query
#[derive(Debug, Deserialize)]
pub struct ListTracesQuery {
    pub service: Option<String>,
    pub status: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct ListTracesResponse {
    pub traces: Vec<TraceSummary>,
    pub total: i64,
}

/// List traces
pub async fn list_traces(
    State(state): State<AppState>,
    Query(query): Query<ListTracesQuery>,
) -> Result<Json<ListTracesResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50);

    let traces = state
        .span_repo
        .list_traces(
            query.service.as_deref(),
            query.status.as_deref(),
            query.since,
            limit,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ListTracesResponse {
        total: traces.len() as i64,
        traces,
    }))
}

/// Get trace details
#[derive(Serialize)]
pub struct TraceDetail {
    pub trace_id: String,
    pub spans: Vec<Span>,
    pub summary: TraceSummary,
}

pub async fn get_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> Result<Json<TraceDetail>, (StatusCode, String)> {
    let spans = state
        .span_repo
        .get_by_trace_id(&trace_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if spans.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Trace not found".to_string()));
    }

    // Build summary
    let root = spans.iter().find(|s| s.parent_span_id.is_none());
    let error_count = spans.iter().filter(|s| s.status == SpanStatus::Error).count() as i64;
    let total_tokens: i64 = spans
        .iter()
        .filter_map(|s| s.tokens_in.map(|t| t as i64))
        .sum::<i64>()
        + spans
            .iter()
            .filter_map(|s| s.tokens_out.map(|t| t as i64))
            .sum::<i64>();
    let total_cost: f64 = spans.iter().filter_map(|s| s.cost_usd).sum();

    let summary = TraceSummary {
        trace_id: trace_id.clone(),
        root_operation: root.map(|s| s.operation_name.clone()).unwrap_or_default(),
        service_name: root.map(|s| s.service_name.clone()).unwrap_or_default(),
        started_at: root.map(|s| s.started_at).unwrap_or_else(chrono::Utc::now),
        duration_ms: root.and_then(|s| s.duration_ms),
        span_count: spans.len() as i64,
        error_count,
        total_tokens,
        total_cost_usd: total_cost,
    };

    Ok(Json(TraceDetail {
        trace_id,
        spans,
        summary,
    }))
}

/// Get spans for a trace
pub async fn get_trace_spans(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> Result<Json<Vec<Span>>, (StatusCode, String)> {
    let spans = state
        .span_repo
        .get_by_trace_id(&trace_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(spans))
}

// ============================================================================
// Metrics Handlers
// ============================================================================

/// Metrics query parameters
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub service: Option<String>,
    pub model: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub group_by: Option<String>,
}

pub async fn get_metrics_summary(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<MetricsSummaryResponse>, (StatusCode, String)> {
    let since = query
        .since
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::hours(1));
    let until = query.until.unwrap_or_else(chrono::Utc::now);

    let summary = state
        .span_repo
        .get_metrics_summary(query.service.as_deref(), query.model.as_deref(), since, until)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(summary))
}

#[derive(Serialize)]
pub struct CostMetricsResponse {
    pub costs: Vec<CostMetric>,
    pub total_cost_usd: f64,
}

pub async fn get_cost_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<CostMetricsResponse>, (StatusCode, String)> {
    let since = query
        .since
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(7));
    let until = query.until.unwrap_or_else(chrono::Utc::now);
    let group_by = query.group_by.as_deref().unwrap_or("model");

    let costs = state
        .span_repo
        .get_cost_by_group(query.service.as_deref(), group_by, since, until)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total: f64 = costs.iter().map(|c| c.total_cost_usd).sum();

    Ok(Json(CostMetricsResponse {
        costs,
        total_cost_usd: total,
    }))
}

#[derive(Serialize)]
pub struct LatencyMetricsResponse {
    pub metrics: Vec<LatencyMetric>,
}

pub async fn get_latency_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<LatencyMetricsResponse>, (StatusCode, String)> {
    let since = query
        .since
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::hours(24));
    let until = query.until.unwrap_or_else(chrono::Utc::now);

    let metrics = state
        .span_repo
        .get_latency_over_time(query.service.as_deref(), query.model.as_deref(), since, until)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LatencyMetricsResponse { metrics }))
}

#[derive(Serialize)]
pub struct ErrorMetricsResponse {
    pub metrics: Vec<ErrorMetric>,
    pub overall_error_rate: f64,
}

pub async fn get_error_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<ErrorMetricsResponse>, (StatusCode, String)> {
    let since = query
        .since
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::hours(24));
    let until = query.until.unwrap_or_else(chrono::Utc::now);

    let metrics = state
        .span_repo
        .get_errors_over_time(query.service.as_deref(), query.model.as_deref(), since, until)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_errors: i64 = metrics.iter().map(|m| m.error_count).sum();
    let total_count: i64 = metrics.iter().map(|m| m.total_count).sum();
    let overall_rate = if total_count > 0 {
        total_errors as f64 / total_count as f64 * 100.0
    } else {
        0.0
    };

    Ok(Json(ErrorMetricsResponse {
        metrics,
        overall_error_rate: overall_rate,
    }))
}

// ============================================================================
// Alert Handlers
// ============================================================================

use crate::models::alert::{AlertEvent, AlertRule, AlertRuleInput};

/// List alert rules
pub async fn list_alert_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRule>>, (StatusCode, String)> {
    let rules = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .list_rules()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rules))
}

/// Create alert rule
pub async fn create_alert_rule(
    State(state): State<AppState>,
    Json(input): Json<AlertRuleInput>,
) -> Result<(StatusCode, Json<AlertRule>), (StatusCode, String)> {
    let rule = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .create_rule(input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(rule)))
}

/// Get alert rule by ID
pub async fn get_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<AlertRule>, (StatusCode, String)> {
    let rule = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .get_rule(rule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Rule not found".to_string()))?;

    Ok(Json(rule))
}

/// Update alert rule
pub async fn update_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
    Json(input): Json<AlertRuleInput>,
) -> Result<Json<AlertRule>, (StatusCode, String)> {
    let rule = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .update_rule(rule_id, input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Rule not found".to_string()))?;

    Ok(Json(rule))
}

/// Delete alert rule
pub async fn delete_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let deleted = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .delete_rule(rule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Rule not found".to_string()))
    }
}

/// Test alert rule
#[derive(Serialize)]
pub struct TestAlertResponse {
    pub would_trigger: bool,
    pub event: Option<AlertEvent>,
    pub current_value: Option<f64>,
}

pub async fn test_alert_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<TestAlertResponse>, (StatusCode, String)> {
    let rule = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .get_rule(rule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Rule not found".to_string()))?;

    let evaluator = state
        .alert_evaluator
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alert evaluator not configured".to_string()))?;

    let event = evaluator
        .test_rule(&rule)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TestAlertResponse {
        would_trigger: event.is_some(),
        current_value: event.as_ref().map(|e| e.metric_value),
        event,
    }))
}

/// List alert events query
#[derive(Debug, Deserialize)]
pub struct ListAlertEventsQuery {
    pub rule_id: Option<Uuid>,
    pub status: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

/// List alert events
pub async fn list_alert_events(
    State(state): State<AppState>,
    Query(query): Query<ListAlertEventsQuery>,
) -> Result<Json<Vec<AlertEvent>>, (StatusCode, String)> {
    let repo = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?;

    let events = if query.status.as_deref() == Some("active") {
        repo.list_active_events().await
    } else if let Some(rule_id) = query.rule_id {
        repo.list_events_for_rule(rule_id, query.limit.unwrap_or(50)).await
    } else {
        let since = query
            .since
            .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(7));
        repo.list_recent_events(since, query.limit.unwrap_or(100)).await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(events))
}

/// Get alert event by ID
pub async fn get_alert_event(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<AlertEvent>, (StatusCode, String)> {
    let event = state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .get_event(event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Event not found".to_string()))?;

    Ok(Json(event))
}

/// Acknowledge an alert
pub async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .alert_repo
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Alerting not configured".to_string()))?
        .acknowledge_event(event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

/// SSE stream endpoint for real-time span updates
pub async fn stream_spans(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let redis = state
        .redis
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Redis not configured".to_string()))?;

    // Determine which channel to subscribe to
    let channel = if let Some(trace_id) = query.trace_id {
        format!("agenttrace:trace:{}", trace_id)
    } else {
        match query.channel.as_deref() {
            Some("llm") => "agenttrace:llm".to_string(),
            _ => "agenttrace:spans".to_string(),
        }
    };

    // Subscribe to the Redis channel
    let rx = redis
        .subscribe(&channel)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert the receiver into a stream of SSE events
    let stream = ReceiverStream::new(rx)
        .map(|payload| {
            Ok(Event::default()
                .event("span")
                .data(payload))
        })
        // Add a keepalive comment every 30 seconds
        .chain(tokio_stream::pending());

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keepalive"),
    ))
}

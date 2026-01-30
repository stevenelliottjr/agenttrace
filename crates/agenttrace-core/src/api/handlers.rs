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
use crate::models::{Span, SpanInput, SpanStatus, SpanKind, SpanEvent};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<Pipeline>,
    pub span_repo: SpanRepository,
    pub redis: Option<RedisPool>,
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

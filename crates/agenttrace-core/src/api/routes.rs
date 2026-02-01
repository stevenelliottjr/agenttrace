//! API routes

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::{self, AppState};

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(handlers::health))

        // Span ingestion
        .route("/api/v1/spans", post(handlers::ingest_span))
        .route("/api/v1/spans/batch", post(handlers::ingest_batch))

        // Span queries
        .route("/api/v1/spans", get(handlers::list_spans))
        .route("/api/v1/spans/:span_id", get(handlers::get_span))

        // Search
        .route("/api/v1/search", get(handlers::search_spans))
        .route("/api/v1/search/advanced", post(handlers::advanced_search))

        // Traces
        .route("/api/v1/traces", get(handlers::list_traces))
        .route("/api/v1/traces/:trace_id", get(handlers::get_trace))
        .route("/api/v1/traces/:trace_id/spans", get(handlers::get_trace_spans))

        // Metrics
        .route("/api/v1/metrics/summary", get(handlers::get_metrics_summary))
        .route("/api/v1/metrics/costs", get(handlers::get_cost_metrics))
        .route("/api/v1/metrics/latency", get(handlers::get_latency_metrics))
        .route("/api/v1/metrics/errors", get(handlers::get_error_metrics))

        // Alerts
        .route("/api/v1/alerts/rules", get(handlers::list_alert_rules))
        .route("/api/v1/alerts/rules", post(handlers::create_alert_rule))
        .route("/api/v1/alerts/rules/:rule_id", get(handlers::get_alert_rule))
        .route("/api/v1/alerts/rules/:rule_id", put(handlers::update_alert_rule))
        .route("/api/v1/alerts/rules/:rule_id", delete(handlers::delete_alert_rule))
        .route("/api/v1/alerts/rules/:rule_id/test", post(handlers::test_alert_rule))
        .route("/api/v1/alerts/events", get(handlers::list_alert_events))
        .route("/api/v1/alerts/events/:event_id", get(handlers::get_alert_event))
        .route("/api/v1/alerts/events/:event_id/acknowledge", post(handlers::acknowledge_alert))

        // Real-time streaming
        .route("/api/v1/stream", get(handlers::stream_spans))

        .with_state(state)
}

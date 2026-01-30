//! API routes

use axum::{
    routing::{get, post},
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
        // Real-time streaming
        .route("/api/v1/stream", get(handlers::stream_spans))
        .with_state(state)
}

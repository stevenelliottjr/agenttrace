//! REST API implementation
//!
//! This module provides the HTTP API for AgentTrace.

pub mod handlers;
pub mod middleware;
pub mod routes;

pub use handlers::AppState;
pub use routes::create_router;

use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::alerting::{AlertEvaluator, AlertRepository};
use crate::collector::Pipeline;
use crate::db::{RedisPool, SpanRepository};
use crate::error::Result;

/// HTTP API server
pub struct HttpServer {
    state: AppState,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(
        pipeline: Arc<Pipeline>,
        span_repo: SpanRepository,
        redis: Option<RedisPool>,
        alert_repo: Option<AlertRepository>,
        alert_evaluator: Option<Arc<AlertEvaluator>>,
    ) -> Self {
        Self {
            state: AppState {
                pipeline,
                span_repo,
                redis,
                alert_repo,
                alert_evaluator,
            },
        }
    }

    /// Start the HTTP server
    pub async fn serve(self, addr: &str) -> Result<()> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = create_router(self.state).layer(cors);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| crate::error::Error::Internal(e.to_string()))?;

        info!("HTTP server listening on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::Error::Internal(e.to_string()))?;

        Ok(())
    }
}

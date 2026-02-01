//! Collector module - High-performance telemetry ingestion
//!
//! The collector receives spans via gRPC, HTTP, and UDP, processes them through
//! a pipeline, and stores them in TimescaleDB while streaming to Redis.

mod cost;
mod grpc;
mod pipeline;

pub use cost::CostCalculator;
pub use grpc::GrpcServer;
pub use pipeline::{Pipeline, PipelineConfig};

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, warn};

use crate::api::HttpServer;
use crate::config::Config;
use crate::db::{Database, SpanRepository};
use crate::error::Result;
use crate::models::Span;

/// The main collector service
pub struct Collector {
    config: Config,
    db: Database,
    pipeline: Arc<Pipeline>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl Collector {
    /// Create a new collector instance
    pub async fn new(config: Config) -> Result<Self> {
        let db = Database::new(&config).await?;

        let pipeline_config = PipelineConfig {
            batch_size: config.collector.batch_size,
            batch_timeout_ms: config.collector.batch_timeout_ms,
            enable_cost_calculation: true,
            enable_redis_streaming: true,
        };

        let pipeline = Arc::new(Pipeline::new(pipeline_config, db.clone()));

        Ok(Self {
            config,
            db,
            pipeline,
            shutdown_tx: None,
        })
    }

    /// Start the collector (HTTP + gRPC servers + pipeline)
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting AgentTrace collector...");

        // Health check databases
        self.db.health_check().await?;
        info!("Database connections healthy");

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start the processing pipeline
        let pipeline = self.pipeline.clone();
        let pipeline_handle = tokio::spawn(async move {
            pipeline.start().await;
        });

        // Start HTTP server
        let http_addr = format!("{}:{}", self.config.server.host, self.config.server.http_port);
        let span_repo = SpanRepository::new(&self.db.postgres);
        let redis_pool = Some(self.db.redis.clone());
        let http_server = HttpServer::new(self.pipeline.clone(), span_repo, redis_pool, None, None);

        info!("Starting HTTP server on {}", http_addr);

        let http_handle = tokio::spawn(async move {
            if let Err(e) = http_server.serve(&http_addr).await {
                error!("HTTP server error: {}", e);
            }
        });

        // Start gRPC server (optional, may fail with skeleton impl)
        let grpc_addr = format!("{}:{}", self.config.server.host, self.config.server.grpc_port);
        let grpc_server = GrpcServer::new(self.pipeline.clone());

        info!("Starting gRPC server on {}", grpc_addr);

        let grpc_handle = tokio::spawn(async move {
            if let Err(e) = grpc_server.serve(&grpc_addr).await {
                warn!("gRPC server error (expected with skeleton impl): {}", e);
            }
        });

        // Wait for shutdown signal
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received, shutting down...");
            }
        }

        // Cleanup
        pipeline_handle.abort();
        http_handle.abort();
        grpc_handle.abort();

        info!("Collector stopped");
        Ok(())
    }

    /// Stop the collector
    pub async fn stop(&self) -> Result<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        Ok(())
    }

    /// Get a reference to the database
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get the pipeline for direct span submission
    pub fn pipeline(&self) -> Arc<Pipeline> {
        self.pipeline.clone()
    }
}

/// Span receiver trait for different ingestion protocols
#[async_trait::async_trait]
pub trait SpanReceiver: Send + Sync {
    /// Receive a single span
    async fn receive(&self, span: Span) -> Result<()>;

    /// Receive a batch of spans
    async fn receive_batch(&self, spans: Vec<Span>) -> Result<usize>;
}

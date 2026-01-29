//! Processing pipeline for spans
//!
//! The pipeline receives spans, enriches them with computed fields,
//! calculates costs, batches them for efficiency, and stores them.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::db::{Database, SpanRepository, RedisStreamer};
use crate::error::Result;
use crate::models::Span;

use super::cost::CostCalculator;

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Number of spans to batch before writing
    pub batch_size: usize,
    /// Maximum time to wait before flushing a partial batch (ms)
    pub batch_timeout_ms: u64,
    /// Whether to calculate costs for LLM spans
    pub enable_cost_calculation: bool,
    /// Whether to stream spans to Redis for real-time updates
    pub enable_redis_streaming: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_ms: 1000,
            enable_cost_calculation: true,
            enable_redis_streaming: true,
        }
    }
}

/// Processing pipeline for spans
pub struct Pipeline {
    config: PipelineConfig,
    span_tx: mpsc::Sender<Span>,
    span_rx: Arc<Mutex<Option<mpsc::Receiver<Span>>>>,
    cost_calculator: CostCalculator,
    span_repository: SpanRepository,
    redis_streamer: RedisStreamer,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(config: PipelineConfig, db: Database) -> Self {
        let (span_tx, span_rx) = mpsc::channel(config.batch_size * 10);

        Self {
            config,
            span_tx,
            span_rx: Arc::new(Mutex::new(Some(span_rx))),
            cost_calculator: CostCalculator::new(),
            span_repository: SpanRepository::new(&db.postgres),
            redis_streamer: RedisStreamer::new(&db.redis),
        }
    }

    /// Submit a span for processing
    pub async fn submit(&self, span: Span) -> Result<()> {
        self.span_tx
            .send(span)
            .await
            .map_err(|e| crate::error::Error::Channel(e.to_string()))?;
        Ok(())
    }

    /// Submit a batch of spans for processing
    pub async fn submit_batch(&self, spans: Vec<Span>) -> Result<usize> {
        let mut count = 0;
        for span in spans {
            if self.submit(span).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Start the pipeline processing loop
    pub async fn start(&self) {
        // Take ownership of the receiver
        let mut span_rx = {
            let mut guard = self.span_rx.lock();
            match guard.take() {
                Some(rx) => rx,
                None => {
                    error!("Pipeline already started");
                    return;
                }
            }
        };

        let batch_size = self.config.batch_size;
        let batch_timeout = Duration::from_millis(self.config.batch_timeout_ms);
        let enable_cost = self.config.enable_cost_calculation;
        let enable_redis = self.config.enable_redis_streaming;

        let cost_calculator = CostCalculator::new();
        let span_repository = self.span_repository.clone();
        let redis_streamer = self.redis_streamer.clone();

        info!(
            "Pipeline started (batch_size={}, timeout={}ms)",
            batch_size, self.config.batch_timeout_ms
        );

        let mut batch: Vec<Span> = Vec::with_capacity(batch_size);
        let mut flush_interval = interval(batch_timeout);

        loop {
            tokio::select! {
                // Receive a span
                Some(mut span) = span_rx.recv() => {
                    // Enrich the span
                    enrich_span(&mut span);

                    // Calculate cost if enabled
                    if enable_cost {
                        cost_calculator.calculate(&mut span);
                    }

                    // Stream to Redis if enabled
                    if enable_redis {
                        if let Err(e) = redis_streamer.publish_span(&span).await {
                            warn!("Failed to publish span to Redis: {}", e);
                        }
                    }

                    batch.push(span);

                    // Flush if batch is full
                    if batch.len() >= batch_size {
                        flush_batch(&span_repository, &mut batch).await;
                    }
                }

                // Periodic flush
                _ = flush_interval.tick() => {
                    if !batch.is_empty() {
                        flush_batch(&span_repository, &mut batch).await;
                    }
                }

                // Channel closed
                else => {
                    // Final flush
                    if !batch.is_empty() {
                        flush_batch(&span_repository, &mut batch).await;
                    }
                    info!("Pipeline stopped");
                    break;
                }
            }
        }
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            queue_capacity: self.span_tx.capacity(),
            queue_max_capacity: self.config.batch_size * 10,
        }
    }
}

/// Enrich a span with computed fields
fn enrich_span(span: &mut Span) {
    // Calculate duration if we have both timestamps
    span.calculate_duration();

    // Ensure service name is set
    if span.service_name.is_empty() {
        span.service_name = "unknown".to_string();
    }

    // Truncate previews if too long
    if let Some(ref mut preview) = span.prompt_preview {
        if preview.len() > 500 {
            preview.truncate(500);
            preview.push_str("...");
        }
    }

    if let Some(ref mut preview) = span.completion_preview {
        if preview.len() > 500 {
            preview.truncate(500);
            preview.push_str("...");
        }
    }
}

/// Flush a batch of spans to the database
async fn flush_batch(repo: &SpanRepository, batch: &mut Vec<Span>) {
    if batch.is_empty() {
        return;
    }

    let batch_size = batch.len();
    debug!("Flushing batch of {} spans", batch_size);

    match repo.insert_batch(batch).await {
        Ok(inserted) => {
            debug!("Inserted {} of {} spans", inserted, batch_size);
        }
        Err(e) => {
            error!("Failed to insert batch: {}", e);
            // TODO: implement retry logic or dead letter queue
        }
    }

    batch.clear();
}

/// Pipeline statistics
#[derive(Debug, Clone)]
pub struct PipelineStats {
    /// Current queue capacity (available slots)
    pub queue_capacity: usize,
    /// Maximum queue capacity
    pub queue_max_capacity: usize,
}

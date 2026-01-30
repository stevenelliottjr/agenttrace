//! Redis connection and pub/sub streaming

use deadpool_redis::{Config as RedisConfig, Pool, Runtime};
use futures_util::StreamExt;
use redis::aio::PubSub;
use redis::AsyncCommands;
use tokio::sync::mpsc;

use crate::config::RedisConfig as AppRedisConfig;
use crate::error::{Error, Result};
use crate::models::Span;

/// Redis connection pool
#[derive(Clone)]
pub struct RedisPool {
    pool: Pool,
    url: String,
}

impl RedisPool {
    /// Create a new Redis connection pool
    pub async fn new(config: &AppRedisConfig) -> Result<Self> {
        let cfg = RedisConfig::from_url(&config.url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| Error::Redis(e.to_string()))?;

        Ok(Self {
            pool,
            url: config.url.clone(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;
        Ok(())
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get the Redis URL for creating pub/sub connections
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Subscribe to a channel and return a receiver for messages
    pub async fn subscribe(&self, channel: &str) -> Result<mpsc::Receiver<String>> {
        let client = redis::Client::open(self.url.as_str())
            .map_err(|e| Error::Redis(e.to_string()))?;

        let (tx, rx) = mpsc::channel::<String>(100);
        let channel = channel.to_string();

        // Spawn a task that creates the pubsub connection and listens for messages
        tokio::spawn(async move {
            // Get a dedicated connection for pubsub
            let conn = match client.get_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to get Redis connection for pubsub: {}", e);
                    return;
                }
            };

            let mut pubsub: PubSub = conn.into_pubsub();

            if let Err(e) = pubsub.subscribe(&channel).await {
                tracing::error!("Failed to subscribe to channel {}: {}", channel, e);
                return;
            }

            tracing::info!("Subscribed to Redis channel: {}", channel);

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                match msg.get_payload::<String>() {
                    Ok(payload) => {
                        if tx.send(payload).await.is_err() {
                            // Receiver dropped, stop the loop
                            tracing::debug!("SSE client disconnected");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get message payload: {}", e);
                    }
                }
            }
        });

        Ok(rx)
    }
}

/// Redis streamer for real-time span updates
#[derive(Clone)]
pub struct RedisStreamer {
    pool: Pool,
}

impl RedisStreamer {
    /// Create a new Redis streamer
    pub fn new(pool: &RedisPool) -> Self {
        Self {
            pool: pool.pool.clone(),
        }
    }

    /// Publish a span to the real-time stream
    pub async fn publish_span(&self, span: &Span) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;

        let span_json = serde_json::to_string(span)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Publish to the spans channel
        let _: () = conn
            .publish("agenttrace:spans", &span_json)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;

        // Also publish to trace-specific channel for filtered subscriptions
        let trace_channel = format!("agenttrace:trace:{}", span.trace_id);
        let _: () = conn
            .publish(&trace_channel, &span_json)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;

        // If it's an LLM call, publish to the llm channel
        if span.is_llm_call() {
            let _: () = conn
                .publish("agenttrace:llm", &span_json)
                .await
                .map_err(|e| Error::Redis(e.to_string()))?;
        }

        Ok(())
    }

    /// Publish multiple spans
    pub async fn publish_batch(&self, spans: &[Span]) -> Result<usize> {
        let mut count = 0;
        for span in spans {
            if self.publish_span(span).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Store the latest metrics snapshot
    pub async fn set_metrics_snapshot(&self, key: &str, data: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;
        let _: () = conn
            .set_ex(key, data, ttl_seconds)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;
        Ok(())
    }

    /// Get the latest metrics snapshot
    pub async fn get_metrics_snapshot(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;
        Ok(value)
    }

    /// Increment a counter (for rate limiting, stats, etc.)
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;
        let value: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;
        Ok(value)
    }

    /// Set a key with expiration
    pub async fn set_with_expiry(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| Error::Redis(e.to_string()))?;
        let _: () = conn
            .set_ex(key, value, ttl_seconds)
            .await
            .map_err(|e| Error::Redis(e.to_string()))?;
        Ok(())
    }
}

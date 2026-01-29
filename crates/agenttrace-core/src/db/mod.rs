//! Database layer for AgentTrace
//!
//! Provides connections to TimescaleDB and Redis.

mod postgres;
mod redis;

pub use postgres::{PostgresPool, SpanRepository};
pub use redis::{RedisPool, RedisStreamer};

use crate::config::Config;
use crate::error::Result;

/// Database connections bundle
#[derive(Clone)]
pub struct Database {
    /// PostgreSQL/TimescaleDB connection pool
    pub postgres: PostgresPool,
    /// Redis connection pool
    pub redis: RedisPool,
}

impl Database {
    /// Create a new database connection bundle
    pub async fn new(config: &Config) -> Result<Self> {
        let postgres = PostgresPool::new(&config.database).await?;
        let redis = RedisPool::new(&config.redis).await?;

        Ok(Self { postgres, redis })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        self.postgres.migrate().await
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        self.postgres.health_check().await?;
        self.redis.health_check().await?;
        Ok(())
    }
}

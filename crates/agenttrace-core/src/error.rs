//! Error types for AgentTrace

use thiserror::Error;

/// Result type alias using AgentTrace's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for AgentTrace operations
#[derive(Error, Debug)]
pub enum Error {
    /// Database error from sqlx
    #[error("Database error: {0}")]
    DatabaseSqlx(#[from] sqlx::Error),

    /// Database error (string)
    #[error("Database error: {0}")]
    Database(String),

    /// Redis error from driver
    #[error("Redis error: {0}")]
    RedisDriver(#[from] redis::RedisError),

    /// Redis error (string)
    #[error("Redis error: {0}")]
    Redis(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error
    #[error("{entity} not found: {id}")]
    NotFound { entity: String, id: String },

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error from serde_json
    #[error("Serialization error: {0}")]
    SerializationJson(#[from] serde_json::Error),

    /// Serialization error (string)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// gRPC/Tonic error
    #[error("gRPC error: {0}")]
    Grpc(String),

    /// Channel send error
    #[error("Channel error: {0}")]
    Channel(String),
}

impl Error {
    /// Create a not found error
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

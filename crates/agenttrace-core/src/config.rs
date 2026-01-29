//! Configuration management for AgentTrace

use serde::{Deserialize, Serialize};

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Redis configuration
    pub redis: RedisConfig,

    /// Collector configuration
    pub collector: CollectorConfig,

    /// TUI configuration
    pub tui: TuiConfig,

    /// Alerting configuration
    pub alerting: AlertingConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            collector: CollectorConfig::default(),
            tui: TuiConfig::default(),
            alerting: AlertingConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// HTTP API port
    pub http_port: u16,
    /// gRPC port
    pub grpc_port: u16,
    /// UDP port
    pub udp_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            http_port: 8080,
            grpc_port: 4317,
            udp_port: 4318,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum connections
    pub max_connections: u32,
    /// Minimum connections
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://agenttrace:agenttrace_dev@localhost:5432/agenttrace".to_string(),
            max_connections: 20,
            min_connections: 5,
        }
    }
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Maximum connections
    pub max_connections: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
        }
    }
}

/// Collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Batch size for writing
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Buffer size for incoming spans
    pub buffer_size: usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_ms: 1000,
            buffer_size: 10000,
        }
    }
}

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Refresh rate in milliseconds
    pub refresh_rate_ms: u64,
    /// Default time range
    pub default_time_range: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 1000,
            default_time_range: "1h".to_string(),
        }
    }
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Check interval in seconds
    pub check_interval_seconds: u64,
    /// Notification cooldown in minutes
    pub notification_cooldown_minutes: u64,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            notification_cooldown_minutes: 5,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json or pretty)
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
        }
    }
}

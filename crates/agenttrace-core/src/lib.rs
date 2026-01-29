//! # AgentTrace
//!
//! Observability platform for AI agents.
//!
//! AgentTrace provides distributed tracing, cost tracking, and performance monitoring
//! specifically designed for AI agent workloads.
//!
//! ## Architecture
//!
//! - **Collector**: High-performance telemetry ingestion via UDP/gRPC
//! - **Storage**: TimescaleDB for time-series data, Redis for real-time
//! - **API**: REST API for queries and management
//! - **TUI**: Terminal-based dashboard
//!
//! ## Quick Start
//!
//! ```bash
//! # Start the collector
//! agenttrace serve
//!
//! # View the TUI dashboard
//! agenttrace dashboard
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod api;
pub mod collector;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod tui;

pub use config::Config;
pub use error::{Error, Result};

/// Re-exports for convenience
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::error::{Error, Result};
    pub use crate::models::*;
}

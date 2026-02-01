//! Terminal User Interface for AgentTrace
//!
//! Provides a real-time terminal dashboard for monitoring agent traces.

mod app;
mod components;
mod event;
mod ui;

pub use app::App;
pub use event::{Event, EventHandler};

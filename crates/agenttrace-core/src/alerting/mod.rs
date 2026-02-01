//! Alerting system for AgentTrace
//!
//! Provides cost threshold alerts, error rate monitoring, and notification delivery.

mod evaluator;
mod notifier;
mod repository;

pub use evaluator::AlertEvaluator;
pub use notifier::{NotificationSender, NotificationResult};
pub use repository::AlertRepository;

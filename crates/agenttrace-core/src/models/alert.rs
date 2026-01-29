//! Alert data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of alert condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// Simple threshold comparison
    Threshold,
    /// Statistical anomaly detection
    Anomaly,
    /// Rate of change detection
    RateChange,
    /// Absence of data
    Absence,
}

/// Comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Equal to
    Eq,
    /// Greater than or equal to
    Gte,
    /// Less than or equal to
    Lte,
    /// Not equal to
    Ne,
}

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational
    Info,
    /// Warning
    #[default]
    Warning,
    /// Critical
    Critical,
}

/// Status of an alert event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is currently active
    #[default]
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
}

/// An alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Description of what this alert monitors
    pub description: Option<String>,

    // Scope
    /// Service to monitor (None = all services)
    pub service_name: Option<String>,

    /// Environment to monitor
    pub environment: Option<String>,

    /// Model to monitor
    pub model_name: Option<String>,

    // Condition
    /// Type of condition
    pub condition_type: ConditionType,

    /// Metric to monitor (e.g., "error_rate", "latency_p99", "cost_sum")
    pub metric: String,

    /// Comparison operator
    pub operator: Operator,

    /// Threshold value
    pub threshold: Option<f64>,

    // Evaluation
    /// Time window in minutes
    pub window_minutes: i32,

    /// Evaluation interval in seconds
    pub evaluation_interval_seconds: i32,

    /// Number of consecutive failures before alerting
    pub consecutive_failures: i32,

    // Notification
    /// Alert severity
    pub severity: Severity,

    /// Notification channels
    pub notification_channels: Vec<NotificationChannel>,

    // State
    /// Whether the rule is enabled
    pub enabled: bool,

    /// Last evaluation time
    pub last_evaluated_at: Option<DateTime<Utc>>,

    /// Last time this rule triggered
    pub last_triggered_at: Option<DateTime<Utc>>,

    // Metadata
    /// When the rule was created
    pub created_at: DateTime<Utc>,

    /// When the rule was last updated
    pub updated_at: DateTime<Utc>,

    /// Who created the rule
    pub created_by: Option<String>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Slack webhook
    Slack { webhook_url: String, channel: Option<String> },
    /// Email notification
    Email { to: Vec<String> },
    /// Generic webhook
    Webhook { url: String, headers: Option<serde_json::Value> },
    /// PagerDuty
    PagerDuty { routing_key: String },
}

/// An alert event (triggered alert)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// Unique identifier
    pub id: Uuid,

    /// The rule that triggered this alert
    pub rule_id: Uuid,

    /// When the alert was triggered
    pub triggered_at: DateTime<Utc>,

    /// When the alert was resolved (if resolved)
    pub resolved_at: Option<DateTime<Utc>>,

    /// Current status
    pub status: AlertStatus,

    /// Severity level
    pub severity: Severity,

    /// Human-readable message
    pub message: String,

    /// The metric value that triggered the alert
    pub metric_value: f64,

    /// The threshold that was exceeded
    pub threshold_value: f64,

    /// Service that triggered the alert
    pub service_name: Option<String>,

    /// Sample trace IDs related to this alert
    pub trace_ids: Vec<String>,

    /// Notifications that were sent
    pub notifications_sent: Vec<NotificationRecord>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Record of a sent notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Channel type
    pub channel_type: String,

    /// When it was sent
    pub sent_at: DateTime<Utc>,

    /// Whether it succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

/// Input for creating a new alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleInput {
    pub name: String,
    pub description: Option<String>,
    pub service_name: Option<String>,
    pub environment: Option<String>,
    pub model_name: Option<String>,
    pub condition_type: ConditionType,
    pub metric: String,
    pub operator: Operator,
    pub threshold: Option<f64>,
    pub window_minutes: Option<i32>,
    pub evaluation_interval_seconds: Option<i32>,
    pub consecutive_failures: Option<i32>,
    pub severity: Option<Severity>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub enabled: Option<bool>,
}

impl AlertRule {
    /// Check if a value triggers this alert
    pub fn check(&self, value: f64) -> bool {
        let threshold = match self.threshold {
            Some(t) => t,
            None => return false,
        };

        match self.operator {
            Operator::Gt => value > threshold,
            Operator::Lt => value < threshold,
            Operator::Eq => (value - threshold).abs() < f64::EPSILON,
            Operator::Gte => value >= threshold,
            Operator::Lte => value <= threshold,
            Operator::Ne => (value - threshold).abs() >= f64::EPSILON,
        }
    }
}

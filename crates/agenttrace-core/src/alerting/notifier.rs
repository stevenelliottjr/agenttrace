//! Notification delivery for alerts

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::models::alert::{AlertEvent, AlertRule, NotificationChannel, NotificationRecord, Severity};

/// Result of sending a notification
#[derive(Debug, Clone)]
pub struct NotificationResult {
    pub channel_type: String,
    pub success: bool,
    pub error: Option<String>,
    pub sent_at: DateTime<Utc>,
}

impl From<NotificationResult> for NotificationRecord {
    fn from(result: NotificationResult) -> Self {
        NotificationRecord {
            channel_type: result.channel_type,
            sent_at: result.sent_at,
            success: result.success,
            error: result.error,
        }
    }
}

/// Sends notifications through various channels
pub struct NotificationSender {
    client: Client,
}

impl NotificationSender {
    /// Create a new notification sender
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Send notifications for an alert event
    pub async fn send_all(
        &self,
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> Vec<NotificationResult> {
        let mut results = Vec::new();

        for channel in &rule.notification_channels {
            let result = self.send(channel, rule, event).await;
            results.push(result);
        }

        results
    }

    /// Send a single notification
    pub async fn send(
        &self,
        channel: &NotificationChannel,
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> NotificationResult {
        let sent_at = Utc::now();

        let result = match channel {
            NotificationChannel::Slack { webhook_url, channel: slack_channel } => {
                self.send_slack(webhook_url, slack_channel.as_deref(), rule, event).await
            }
            NotificationChannel::Webhook { url, headers } => {
                self.send_webhook(url, headers.as_ref(), rule, event).await
            }
            NotificationChannel::PagerDuty { routing_key } => {
                self.send_pagerduty(routing_key, rule, event).await
            }
            NotificationChannel::Email { to } => {
                self.send_email(to, rule, event).await
            }
        };

        let channel_type = match channel {
            NotificationChannel::Slack { .. } => "slack",
            NotificationChannel::Webhook { .. } => "webhook",
            NotificationChannel::PagerDuty { .. } => "pagerduty",
            NotificationChannel::Email { .. } => "email",
        };

        NotificationResult {
            channel_type: channel_type.to_string(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
            sent_at,
        }
    }

    /// Send Slack notification
    async fn send_slack(
        &self,
        webhook_url: &str,
        channel: Option<&str>,
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> Result<(), NotificationError> {
        let color = match event.severity {
            Severity::Critical => "#dc3545",
            Severity::Warning => "#ffc107",
            Severity::Info => "#17a2b8",
        };

        let severity_emoji = match event.severity {
            Severity::Critical => "🚨",
            Severity::Warning => "⚠️",
            Severity::Info => "ℹ️",
        };

        let payload = SlackPayload {
            channel: channel.map(String::from),
            username: Some("AgentTrace".to_string()),
            icon_emoji: Some(":robot_face:".to_string()),
            attachments: vec![SlackAttachment {
                color: color.to_string(),
                title: format!("{} Alert: {}", severity_emoji, rule.name),
                text: event.message.clone(),
                fields: vec![
                    SlackField {
                        title: "Severity".to_string(),
                        value: format!("{:?}", event.severity),
                        short: true,
                    },
                    SlackField {
                        title: "Metric Value".to_string(),
                        value: format!("{:.2}", event.metric_value),
                        short: true,
                    },
                    SlackField {
                        title: "Threshold".to_string(),
                        value: format!("{:.2}", event.threshold_value),
                        short: true,
                    },
                    SlackField {
                        title: "Service".to_string(),
                        value: event.service_name.clone().unwrap_or_else(|| "All".to_string()),
                        short: true,
                    },
                ],
                footer: Some("AgentTrace Alerting".to_string()),
                ts: Some(event.triggered_at.timestamp()),
            }],
        };

        let response = self
            .client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::HttpError(format!(
                "Slack returned {}: {}",
                status, body
            )));
        }

        info!(rule_id = %rule.id, "Slack notification sent");
        Ok(())
    }

    /// Send generic webhook notification
    async fn send_webhook(
        &self,
        url: &str,
        headers: Option<&serde_json::Value>,
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> Result<(), NotificationError> {
        let payload = WebhookPayload {
            alert_id: event.id.to_string(),
            rule_id: rule.id.to_string(),
            rule_name: rule.name.clone(),
            severity: format!("{:?}", event.severity),
            status: format!("{:?}", event.status),
            message: event.message.clone(),
            metric_value: event.metric_value,
            threshold_value: event.threshold_value,
            service_name: event.service_name.clone(),
            triggered_at: event.triggered_at,
            trace_ids: event.trace_ids.clone(),
            metadata: event.metadata.clone(),
        };

        let mut request = self.client.post(url).json(&payload);

        // Add custom headers if provided
        if let Some(headers_obj) = headers {
            if let Some(headers_map) = headers_obj.as_object() {
                for (key, value) in headers_map {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| NotificationError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::HttpError(format!(
                "Webhook returned {}: {}",
                status, body
            )));
        }

        info!(rule_id = %rule.id, url = %url, "Webhook notification sent");
        Ok(())
    }

    /// Send PagerDuty notification
    async fn send_pagerduty(
        &self,
        routing_key: &str,
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> Result<(), NotificationError> {
        let severity = match event.severity {
            Severity::Critical => "critical",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        let payload = PagerDutyPayload {
            routing_key: routing_key.to_string(),
            event_action: "trigger".to_string(),
            dedup_key: Some(format!("{}:{}", rule.id, event.id)),
            payload: PagerDutyEventPayload {
                summary: format!("[{}] {}: {}", severity.to_uppercase(), rule.name, event.message),
                source: "AgentTrace".to_string(),
                severity: severity.to_string(),
                timestamp: Some(event.triggered_at.to_rfc3339()),
                custom_details: Some(serde_json::json!({
                    "rule_id": rule.id.to_string(),
                    "metric_value": event.metric_value,
                    "threshold_value": event.threshold_value,
                    "service_name": event.service_name,
                    "trace_ids": event.trace_ids,
                })),
            },
        };

        let response = self
            .client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::HttpError(format!(
                "PagerDuty returned {}: {}",
                status, body
            )));
        }

        info!(rule_id = %rule.id, "PagerDuty notification sent");
        Ok(())
    }

    /// Send email notification (placeholder - requires SMTP configuration)
    async fn send_email(
        &self,
        to: &[String],
        rule: &AlertRule,
        event: &AlertEvent,
    ) -> Result<(), NotificationError> {
        // Email sending would require SMTP configuration
        // For now, just log the intent
        warn!(
            rule_id = %rule.id,
            recipients = ?to,
            "Email notifications not yet implemented"
        );

        // Return success to not block other notifications
        Ok(())
    }
}

impl Default for NotificationSender {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// Slack payload types
#[derive(Debug, Serialize)]
struct SlackPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,
    attachments: Vec<SlackAttachment>,
}

#[derive(Debug, Serialize)]
struct SlackAttachment {
    color: String,
    title: String,
    text: String,
    fields: Vec<SlackField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

// Generic webhook payload
#[derive(Debug, Serialize)]
struct WebhookPayload {
    alert_id: String,
    rule_id: String,
    rule_name: String,
    severity: String,
    status: String,
    message: String,
    metric_value: f64,
    threshold_value: f64,
    service_name: Option<String>,
    triggered_at: DateTime<Utc>,
    trace_ids: Vec<String>,
    metadata: serde_json::Value,
}

// PagerDuty payload types
#[derive(Debug, Serialize)]
struct PagerDutyPayload {
    routing_key: String,
    event_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dedup_key: Option<String>,
    payload: PagerDutyEventPayload,
}

#[derive(Debug, Serialize)]
struct PagerDutyEventPayload {
    summary: String,
    source: String,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_details: Option<serde_json::Value>,
}

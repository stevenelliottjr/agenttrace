//! Alert rule evaluation engine

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Interval};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::db::SpanRepository;
use crate::models::alert::{
    AlertEvent, AlertRule, AlertRuleInput, AlertStatus, ConditionType, NotificationRecord,
    Operator, Severity,
};

use super::notifier::NotificationSender;
use super::repository::AlertRepository;

/// Metric value with metadata
#[derive(Debug, Clone)]
pub struct MetricValue {
    pub value: f64,
    pub sample_trace_ids: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Alert evaluator that periodically checks rules against metrics
pub struct AlertEvaluator {
    /// Alert rule repository
    alert_repo: AlertRepository,
    /// Span repository for querying metrics
    span_repo: SpanRepository,
    /// Notification sender
    notifier: NotificationSender,
    /// State tracking for consecutive failures
    failure_counts: Arc<RwLock<HashMap<Uuid, i32>>>,
    /// Currently active alerts (rule_id -> event)
    active_alerts: Arc<RwLock<HashMap<Uuid, AlertEvent>>>,
    /// Default evaluation interval
    default_interval_secs: u64,
}

impl AlertEvaluator {
    /// Create a new alert evaluator
    pub fn new(alert_repo: AlertRepository, span_repo: SpanRepository) -> Self {
        Self {
            alert_repo,
            span_repo,
            notifier: NotificationSender::new(),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            default_interval_secs: 60,
        }
    }

    /// Start the evaluation loop
    pub async fn start(&self) {
        info!("Starting alert evaluator");

        let mut ticker = interval(std::time::Duration::from_secs(self.default_interval_secs));

        loop {
            ticker.tick().await;

            if let Err(e) = self.evaluate_all().await {
                error!(error = %e, "Error evaluating alerts");
            }
        }
    }

    /// Evaluate all enabled rules
    pub async fn evaluate_all(&self) -> crate::error::Result<()> {
        let rules = self.alert_repo.list_enabled().await?;

        debug!(count = rules.len(), "Evaluating alert rules");

        for rule in rules {
            if let Err(e) = self.evaluate_rule(&rule).await {
                error!(rule_id = %rule.id, error = %e, "Error evaluating rule");
            }
        }

        Ok(())
    }

    /// Evaluate a single rule
    pub async fn evaluate_rule(&self, rule: &AlertRule) -> crate::error::Result<()> {
        // Calculate time window
        let window_end = Utc::now();
        let window_start = window_end - Duration::minutes(rule.window_minutes as i64);

        // Get metric value based on rule configuration
        let metric_value = self
            .get_metric_value(rule, window_start, window_end)
            .await?;

        let Some(metric) = metric_value else {
            debug!(rule_id = %rule.id, "No data for metric");
            return Ok(());
        };

        // Check if threshold is breached
        let is_breached = rule.check(metric.value);

        debug!(
            rule_id = %rule.id,
            metric = rule.metric,
            value = metric.value,
            threshold = ?rule.threshold,
            breached = is_breached,
            "Evaluated rule"
        );

        if is_breached {
            self.handle_breach(rule, metric).await?;
        } else {
            self.handle_recovery(rule).await?;
        }

        // Update last evaluated time
        self.alert_repo.update_last_evaluated(rule.id).await?;

        Ok(())
    }

    /// Get metric value for a rule
    async fn get_metric_value(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let value = match rule.metric.as_str() {
            "error_rate" => self.get_error_rate(rule, start, end).await?,
            "latency_p50" => self.get_latency_percentile(rule, start, end, 0.5).await?,
            "latency_p95" => self.get_latency_percentile(rule, start, end, 0.95).await?,
            "latency_p99" => self.get_latency_percentile(rule, start, end, 0.99).await?,
            "latency_avg" => self.get_latency_avg(rule, start, end).await?,
            "cost_sum" => self.get_cost_sum(rule, start, end).await?,
            "cost_rate" => self.get_cost_rate(rule, start, end).await?,
            "token_sum" => self.get_token_sum(rule, start, end).await?,
            "span_count" => self.get_span_count(rule, start, end).await?,
            "throughput" => self.get_throughput(rule, start, end).await?,
            _ => {
                warn!(metric = rule.metric, "Unknown metric type");
                None
            }
        };

        Ok(value)
    }

    /// Get error rate metric
    async fn get_error_rate(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let stats = self
            .span_repo
            .get_error_stats(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        if stats.total == 0 {
            return Ok(None);
        }

        let rate = stats.error_count as f64 / stats.total as f64 * 100.0;

        Ok(Some(MetricValue {
            value: rate,
            sample_trace_ids: stats.sample_trace_ids,
            timestamp: Utc::now(),
        }))
    }

    /// Get latency percentile metric
    async fn get_latency_percentile(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        percentile: f64,
    ) -> crate::error::Result<Option<MetricValue>> {
        let latency = self
            .span_repo
            .get_latency_percentile(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
                percentile,
            )
            .await?;

        Ok(latency.map(|l| MetricValue {
            value: l,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get average latency metric
    async fn get_latency_avg(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let latency = self
            .span_repo
            .get_latency_avg(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        Ok(latency.map(|l| MetricValue {
            value: l,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get total cost metric
    async fn get_cost_sum(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let cost = self
            .span_repo
            .get_cost_sum(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        Ok(cost.map(|c| MetricValue {
            value: c,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get cost rate (per hour) metric
    async fn get_cost_rate(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let cost = self
            .span_repo
            .get_cost_sum(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        let duration_hours = (end - start).num_minutes() as f64 / 60.0;
        if duration_hours == 0.0 {
            return Ok(None);
        }

        Ok(cost.map(|c| MetricValue {
            value: c / duration_hours,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get total token usage metric
    async fn get_token_sum(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let tokens = self
            .span_repo
            .get_token_sum(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        Ok(tokens.map(|t| MetricValue {
            value: t as f64,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get span count metric
    async fn get_span_count(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let count = self
            .span_repo
            .get_span_count(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        Ok(Some(MetricValue {
            value: count as f64,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Get throughput (spans per minute) metric
    async fn get_throughput(
        &self,
        rule: &AlertRule,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::error::Result<Option<MetricValue>> {
        let count = self
            .span_repo
            .get_span_count(
                rule.service_name.as_deref(),
                rule.model_name.as_deref(),
                start,
                end,
            )
            .await?;

        let duration_minutes = (end - start).num_minutes() as f64;
        if duration_minutes == 0.0 {
            return Ok(None);
        }

        Ok(Some(MetricValue {
            value: count as f64 / duration_minutes,
            sample_trace_ids: vec![],
            timestamp: Utc::now(),
        }))
    }

    /// Handle a threshold breach
    async fn handle_breach(&self, rule: &AlertRule, metric: MetricValue) -> crate::error::Result<()> {
        // Increment failure count
        let mut counts = self.failure_counts.write().await;
        let count = counts.entry(rule.id).or_insert(0);
        *count += 1;

        debug!(
            rule_id = %rule.id,
            consecutive_failures = *count,
            required = rule.consecutive_failures,
            "Breach detected"
        );

        // Check if we've hit the consecutive failure threshold
        if *count < rule.consecutive_failures {
            return Ok(());
        }

        // Check if alert is already active
        let active = self.active_alerts.read().await;
        if active.contains_key(&rule.id) {
            return Ok(());
        }
        drop(active);

        // Create alert event
        let event = AlertEvent {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            triggered_at: Utc::now(),
            resolved_at: None,
            status: AlertStatus::Active,
            severity: rule.severity,
            message: self.format_alert_message(rule, &metric),
            metric_value: metric.value,
            threshold_value: rule.threshold.unwrap_or(0.0),
            service_name: rule.service_name.clone(),
            trace_ids: metric.sample_trace_ids,
            notifications_sent: vec![],
            metadata: serde_json::json!({}),
        };

        info!(
            rule_id = %rule.id,
            event_id = %event.id,
            severity = ?event.severity,
            "Alert triggered"
        );

        // Store alert event
        self.alert_repo.create_event(&event).await?;

        // Update last triggered time
        self.alert_repo.update_last_triggered(rule.id).await?;

        // Send notifications
        let results = self.notifier.send_all(rule, &event).await;

        // Update event with notification records
        let records: Vec<NotificationRecord> = results.into_iter().map(|r| r.into()).collect();
        self.alert_repo.update_event_notifications(event.id, &records).await?;

        // Mark as active
        let mut active = self.active_alerts.write().await;
        active.insert(rule.id, event);

        Ok(())
    }

    /// Handle recovery (no longer breaching)
    async fn handle_recovery(&self, rule: &AlertRule) -> crate::error::Result<()> {
        // Reset failure count
        let mut counts = self.failure_counts.write().await;
        counts.remove(&rule.id);

        // Check if there's an active alert to resolve
        let mut active = self.active_alerts.write().await;
        if let Some(mut event) = active.remove(&rule.id) {
            info!(
                rule_id = %rule.id,
                event_id = %event.id,
                "Alert resolved"
            );

            event.status = AlertStatus::Resolved;
            event.resolved_at = Some(Utc::now());

            self.alert_repo.resolve_event(event.id).await?;
        }

        Ok(())
    }

    /// Format alert message
    fn format_alert_message(&self, rule: &AlertRule, metric: &MetricValue) -> String {
        let operator_str = match rule.operator {
            Operator::Gt => "exceeded",
            Operator::Lt => "fell below",
            Operator::Eq => "equals",
            Operator::Gte => "reached or exceeded",
            Operator::Lte => "fell to or below",
            Operator::Ne => "differs from",
        };

        let scope = match (&rule.service_name, &rule.model_name) {
            (Some(s), Some(m)) => format!(" for service '{}' with model '{}'", s, m),
            (Some(s), None) => format!(" for service '{}'", s),
            (None, Some(m)) => format!(" for model '{}'", m),
            (None, None) => String::new(),
        };

        format!(
            "{} {} threshold of {:.2}{} (current value: {:.2})",
            rule.metric,
            operator_str,
            rule.threshold.unwrap_or(0.0),
            scope,
            metric.value
        )
    }

    /// Manually test a rule (returns the event without persisting)
    pub async fn test_rule(&self, rule: &AlertRule) -> crate::error::Result<Option<AlertEvent>> {
        let window_end = Utc::now();
        let window_start = window_end - Duration::minutes(rule.window_minutes as i64);

        let metric_value = self.get_metric_value(rule, window_start, window_end).await?;

        let Some(metric) = metric_value else {
            return Ok(None);
        };

        let is_breached = rule.check(metric.value);

        if !is_breached {
            return Ok(None);
        }

        let event = AlertEvent {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            triggered_at: Utc::now(),
            resolved_at: None,
            status: AlertStatus::Active,
            severity: rule.severity,
            message: self.format_alert_message(rule, &metric),
            metric_value: metric.value,
            threshold_value: rule.threshold.unwrap_or(0.0),
            service_name: rule.service_name.clone(),
            trace_ids: metric.sample_trace_ids,
            notifications_sent: vec![],
            metadata: serde_json::json!({"test": true}),
        };

        Ok(Some(event))
    }
}

//! Alert repository for storing and querying alert rules and events

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::alert::{
    AlertEvent, AlertRule, AlertRuleInput, AlertStatus, ConditionType, NotificationChannel,
    NotificationRecord, Operator, Severity,
};

/// Repository for alert rules and events
#[derive(Clone)]
pub struct AlertRepository {
    pool: PgPool,
}

impl AlertRepository {
    /// Create a new alert repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Alert Rules ---

    /// Create a new alert rule
    pub async fn create_rule(&self, input: AlertRuleInput) -> Result<AlertRule> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let rule = AlertRule {
            id,
            name: input.name,
            description: input.description,
            service_name: input.service_name,
            environment: input.environment,
            model_name: input.model_name,
            condition_type: input.condition_type,
            metric: input.metric,
            operator: input.operator,
            threshold: input.threshold,
            window_minutes: input.window_minutes.unwrap_or(5),
            evaluation_interval_seconds: input.evaluation_interval_seconds.unwrap_or(60),
            consecutive_failures: input.consecutive_failures.unwrap_or(1),
            severity: input.severity.unwrap_or_default(),
            notification_channels: input.notification_channels.unwrap_or_default(),
            enabled: input.enabled.unwrap_or(true),
            last_evaluated_at: None,
            last_triggered_at: None,
            created_at: now,
            updated_at: now,
            created_by: None,
        };

        let channels_json = serde_json::to_value(&rule.notification_channels)?;

        sqlx::query(
            r#"
            INSERT INTO alert_rules (
                id, name, description, service_name, environment, model_name,
                condition_type, metric, operator, threshold,
                window_minutes, evaluation_interval_seconds, consecutive_failures,
                severity, notification_channels, enabled,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.service_name)
        .bind(&rule.environment)
        .bind(&rule.model_name)
        .bind(format!("{:?}", rule.condition_type).to_lowercase())
        .bind(&rule.metric)
        .bind(format!("{:?}", rule.operator).to_lowercase())
        .bind(rule.threshold)
        .bind(rule.window_minutes)
        .bind(rule.evaluation_interval_seconds)
        .bind(rule.consecutive_failures)
        .bind(format!("{:?}", rule.severity).to_lowercase())
        .bind(&channels_json)
        .bind(rule.enabled)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, id: Uuid) -> Result<Option<AlertRule>> {
        let row = sqlx::query_as::<_, AlertRuleRow>(
            r#"
            SELECT * FROM alert_rules WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// List all rules
    pub async fn list_rules(&self) -> Result<Vec<AlertRule>> {
        let rows = sqlx::query_as::<_, AlertRuleRow>(
            r#"
            SELECT * FROM alert_rules ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// List enabled rules
    pub async fn list_enabled(&self) -> Result<Vec<AlertRule>> {
        let rows = sqlx::query_as::<_, AlertRuleRow>(
            r#"
            SELECT * FROM alert_rules WHERE enabled = true ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Update a rule
    pub async fn update_rule(&self, id: Uuid, input: AlertRuleInput) -> Result<Option<AlertRule>> {
        let channels_json = input
            .notification_channels
            .as_ref()
            .map(|c| serde_json::to_value(c).ok())
            .flatten();

        let result = sqlx::query(
            r#"
            UPDATE alert_rules SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                service_name = COALESCE($4, service_name),
                environment = COALESCE($5, environment),
                model_name = COALESCE($6, model_name),
                threshold = COALESCE($7, threshold),
                window_minutes = COALESCE($8, window_minutes),
                evaluation_interval_seconds = COALESCE($9, evaluation_interval_seconds),
                consecutive_failures = COALESCE($10, consecutive_failures),
                notification_channels = COALESCE($11, notification_channels),
                enabled = COALESCE($12, enabled),
                updated_at = $13
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.service_name)
        .bind(&input.environment)
        .bind(&input.model_name)
        .bind(input.threshold)
        .bind(input.window_minutes)
        .bind(input.evaluation_interval_seconds)
        .bind(input.consecutive_failures)
        .bind(&channels_json)
        .bind(input.enabled)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_rule(id).await
    }

    /// Delete a rule
    pub async fn delete_rule(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM alert_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update last evaluated time
    pub async fn update_last_evaluated(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE alert_rules SET last_evaluated_at = $2 WHERE id = $1")
            .bind(id)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update last triggered time
    pub async fn update_last_triggered(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE alert_rules SET last_triggered_at = $2 WHERE id = $1")
            .bind(id)
            .bind(Utc::now())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Alert Events ---

    /// Create an alert event
    pub async fn create_event(&self, event: &AlertEvent) -> Result<()> {
        let trace_ids_json = serde_json::to_value(&event.trace_ids)?;
        let notifications_json = serde_json::to_value(&event.notifications_sent)?;

        sqlx::query(
            r#"
            INSERT INTO alert_events (
                id, rule_id, triggered_at, status, severity, message,
                metric_value, threshold_value, service_name, trace_ids,
                notifications_sent, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(event.id)
        .bind(event.rule_id)
        .bind(event.triggered_at)
        .bind(format!("{:?}", event.status).to_lowercase())
        .bind(format!("{:?}", event.severity).to_lowercase())
        .bind(&event.message)
        .bind(event.metric_value)
        .bind(event.threshold_value)
        .bind(&event.service_name)
        .bind(&trace_ids_json)
        .bind(&notifications_json)
        .bind(&event.metadata)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get an event by ID
    pub async fn get_event(&self, id: Uuid) -> Result<Option<AlertEvent>> {
        let row = sqlx::query_as::<_, AlertEventRow>(
            "SELECT * FROM alert_events WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// List events for a rule
    pub async fn list_events_for_rule(
        &self,
        rule_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AlertEvent>> {
        let rows = sqlx::query_as::<_, AlertEventRow>(
            r#"
            SELECT * FROM alert_events
            WHERE rule_id = $1
            ORDER BY triggered_at DESC
            LIMIT $2
            "#,
        )
        .bind(rule_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// List active events
    pub async fn list_active_events(&self) -> Result<Vec<AlertEvent>> {
        let rows = sqlx::query_as::<_, AlertEventRow>(
            r#"
            SELECT * FROM alert_events
            WHERE status = 'active'
            ORDER BY triggered_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// List recent events
    pub async fn list_recent_events(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AlertEvent>> {
        let rows = sqlx::query_as::<_, AlertEventRow>(
            r#"
            SELECT * FROM alert_events
            WHERE triggered_at >= $1
            ORDER BY triggered_at DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Resolve an event
    pub async fn resolve_event(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE alert_events
            SET status = 'resolved', resolved_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Acknowledge an event
    pub async fn acknowledge_event(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE alert_events SET status = 'acknowledged' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update event notifications
    pub async fn update_event_notifications(
        &self,
        id: Uuid,
        notifications: &[NotificationRecord],
    ) -> Result<()> {
        let json = serde_json::to_value(notifications)?;

        sqlx::query("UPDATE alert_events SET notifications_sent = $2 WHERE id = $1")
            .bind(id)
            .bind(&json)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// Database row types for mapping

#[derive(sqlx::FromRow)]
struct AlertRuleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    service_name: Option<String>,
    environment: Option<String>,
    model_name: Option<String>,
    condition_type: String,
    metric: String,
    operator: String,
    threshold: Option<f64>,
    window_minutes: i32,
    evaluation_interval_seconds: i32,
    consecutive_failures: i32,
    severity: String,
    notification_channels: serde_json::Value,
    enabled: bool,
    last_evaluated_at: Option<DateTime<Utc>>,
    last_triggered_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<String>,
}

impl From<AlertRuleRow> for AlertRule {
    fn from(row: AlertRuleRow) -> Self {
        let condition_type = match row.condition_type.as_str() {
            "threshold" => ConditionType::Threshold,
            "anomaly" => ConditionType::Anomaly,
            "rate_change" => ConditionType::RateChange,
            "absence" => ConditionType::Absence,
            _ => ConditionType::Threshold,
        };

        let operator = match row.operator.as_str() {
            "gt" => Operator::Gt,
            "lt" => Operator::Lt,
            "eq" => Operator::Eq,
            "gte" => Operator::Gte,
            "lte" => Operator::Lte,
            "ne" => Operator::Ne,
            _ => Operator::Gt,
        };

        let severity = match row.severity.as_str() {
            "info" => Severity::Info,
            "warning" => Severity::Warning,
            "critical" => Severity::Critical,
            _ => Severity::Warning,
        };

        let notification_channels: Vec<NotificationChannel> =
            serde_json::from_value(row.notification_channels).unwrap_or_default();

        AlertRule {
            id: row.id,
            name: row.name,
            description: row.description,
            service_name: row.service_name,
            environment: row.environment,
            model_name: row.model_name,
            condition_type,
            metric: row.metric,
            operator,
            threshold: row.threshold,
            window_minutes: row.window_minutes,
            evaluation_interval_seconds: row.evaluation_interval_seconds,
            consecutive_failures: row.consecutive_failures,
            severity,
            notification_channels,
            enabled: row.enabled,
            last_evaluated_at: row.last_evaluated_at,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertEventRow {
    id: Uuid,
    rule_id: Uuid,
    triggered_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    status: String,
    severity: String,
    message: String,
    metric_value: f64,
    threshold_value: f64,
    service_name: Option<String>,
    trace_ids: serde_json::Value,
    notifications_sent: serde_json::Value,
    metadata: serde_json::Value,
}

impl From<AlertEventRow> for AlertEvent {
    fn from(row: AlertEventRow) -> Self {
        let status = match row.status.as_str() {
            "active" => AlertStatus::Active,
            "acknowledged" => AlertStatus::Acknowledged,
            "resolved" => AlertStatus::Resolved,
            _ => AlertStatus::Active,
        };

        let severity = match row.severity.as_str() {
            "info" => Severity::Info,
            "warning" => Severity::Warning,
            "critical" => Severity::Critical,
            _ => Severity::Warning,
        };

        let trace_ids: Vec<String> = serde_json::from_value(row.trace_ids).unwrap_or_default();
        let notifications_sent: Vec<NotificationRecord> =
            serde_json::from_value(row.notifications_sent).unwrap_or_default();

        AlertEvent {
            id: row.id,
            rule_id: row.rule_id,
            triggered_at: row.triggered_at,
            resolved_at: row.resolved_at,
            status,
            severity,
            message: row.message,
            metric_value: row.metric_value,
            threshold_value: row.threshold_value,
            service_name: row.service_name,
            trace_ids,
            notifications_sent,
            metadata: row.metadata,
        }
    }
}

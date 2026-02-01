//! PostgreSQL/TimescaleDB connection and queries

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use uuid::Uuid;

use crate::config::DatabaseConfig;
use crate::error::{Error, Result};
use crate::models::{
    Span, SpanStatus, SpanKind,
    CostMetric, ErrorMetric, ErrorStats, LatencyMetric, MetricsSummaryResponse,
    SearchFilter, SortConfig, TraceSummary,
};

/// PostgreSQL connection pool
#[derive(Clone)]
pub struct PostgresPool {
    pool: PgPool,
}

impl PostgresPool {
    /// Create a new PostgreSQL connection pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Run migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Repository for span operations
#[derive(Clone)]
pub struct SpanRepository {
    pool: PgPool,
}

impl SpanRepository {
    /// Create a new span repository
    pub fn new(pool: &PostgresPool) -> Self {
        Self {
            pool: pool.pool.clone(),
        }
    }

    /// Insert a single span
    pub async fn insert(&self, span: &Span) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO spans (
                id, span_id, trace_id, parent_span_id, operation_name, service_name,
                span_kind, started_at, ended_at, duration_ms, status, status_message,
                model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                cost_usd, tool_name, tool_input, tool_output, tool_duration_ms,
                prompt_preview, completion_preview, attributes, events
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26
            )
            ON CONFLICT (span_id, started_at) DO UPDATE SET
                ended_at = EXCLUDED.ended_at,
                duration_ms = EXCLUDED.duration_ms,
                status = EXCLUDED.status,
                status_message = EXCLUDED.status_message,
                tokens_in = EXCLUDED.tokens_in,
                tokens_out = EXCLUDED.tokens_out,
                cost_usd = EXCLUDED.cost_usd,
                tool_output = EXCLUDED.tool_output,
                completion_preview = EXCLUDED.completion_preview,
                events = EXCLUDED.events
            "#,
        )
        .bind(&span.id)
        .bind(&span.span_id)
        .bind(&span.trace_id)
        .bind(&span.parent_span_id)
        .bind(&span.operation_name)
        .bind(&span.service_name)
        .bind(span_kind_to_str(&span.span_kind))
        .bind(&span.started_at)
        .bind(&span.ended_at)
        .bind(&span.duration_ms)
        .bind(span_status_to_str(&span.status))
        .bind(&span.status_message)
        .bind(&span.model_name)
        .bind(&span.model_provider)
        .bind(&span.tokens_in)
        .bind(&span.tokens_out)
        .bind(&span.tokens_reasoning)
        .bind(&span.cost_usd)
        .bind(&span.tool_name)
        .bind(&span.tool_input)
        .bind(&span.tool_output)
        .bind(&span.tool_duration_ms)
        .bind(&span.prompt_preview)
        .bind(&span.completion_preview)
        .bind(&span.attributes)
        .bind(serde_json::to_value(&span.events).unwrap_or_default())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Insert multiple spans in a batch
    pub async fn insert_batch(&self, spans: &[Span]) -> Result<usize> {
        if spans.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await.map_err(|e| Error::Database(e.to_string()))?;
        let mut count = 0;

        for span in spans {
            let result = sqlx::query(
                r#"
                INSERT INTO spans (
                    id, span_id, trace_id, parent_span_id, operation_name, service_name,
                    span_kind, started_at, ended_at, duration_ms, status, status_message,
                    model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                    cost_usd, tool_name, tool_input, tool_output, tool_duration_ms,
                    prompt_preview, completion_preview, attributes, events
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                    $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26
                )
                ON CONFLICT (span_id, started_at) DO NOTHING
                "#,
            )
            .bind(&span.id)
            .bind(&span.span_id)
            .bind(&span.trace_id)
            .bind(&span.parent_span_id)
            .bind(&span.operation_name)
            .bind(&span.service_name)
            .bind(span_kind_to_str(&span.span_kind))
            .bind(&span.started_at)
            .bind(&span.ended_at)
            .bind(&span.duration_ms)
            .bind(span_status_to_str(&span.status))
            .bind(&span.status_message)
            .bind(&span.model_name)
            .bind(&span.model_provider)
            .bind(&span.tokens_in)
            .bind(&span.tokens_out)
            .bind(&span.tokens_reasoning)
            .bind(&span.cost_usd)
            .bind(&span.tool_name)
            .bind(&span.tool_input)
            .bind(&span.tool_output)
            .bind(&span.tool_duration_ms)
            .bind(&span.prompt_preview)
            .bind(&span.completion_preview)
            .bind(&span.attributes)
            .bind(serde_json::to_value(&span.events).unwrap_or_default())
            .execute(&mut *tx)
            .await;

            if result.is_ok() {
                count += 1;
            }
        }

        tx.commit().await.map_err(|e| Error::Database(e.to_string()))?;
        Ok(count)
    }

    /// Get a span by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<Span>> {
        let row = sqlx::query(
            r#"
            SELECT id, span_id, trace_id, parent_span_id, operation_name, service_name,
                   span_kind, started_at, ended_at, duration_ms, status, status_message,
                   model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                   CAST(cost_usd AS DOUBLE PRECISION) as cost_usd,
                   tool_name, tool_input, tool_output, tool_duration_ms,
                   prompt_preview, completion_preview, attributes, events
            FROM spans WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(row_to_span(&row)?)),
            None => Ok(None),
        }
    }

    /// Get spans by trace ID
    pub async fn get_by_trace_id(&self, trace_id: &str) -> Result<Vec<Span>> {
        let rows = sqlx::query(
            r#"
            SELECT id, span_id, trace_id, parent_span_id, operation_name, service_name,
                   span_kind, started_at, ended_at, duration_ms, status, status_message,
                   model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                   CAST(cost_usd AS DOUBLE PRECISION) as cost_usd,
                   tool_name, tool_input, tool_output, tool_duration_ms,
                   prompt_preview, completion_preview, attributes, events
            FROM spans WHERE trace_id = $1 ORDER BY started_at ASC
            "#,
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        rows.iter().map(row_to_span).collect()
    }

    /// Get recent spans
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<Span>> {
        let rows = sqlx::query(
            r#"
            SELECT id, span_id, trace_id, parent_span_id, operation_name, service_name,
                   span_kind, started_at, ended_at, duration_ms, status, status_message,
                   model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                   CAST(cost_usd AS DOUBLE PRECISION) as cost_usd,
                   tool_name, tool_input, tool_output, tool_duration_ms,
                   prompt_preview, completion_preview, attributes, events
            FROM spans ORDER BY started_at DESC LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        rows.iter().map(row_to_span).collect()
    }

    // =========================================================================
    // Search Methods
    // =========================================================================

    /// Search spans with filters
    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &self,
        query: Option<&str>,
        service: Option<&str>,
        model: Option<&str>,
        status: Option<&str>,
        min_duration: Option<f64>,
        max_duration: Option<f64>,
        min_cost: Option<f64>,
        max_cost: Option<f64>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        sort_by: &str,
        sort_desc: bool,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Span>, i64)> {
        let mut conditions = vec!["1=1".to_string()];

        if let Some(q) = query {
            conditions.push(format!(
                "(operation_name ILIKE '%{}%' OR prompt_preview ILIKE '%{}%' OR completion_preview ILIKE '%{}%')",
                q.replace('\'', "''"), q.replace('\'', "''"), q.replace('\'', "''")
            ));
        }

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        if let Some(s) = status {
            conditions.push(format!("status = '{}'", s.replace('\'', "''")));
        }

        if let Some(min) = min_duration {
            conditions.push(format!("duration_ms >= {}", min));
        }

        if let Some(max) = max_duration {
            conditions.push(format!("duration_ms <= {}", max));
        }

        if let Some(min) = min_cost {
            conditions.push(format!("cost_usd >= {}", min));
        }

        if let Some(max) = max_cost {
            conditions.push(format!("cost_usd <= {}", max));
        }

        if let Some(start) = since {
            conditions.push(format!("started_at >= '{}'", start.format("%Y-%m-%d %H:%M:%S")));
        }

        if let Some(end) = until {
            conditions.push(format!("started_at <= '{}'", end.format("%Y-%m-%d %H:%M:%S")));
        }

        let where_clause = conditions.join(" AND ");
        let order = if sort_desc { "DESC" } else { "ASC" };

        let count_sql = format!("SELECT COUNT(*) as cnt FROM spans WHERE {}", where_clause);
        let count_row = sqlx::query(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        let total: i64 = count_row.try_get("cnt").unwrap_or(0);

        let sql = format!(
            r#"
            SELECT id, span_id, trace_id, parent_span_id, operation_name, service_name,
                   span_kind, started_at, ended_at, duration_ms, status, status_message,
                   model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                   CAST(cost_usd AS DOUBLE PRECISION) as cost_usd,
                   tool_name, tool_input, tool_output, tool_duration_ms,
                   prompt_preview, completion_preview, attributes, events
            FROM spans WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}
            "#,
            where_clause, sort_by, order, limit, offset
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let spans: Vec<Span> = rows.iter().filter_map(|r| row_to_span(r).ok()).collect();

        Ok((spans, total))
    }

    /// Advanced search with complex filters
    pub async fn advanced_search(
        &self,
        filters: &[SearchFilter],
        sort: Option<&SortConfig>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Span>, i64)> {
        let mut conditions = vec!["1=1".to_string()];

        for filter in filters {
            let op = match filter.operator.as_str() {
                "eq" => "=",
                "ne" => "!=",
                "gt" => ">",
                "gte" => ">=",
                "lt" => "<",
                "lte" => "<=",
                "contains" => "ILIKE",
                _ => "=",
            };

            let value_str = match &filter.value {
                serde_json::Value::String(s) => {
                    if filter.operator == "contains" {
                        format!("'%{}%'", s.replace('\'', "''"))
                    } else {
                        format!("'{}'", s.replace('\'', "''"))
                    }
                }
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };

            conditions.push(format!("{} {} {}", filter.field, op, value_str));
        }

        let where_clause = conditions.join(" AND ");
        let (sort_field, sort_desc) = sort
            .map(|s| (s.field.as_str(), s.descending))
            .unwrap_or(("started_at", true));
        let order = if sort_desc { "DESC" } else { "ASC" };

        let count_sql = format!("SELECT COUNT(*) as cnt FROM spans WHERE {}", where_clause);
        let count_row = sqlx::query(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        let total: i64 = count_row.try_get("cnt").unwrap_or(0);

        let sql = format!(
            r#"
            SELECT id, span_id, trace_id, parent_span_id, operation_name, service_name,
                   span_kind, started_at, ended_at, duration_ms, status, status_message,
                   model_name, model_provider, tokens_in, tokens_out, tokens_reasoning,
                   CAST(cost_usd AS DOUBLE PRECISION) as cost_usd,
                   tool_name, tool_input, tool_output, tool_duration_ms,
                   prompt_preview, completion_preview, attributes, events
            FROM spans WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}
            "#,
            where_clause, sort_field, order, limit, offset
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let spans: Vec<Span> = rows.iter().filter_map(|r| row_to_span(r).ok()).collect();

        Ok((spans, total))
    }

    /// List traces with summaries
    pub async fn list_traces(
        &self,
        service: Option<&str>,
        status: Option<&str>,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<TraceSummary>> {
        let mut conditions = vec!["parent_span_id IS NULL".to_string()];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(s) = status {
            conditions.push(format!("status = '{}'", s.replace('\'', "''")));
        }

        if let Some(start) = since {
            conditions.push(format!("started_at >= '{}'", start.format("%Y-%m-%d %H:%M:%S")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                s.trace_id,
                s.operation_name as root_operation,
                s.service_name,
                s.started_at,
                s.duration_ms,
                COALESCE(stats.span_count, 1) as span_count,
                COALESCE(stats.error_count, 0) as error_count,
                COALESCE(stats.total_tokens, 0) as total_tokens,
                COALESCE(stats.total_cost, 0) as total_cost_usd
            FROM spans s
            LEFT JOIN (
                SELECT
                    trace_id,
                    COUNT(*) as span_count,
                    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
                    SUM(COALESCE(tokens_in, 0) + COALESCE(tokens_out, 0)) as total_tokens,
                    SUM(COALESCE(cost_usd, 0)) as total_cost
                FROM spans
                GROUP BY trace_id
            ) stats ON s.trace_id = stats.trace_id
            WHERE {}
            ORDER BY s.started_at DESC
            LIMIT {}
            "#,
            where_clause, limit
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let mut traces = Vec::new();
        for row in rows {
            traces.push(TraceSummary {
                trace_id: row.try_get("trace_id").unwrap_or_default(),
                root_operation: row.try_get("root_operation").unwrap_or_default(),
                service_name: row.try_get("service_name").unwrap_or_default(),
                started_at: row.try_get("started_at").unwrap_or_else(|_| Utc::now()),
                duration_ms: row.try_get("duration_ms").ok(),
                span_count: row.try_get("span_count").unwrap_or(0),
                error_count: row.try_get("error_count").unwrap_or(0),
                total_tokens: row.try_get("total_tokens").unwrap_or(0),
                total_cost_usd: row.try_get::<f64, _>("total_cost_usd").unwrap_or(0.0),
            });
        }

        Ok(traces)
    }

    // =========================================================================
    // Metrics Methods
    // =========================================================================

    /// Get metrics summary
    pub async fn get_metrics_summary(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<MetricsSummaryResponse> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                COUNT(*) as total_spans,
                COUNT(DISTINCT trace_id) as total_traces,
                SUM(COALESCE(tokens_in, 0) + COALESCE(tokens_out, 0)) as total_tokens,
                SUM(COALESCE(cost_usd, 0)) as total_cost_usd,
                SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
                AVG(duration_ms) as avg_latency_ms,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY duration_ms) as p50_latency_ms,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_latency_ms,
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms) as p99_latency_ms
            FROM spans
            WHERE {}
            "#,
            where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let total_spans: i64 = row.try_get("total_spans").unwrap_or(0);
        let error_count: i64 = row.try_get("error_count").unwrap_or(0);

        Ok(MetricsSummaryResponse {
            total_spans,
            total_traces: row.try_get("total_traces").unwrap_or(0),
            total_tokens: row.try_get("total_tokens").unwrap_or(0),
            total_cost_usd: row.try_get::<f64, _>("total_cost_usd").unwrap_or(0.0),
            error_count,
            error_rate: if total_spans > 0 {
                error_count as f64 / total_spans as f64 * 100.0
            } else {
                0.0
            },
            avg_latency_ms: row.try_get::<f64, _>("avg_latency_ms").unwrap_or(0.0),
            p50_latency_ms: row.try_get::<f64, _>("p50_latency_ms").unwrap_or(0.0),
            p95_latency_ms: row.try_get::<f64, _>("p95_latency_ms").unwrap_or(0.0),
            p99_latency_ms: row.try_get::<f64, _>("p99_latency_ms").unwrap_or(0.0),
        })
    }

    /// Get cost metrics grouped by field
    pub async fn get_cost_by_group(
        &self,
        service: Option<&str>,
        group_by: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<CostMetric>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");
        let group_field = match group_by {
            "model" => "model_name",
            "service" => "service_name",
            "operation" => "operation_name",
            _ => "model_name",
        };

        let sql = format!(
            r#"
            SELECT
                COALESCE({}, 'unknown') as group_name,
                SUM(COALESCE(cost_usd, 0)) as total_cost_usd,
                SUM(COALESCE(tokens_in, 0) + COALESCE(tokens_out, 0)) as total_tokens,
                COUNT(*) as call_count
            FROM spans
            WHERE {}
            GROUP BY {}
            ORDER BY total_cost_usd DESC
            "#,
            group_field, where_clause, group_field
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let mut costs = Vec::new();
        for row in rows {
            costs.push(CostMetric {
                group: row.try_get("group_name").unwrap_or_default(),
                total_cost_usd: row.try_get::<f64, _>("total_cost_usd").unwrap_or(0.0),
                total_tokens: row.try_get("total_tokens").unwrap_or(0),
                call_count: row.try_get("call_count").unwrap_or(0),
            });
        }

        Ok(costs)
    }

    /// Get latency metrics over time
    pub async fn get_latency_over_time(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<LatencyMetric>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                time_bucket('1 hour', started_at) as bucket,
                AVG(duration_ms) as avg_ms,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY duration_ms) as p50_ms,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_ms,
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms) as p99_ms,
                COUNT(*) as count
            FROM spans
            WHERE {}
            GROUP BY bucket
            ORDER BY bucket
            "#,
            where_clause
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(LatencyMetric {
                timestamp: row.try_get("bucket").unwrap_or_else(|_| Utc::now()),
                avg_ms: row.try_get::<f64, _>("avg_ms").unwrap_or(0.0),
                p50_ms: row.try_get::<f64, _>("p50_ms").unwrap_or(0.0),
                p95_ms: row.try_get::<f64, _>("p95_ms").unwrap_or(0.0),
                p99_ms: row.try_get::<f64, _>("p99_ms").unwrap_or(0.0),
                count: row.try_get("count").unwrap_or(0),
            });
        }

        Ok(metrics)
    }

    /// Get error metrics over time
    pub async fn get_errors_over_time(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<ErrorMetric>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                time_bucket('1 hour', started_at) as bucket,
                SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
                COUNT(*) as total_count
            FROM spans
            WHERE {}
            GROUP BY bucket
            ORDER BY bucket
            "#,
            where_clause
        );

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let mut metrics = Vec::new();
        for row in rows {
            let error_count: i64 = row.try_get("error_count").unwrap_or(0);
            let total_count: i64 = row.try_get("total_count").unwrap_or(0);
            let error_rate = if total_count > 0 {
                error_count as f64 / total_count as f64 * 100.0
            } else {
                0.0
            };

            metrics.push(ErrorMetric {
                timestamp: row.try_get("bucket").unwrap_or_else(|_| Utc::now()),
                error_count,
                total_count,
                error_rate,
            });
        }

        Ok(metrics)
    }

    // =========================================================================
    // Alerting Metric Methods
    // =========================================================================

    /// Get error statistics for alerting
    pub async fn get_error_stats(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<ErrorStats> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT
                SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
                COUNT(*) as total,
                ARRAY_AGG(DISTINCT trace_id) FILTER (WHERE status = 'error') as sample_trace_ids
            FROM spans
            WHERE {}
            "#,
            where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(ErrorStats {
            error_count: row.try_get("error_count").unwrap_or(0),
            total: row.try_get("total").unwrap_or(0),
            sample_trace_ids: row.try_get::<Vec<String>, _>("sample_trace_ids").unwrap_or_default(),
        })
    }

    /// Get latency percentile for alerting
    pub async fn get_latency_percentile(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
        percentile: f64,
    ) -> Result<Option<f64>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
            "duration_ms IS NOT NULL".to_string(),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT PERCENTILE_CONT({}) WITHIN GROUP (ORDER BY duration_ms) as p_val
            FROM spans
            WHERE {}
            "#,
            percentile, where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.try_get::<f64, _>("p_val").ok())
    }

    /// Get average latency for alerting
    pub async fn get_latency_avg(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Option<f64>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
            "duration_ms IS NOT NULL".to_string(),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            "SELECT AVG(duration_ms) as avg_val FROM spans WHERE {}",
            where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.try_get::<f64, _>("avg_val").ok())
    }

    /// Get total cost for alerting
    pub async fn get_cost_sum(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Option<f64>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            "SELECT SUM(COALESCE(cost_usd, 0)) as total_cost FROM spans WHERE {}",
            where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.try_get::<f64, _>("total_cost").ok())
    }

    /// Get total token count for alerting
    pub async fn get_token_sum(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Option<i64>> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!(
            "SELECT SUM(COALESCE(tokens_in, 0) + COALESCE(tokens_out, 0)) as total_tokens FROM spans WHERE {}",
            where_clause
        );

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.try_get::<i64, _>("total_tokens").ok())
    }

    /// Get span count for alerting
    pub async fn get_span_count(
        &self,
        service: Option<&str>,
        model: Option<&str>,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<i64> {
        let mut conditions = vec![
            format!("started_at >= '{}'", since.format("%Y-%m-%d %H:%M:%S")),
            format!("started_at <= '{}'", until.format("%Y-%m-%d %H:%M:%S")),
        ];

        if let Some(svc) = service {
            conditions.push(format!("service_name = '{}'", svc.replace('\'', "''")));
        }

        if let Some(m) = model {
            conditions.push(format!("model_name = '{}'", m.replace('\'', "''")));
        }

        let where_clause = conditions.join(" AND ");

        let sql = format!("SELECT COUNT(*) as cnt FROM spans WHERE {}", where_clause);

        let row = sqlx::query(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.try_get("cnt").unwrap_or(0))
    }
}

fn span_status_to_str(status: &SpanStatus) -> &'static str {
    match status {
        SpanStatus::Ok => "ok",
        SpanStatus::Error => "error",
        SpanStatus::Unset => "unset",
    }
}

fn span_kind_to_str(kind: &SpanKind) -> &'static str {
    match kind {
        SpanKind::Internal => "internal",
        SpanKind::Client => "client",
        SpanKind::Server => "server",
        SpanKind::Producer => "producer",
        SpanKind::Consumer => "consumer",
    }
}

fn row_to_span(row: &sqlx::postgres::PgRow) -> Result<Span> {
    Ok(Span {
        id: row.try_get("id").map_err(|e| Error::Database(e.to_string()))?,
        span_id: row.try_get("span_id").map_err(|e| Error::Database(e.to_string()))?,
        trace_id: row.try_get("trace_id").map_err(|e| Error::Database(e.to_string()))?,
        parent_span_id: row.try_get("parent_span_id").ok(),
        operation_name: row.try_get("operation_name").map_err(|e| Error::Database(e.to_string()))?,
        service_name: row.try_get("service_name").unwrap_or_default(),
        span_kind: SpanKind::Internal, // TODO: parse from DB
        started_at: row.try_get("started_at").map_err(|e| Error::Database(e.to_string()))?,
        ended_at: row.try_get("ended_at").ok(),
        duration_ms: row.try_get("duration_ms").ok(),
        status: SpanStatus::Ok, // TODO: parse from DB
        status_message: row.try_get("status_message").ok(),
        model_name: row.try_get("model_name").ok(),
        model_provider: row.try_get("model_provider").ok(),
        tokens_in: row.try_get("tokens_in").ok(),
        tokens_out: row.try_get("tokens_out").ok(),
        tokens_reasoning: row.try_get("tokens_reasoning").ok(),
        cost_usd: row.try_get("cost_usd").ok(),
        tool_name: row.try_get("tool_name").ok(),
        tool_input: row.try_get("tool_input").ok(),
        tool_output: row.try_get("tool_output").ok(),
        tool_duration_ms: row.try_get("tool_duration_ms").ok(),
        prompt_preview: row.try_get("prompt_preview").ok(),
        completion_preview: row.try_get("completion_preview").ok(),
        attributes: row.try_get("attributes").unwrap_or_default(),
        events: vec![],
        links: vec![],
    })
}

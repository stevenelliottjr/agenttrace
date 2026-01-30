//! PostgreSQL/TimescaleDB connection and queries

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use uuid::Uuid;

use crate::config::DatabaseConfig;
use crate::error::{Error, Result};
use crate::models::{Span, SpanInput, SpanStatus, SpanKind};

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

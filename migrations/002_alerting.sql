-- Alert rules table
CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    -- Scope
    service_name TEXT,
    environment TEXT,
    model_name TEXT,

    -- Condition
    condition_type TEXT NOT NULL DEFAULT 'threshold',
    metric TEXT NOT NULL,
    operator TEXT NOT NULL DEFAULT 'gt',
    threshold DOUBLE PRECISION,

    -- Evaluation
    window_minutes INTEGER NOT NULL DEFAULT 5,
    evaluation_interval_seconds INTEGER NOT NULL DEFAULT 60,
    consecutive_failures INTEGER NOT NULL DEFAULT 1,

    -- Notification
    severity TEXT NOT NULL DEFAULT 'warning',
    notification_channels JSONB NOT NULL DEFAULT '[]',

    -- State
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_evaluated_at TIMESTAMPTZ,
    last_triggered_at TIMESTAMPTZ,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules (enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_alert_rules_service ON alert_rules (service_name);
CREATE INDEX IF NOT EXISTS idx_alert_rules_metric ON alert_rules (metric);

-- Alert events table
CREATE TABLE IF NOT EXISTS alert_events (
    id UUID PRIMARY KEY,
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,

    status TEXT NOT NULL DEFAULT 'active',
    severity TEXT NOT NULL DEFAULT 'warning',
    message TEXT NOT NULL,

    metric_value DOUBLE PRECISION NOT NULL,
    threshold_value DOUBLE PRECISION NOT NULL,

    service_name TEXT,
    trace_ids JSONB NOT NULL DEFAULT '[]',
    notifications_sent JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_alert_events_rule_id ON alert_events (rule_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_status ON alert_events (status) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_alert_events_triggered_at ON alert_events (triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_severity ON alert_events (severity, triggered_at DESC);

-- Search optimization for full-text search on spans
CREATE INDEX IF NOT EXISTS idx_spans_operation_name_gin ON spans USING gin (to_tsvector('english', operation_name));
CREATE INDEX IF NOT EXISTS idx_spans_service_name_gin ON spans USING gin (to_tsvector('english', service_name));

-- Combined text search index
CREATE INDEX IF NOT EXISTS idx_spans_fulltext ON spans USING gin (
    to_tsvector('english',
        COALESCE(operation_name, '') || ' ' ||
        COALESCE(service_name, '') || ' ' ||
        COALESCE(model_name, '') || ' ' ||
        COALESCE(tool_name, '')
    )
);

-- Additional indexes for filtering
CREATE INDEX IF NOT EXISTS idx_spans_model_name ON spans (model_name) WHERE model_name IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spans_model_provider ON spans (model_provider) WHERE model_provider IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spans_status ON spans (status);
CREATE INDEX IF NOT EXISTS idx_spans_cost ON spans (cost_usd) WHERE cost_usd IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spans_duration ON spans (duration_ms) WHERE duration_ms IS NOT NULL;

-- Composite index for common filter combinations
CREATE INDEX IF NOT EXISTS idx_spans_service_status_time ON spans (service_name, status, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_spans_model_time ON spans (model_name, started_at DESC) WHERE model_name IS NOT NULL;

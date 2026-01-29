-- AgentTrace Initial Schema
-- Migration: 001_initial_schema.sql
-- Description: Core tables for traces, spans, metrics, and alerts

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- TRACES TABLE
-- Parent container for related spans
-- ============================================================================
CREATE TABLE traces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trace_id VARCHAR(32) NOT NULL UNIQUE,
    service_name VARCHAR(255) NOT NULL,
    environment VARCHAR(50) DEFAULT 'production',
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms DOUBLE PRECISION,
    status VARCHAR(20) DEFAULT 'in_progress' CHECK (status IN ('ok', 'error', 'in_progress')),
    root_span_id VARCHAR(32),
    
    -- Aggregated metrics
    total_tokens_in INTEGER DEFAULT 0,
    total_tokens_out INTEGER DEFAULT 0,
    total_cost_usd DECIMAL(10, 6) DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    span_count INTEGER DEFAULT 0,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    tags VARCHAR(100)[] DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_traces_trace_id ON traces(trace_id);
CREATE INDEX idx_traces_service_name ON traces(service_name);
CREATE INDEX idx_traces_environment ON traces(environment);
CREATE INDEX idx_traces_started_at ON traces(started_at DESC);
CREATE INDEX idx_traces_status ON traces(status);
CREATE INDEX idx_traces_tags ON traces USING GIN(tags);
CREATE INDEX idx_traces_metadata ON traces USING GIN(metadata);

-- ============================================================================
-- SPANS TABLE (Hypertable)
-- Individual operations within a trace
-- ============================================================================
CREATE TABLE spans (
    id UUID DEFAULT uuid_generate_v4(),
    span_id VARCHAR(32) NOT NULL,
    trace_id VARCHAR(32) NOT NULL,
    parent_span_id VARCHAR(32),
    
    -- Core span data
    operation_name VARCHAR(255) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    span_kind VARCHAR(20) DEFAULT 'internal' CHECK (span_kind IN ('internal', 'client', 'server', 'producer', 'consumer')),
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_ms DOUBLE PRECISION,
    
    -- Status
    status VARCHAR(20) DEFAULT 'ok' CHECK (status IN ('ok', 'error', 'unset')),
    status_message TEXT,
    
    -- AI-specific fields
    model_name VARCHAR(100),
    model_provider VARCHAR(50),
    tokens_in INTEGER,
    tokens_out INTEGER,
    tokens_reasoning INTEGER,  -- For o1-style thinking tokens
    cost_usd DECIMAL(10, 6),
    
    -- Tool usage
    tool_name VARCHAR(100),
    tool_input JSONB,
    tool_output JSONB,
    tool_duration_ms DOUBLE PRECISION,
    
    -- Prompt/completion (optional, can be large)
    prompt_preview TEXT,       -- First 500 chars of prompt
    completion_preview TEXT,   -- First 500 chars of completion
    
    -- Attributes and events
    attributes JSONB DEFAULT '{}',
    events JSONB DEFAULT '[]',
    links JSONB DEFAULT '[]',
    
    -- Sampling
    is_sampled BOOLEAN DEFAULT true,
    sample_rate DOUBLE PRECISION DEFAULT 1.0,
    
    PRIMARY KEY (span_id, started_at)
);

-- Convert to hypertable for time-series optimization
SELECT create_hypertable('spans', 'started_at', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Indexes for common query patterns
CREATE INDEX idx_spans_trace_id ON spans(trace_id, started_at DESC);
CREATE INDEX idx_spans_parent_span_id ON spans(parent_span_id) WHERE parent_span_id IS NOT NULL;
CREATE INDEX idx_spans_operation_name ON spans(operation_name, started_at DESC);
CREATE INDEX idx_spans_service_name ON spans(service_name, started_at DESC);
CREATE INDEX idx_spans_model_name ON spans(model_name) WHERE model_name IS NOT NULL;
CREATE INDEX idx_spans_model_provider ON spans(model_provider) WHERE model_provider IS NOT NULL;
CREATE INDEX idx_spans_tool_name ON spans(tool_name) WHERE tool_name IS NOT NULL;
CREATE INDEX idx_spans_status ON spans(status, started_at DESC);
CREATE INDEX idx_spans_attributes ON spans USING GIN(attributes);

-- ============================================================================
-- SPAN EVENTS TABLE (Hypertable)
-- Point-in-time events within spans
-- ============================================================================
CREATE TABLE span_events (
    id UUID DEFAULT uuid_generate_v4(),
    span_id VARCHAR(32) NOT NULL,
    trace_id VARCHAR(32) NOT NULL,
    name VARCHAR(255) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    attributes JSONB DEFAULT '{}',
    PRIMARY KEY (id, timestamp)
);

SELECT create_hypertable('span_events', 'timestamp',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

CREATE INDEX idx_span_events_span_id ON span_events(span_id, timestamp DESC);
CREATE INDEX idx_span_events_trace_id ON span_events(trace_id, timestamp DESC);
CREATE INDEX idx_span_events_name ON span_events(name);

-- ============================================================================
-- METRICS HOURLY TABLE (Hypertable)
-- Pre-aggregated metrics for fast dashboard queries
-- ============================================================================
CREATE TABLE metrics_hourly (
    time TIMESTAMPTZ NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    environment VARCHAR(50) DEFAULT 'production',
    model_name VARCHAR(100),
    model_provider VARCHAR(50),
    operation_name VARCHAR(255),
    
    -- Counters
    request_count BIGINT DEFAULT 0,
    error_count BIGINT DEFAULT 0,
    tool_call_count BIGINT DEFAULT 0,
    
    -- Token metrics
    tokens_in_sum BIGINT DEFAULT 0,
    tokens_out_sum BIGINT DEFAULT 0,
    tokens_reasoning_sum BIGINT DEFAULT 0,
    tokens_in_avg DOUBLE PRECISION,
    tokens_out_avg DOUBLE PRECISION,
    tokens_in_max INTEGER,
    tokens_out_max INTEGER,
    
    -- Cost metrics
    cost_sum DECIMAL(12, 6) DEFAULT 0,
    cost_avg DECIMAL(10, 6),
    cost_max DECIMAL(10, 6),
    
    -- Latency metrics
    latency_sum_ms DOUBLE PRECISION DEFAULT 0,
    latency_avg_ms DOUBLE PRECISION,
    latency_min_ms DOUBLE PRECISION,
    latency_max_ms DOUBLE PRECISION,
    latency_p50_ms DOUBLE PRECISION,
    latency_p90_ms DOUBLE PRECISION,
    latency_p95_ms DOUBLE PRECISION,
    latency_p99_ms DOUBLE PRECISION,
    
    PRIMARY KEY (time, service_name, environment, model_name, operation_name)
);

SELECT create_hypertable('metrics_hourly', 'time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

CREATE INDEX idx_metrics_hourly_service ON metrics_hourly(service_name, time DESC);
CREATE INDEX idx_metrics_hourly_model ON metrics_hourly(model_name, time DESC) WHERE model_name IS NOT NULL;

-- ============================================================================
-- CONTINUOUS AGGREGATE FOR REAL-TIME METRICS
-- Auto-updated materialized view
-- ============================================================================
CREATE MATERIALIZED VIEW metrics_5min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', started_at) AS bucket,
    service_name,
    model_name,
    model_provider,
    COUNT(*) as request_count,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
    SUM(COALESCE(tokens_in, 0)) as tokens_in_sum,
    SUM(COALESCE(tokens_out, 0)) as tokens_out_sum,
    SUM(COALESCE(cost_usd, 0)) as cost_sum,
    AVG(duration_ms) as latency_avg_ms,
    MIN(duration_ms) as latency_min_ms,
    MAX(duration_ms) as latency_max_ms,
    percentile_cont(0.50) WITHIN GROUP (ORDER BY duration_ms) as latency_p50_ms,
    percentile_cont(0.90) WITHIN GROUP (ORDER BY duration_ms) as latency_p90_ms,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) as latency_p95_ms,
    percentile_cont(0.99) WITHIN GROUP (ORDER BY duration_ms) as latency_p99_ms
FROM spans
WHERE started_at > NOW() - INTERVAL '7 days'
GROUP BY bucket, service_name, model_name, model_provider
WITH NO DATA;

-- Refresh policy: update every 5 minutes
SELECT add_continuous_aggregate_policy('metrics_5min',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes',
    if_not_exists => TRUE
);

-- ============================================================================
-- ALERT RULES TABLE
-- Configurable alerting conditions
-- ============================================================================
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Scope
    service_name VARCHAR(255),  -- NULL = all services
    environment VARCHAR(50),
    model_name VARCHAR(100),
    
    -- Condition
    condition_type VARCHAR(50) NOT NULL CHECK (condition_type IN ('threshold', 'anomaly', 'rate_change', 'absence')),
    metric VARCHAR(100) NOT NULL,  -- e.g., 'error_rate', 'latency_p99', 'cost_sum', 'token_usage'
    operator VARCHAR(10) NOT NULL CHECK (operator IN ('gt', 'lt', 'eq', 'gte', 'lte', 'ne')),
    threshold DOUBLE PRECISION,
    
    -- Evaluation
    window_minutes INTEGER DEFAULT 5,
    evaluation_interval_seconds INTEGER DEFAULT 60,
    consecutive_failures INTEGER DEFAULT 1,  -- How many consecutive failures before alert
    
    -- Severity and notifications
    severity VARCHAR(20) DEFAULT 'warning' CHECK (severity IN ('info', 'warning', 'critical')),
    notification_channels JSONB DEFAULT '[]',  -- [{type: "slack", webhook: "..."}, {type: "email", to: "..."}]
    
    -- State
    enabled BOOLEAN DEFAULT true,
    last_evaluated_at TIMESTAMPTZ,
    last_triggered_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(255)
);

CREATE INDEX idx_alert_rules_enabled ON alert_rules(enabled) WHERE enabled = true;
CREATE INDEX idx_alert_rules_service ON alert_rules(service_name) WHERE service_name IS NOT NULL;

-- ============================================================================
-- ALERT EVENTS TABLE (Hypertable)
-- History of triggered alerts
-- ============================================================================
CREATE TABLE alert_events (
    id UUID DEFAULT uuid_generate_v4(),
    rule_id UUID REFERENCES alert_rules(id) ON DELETE CASCADE,
    
    triggered_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    
    severity VARCHAR(20),
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'acknowledged', 'resolved')),
    
    -- Context
    message TEXT,
    metric_value DOUBLE PRECISION,
    threshold_value DOUBLE PRECISION,
    
    -- Affected resources
    service_name VARCHAR(255),
    trace_ids VARCHAR(32)[],  -- Sample trace IDs that triggered this
    
    -- Notification tracking
    notifications_sent JSONB DEFAULT '[]',
    
    metadata JSONB DEFAULT '{}',
    
    PRIMARY KEY (id, triggered_at)
);

SELECT create_hypertable('alert_events', 'triggered_at',
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => TRUE
);

CREATE INDEX idx_alert_events_rule ON alert_events(rule_id, triggered_at DESC);
CREATE INDEX idx_alert_events_status ON alert_events(status, triggered_at DESC);
CREATE INDEX idx_alert_events_severity ON alert_events(severity, triggered_at DESC);

-- ============================================================================
-- MODEL PRICING TABLE
-- Reference table for cost calculations
-- ============================================================================
CREATE TABLE model_pricing (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider VARCHAR(50) NOT NULL,
    model_name VARCHAR(100) NOT NULL,
    model_pattern VARCHAR(200),  -- Regex pattern for model name matching
    
    -- Pricing (per 1K tokens)
    input_cost_per_1k DECIMAL(10, 8) NOT NULL,
    output_cost_per_1k DECIMAL(10, 8) NOT NULL,
    reasoning_cost_per_1k DECIMAL(10, 8),  -- For o1-style models
    
    -- Validity
    effective_date DATE NOT NULL,
    deprecated_date DATE,
    
    -- Metadata
    notes TEXT,
    source_url TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(provider, model_name, effective_date)
);

CREATE INDEX idx_model_pricing_lookup ON model_pricing(provider, model_name, effective_date DESC);
CREATE INDEX idx_model_pricing_active ON model_pricing(provider, effective_date DESC) WHERE deprecated_date IS NULL;

-- Insert common model pricing (as of early 2025)
INSERT INTO model_pricing (provider, model_name, input_cost_per_1k, output_cost_per_1k, reasoning_cost_per_1k, effective_date) VALUES
-- Anthropic
('anthropic', 'claude-3-5-sonnet-20241022', 0.003, 0.015, NULL, '2024-10-22'),
('anthropic', 'claude-3-5-haiku-20241022', 0.001, 0.005, NULL, '2024-10-22'),
('anthropic', 'claude-3-opus-20240229', 0.015, 0.075, NULL, '2024-02-29'),
('anthropic', 'claude-sonnet-4-20250514', 0.003, 0.015, NULL, '2025-05-14'),
('anthropic', 'claude-opus-4-5-20251101', 0.015, 0.075, NULL, '2025-11-01'),
-- OpenAI
('openai', 'gpt-4o', 0.005, 0.015, NULL, '2024-05-13'),
('openai', 'gpt-4o-mini', 0.00015, 0.0006, NULL, '2024-07-18'),
('openai', 'gpt-4-turbo', 0.01, 0.03, NULL, '2024-04-09'),
('openai', 'gpt-4', 0.03, 0.06, NULL, '2023-03-14'),
('openai', 'gpt-3.5-turbo', 0.0005, 0.0015, NULL, '2023-03-01'),
('openai', 'o1', 0.015, 0.06, 0.06, '2024-12-05'),
('openai', 'o1-mini', 0.003, 0.012, 0.012, '2024-09-12'),
('openai', 'o1-pro', 0.15, 0.6, 0.6, '2024-12-05'),
-- Google
('google', 'gemini-1.5-pro', 0.00125, 0.005, NULL, '2024-05-14'),
('google', 'gemini-1.5-flash', 0.000075, 0.0003, NULL, '2024-05-14'),
('google', 'gemini-2.0-flash', 0.0001, 0.0004, NULL, '2024-12-11');

-- ============================================================================
-- SERVICES TABLE
-- Registered services for quick lookups
-- ============================================================================
CREATE TABLE services (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    description TEXT,
    
    -- Contact
    owner VARCHAR(255),
    team VARCHAR(255),
    
    -- Settings
    default_sampling_rate DOUBLE PRECISION DEFAULT 1.0,
    retention_days INTEGER DEFAULT 30,
    
    -- Stats (updated periodically)
    first_seen_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    total_traces BIGINT DEFAULT 0,
    
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_services_name ON services(name);
CREATE INDEX idx_services_last_seen ON services(last_seen_at DESC);

-- ============================================================================
-- API KEYS TABLE
-- For SDK authentication
-- ============================================================================
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_hash VARCHAR(64) NOT NULL UNIQUE,  -- SHA256 hash of the key
    key_prefix VARCHAR(12) NOT NULL,       -- First 8 chars for identification
    
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Scope
    service_name VARCHAR(255),  -- NULL = all services
    permissions VARCHAR(50)[] DEFAULT ARRAY['write:spans', 'read:traces'],
    
    -- Limits
    rate_limit_per_minute INTEGER,
    
    -- State
    enabled BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT DEFAULT 0,
    
    -- Expiry
    expires_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(255)
);

CREATE INDEX idx_api_keys_hash ON api_keys(key_hash) WHERE enabled = true;
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);

-- ============================================================================
-- RETENTION POLICIES
-- Auto-delete old data
-- ============================================================================

-- Keep spans for 30 days by default
SELECT add_retention_policy('spans', INTERVAL '30 days', if_not_exists => TRUE);

-- Keep span events for 30 days
SELECT add_retention_policy('span_events', INTERVAL '30 days', if_not_exists => TRUE);

-- Keep hourly metrics for 90 days
SELECT add_retention_policy('metrics_hourly', INTERVAL '90 days', if_not_exists => TRUE);

-- Keep alert events for 1 year
SELECT add_retention_policy('alert_events', INTERVAL '365 days', if_not_exists => TRUE);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Function to calculate cost for a span
CREATE OR REPLACE FUNCTION calculate_span_cost(
    p_model_provider VARCHAR,
    p_model_name VARCHAR,
    p_tokens_in INTEGER,
    p_tokens_out INTEGER,
    p_tokens_reasoning INTEGER DEFAULT NULL
) RETURNS DECIMAL(10, 6) AS $$
DECLARE
    v_pricing model_pricing%ROWTYPE;
    v_cost DECIMAL(10, 6);
BEGIN
    -- Get the most recent pricing for this model
    SELECT * INTO v_pricing
    FROM model_pricing
    WHERE provider = p_model_provider
      AND model_name = p_model_name
      AND effective_date <= CURRENT_DATE
      AND (deprecated_date IS NULL OR deprecated_date > CURRENT_DATE)
    ORDER BY effective_date DESC
    LIMIT 1;
    
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    
    -- Calculate cost
    v_cost := (COALESCE(p_tokens_in, 0)::DECIMAL / 1000.0) * v_pricing.input_cost_per_1k
            + (COALESCE(p_tokens_out, 0)::DECIMAL / 1000.0) * v_pricing.output_cost_per_1k;
    
    -- Add reasoning token cost if applicable
    IF p_tokens_reasoning IS NOT NULL AND v_pricing.reasoning_cost_per_1k IS NOT NULL THEN
        v_cost := v_cost + (p_tokens_reasoning::DECIMAL / 1000.0) * v_pricing.reasoning_cost_per_1k;
    END IF;
    
    RETURN v_cost;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to update trace aggregates
CREATE OR REPLACE FUNCTION update_trace_aggregates(p_trace_id VARCHAR) RETURNS void AS $$
BEGIN
    UPDATE traces t SET
        total_tokens_in = (SELECT COALESCE(SUM(tokens_in), 0) FROM spans WHERE trace_id = p_trace_id),
        total_tokens_out = (SELECT COALESCE(SUM(tokens_out), 0) FROM spans WHERE trace_id = p_trace_id),
        total_cost_usd = (SELECT COALESCE(SUM(cost_usd), 0) FROM spans WHERE trace_id = p_trace_id),
        error_count = (SELECT COUNT(*) FROM spans WHERE trace_id = p_trace_id AND status = 'error'),
        span_count = (SELECT COUNT(*) FROM spans WHERE trace_id = p_trace_id),
        ended_at = (SELECT MAX(ended_at) FROM spans WHERE trace_id = p_trace_id),
        duration_ms = EXTRACT(EPOCH FROM (
            (SELECT MAX(ended_at) FROM spans WHERE trace_id = p_trace_id) - t.started_at
        )) * 1000,
        status = CASE 
            WHEN EXISTS (SELECT 1 FROM spans WHERE trace_id = p_trace_id AND status = 'error') THEN 'error'
            WHEN EXISTS (SELECT 1 FROM spans WHERE trace_id = p_trace_id AND ended_at IS NULL) THEN 'in_progress'
            ELSE 'ok'
        END,
        updated_at = NOW()
    WHERE trace_id = p_trace_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER traces_updated_at BEFORE UPDATE ON traces
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER alert_rules_updated_at BEFORE UPDATE ON alert_rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER services_updated_at BEFORE UPDATE ON services
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================================
-- SEED DATA FOR DEVELOPMENT
-- ============================================================================

-- Create a default API key for development (key: at_dev_test123456789)
-- In production, generate proper keys via the API
INSERT INTO api_keys (key_hash, key_prefix, name, description, permissions)
VALUES (
    '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',  -- SHA256 of 'at_dev_test123456789'
    'at_dev_te',
    'Development Key',
    'Default key for local development. DO NOT use in production.',
    ARRAY['write:spans', 'read:traces', 'read:metrics', 'manage:alerts']
);

-- Create sample service entries
INSERT INTO services (name, display_name, description, owner)
VALUES 
    ('demo-agent', 'Demo Agent', 'Sample agent for testing', 'dev-team'),
    ('claude-code-agent', 'Claude Code Agent', 'Claude Code integration', 'platform-team');

COMMENT ON TABLE traces IS 'Parent records for distributed traces';
COMMENT ON TABLE spans IS 'Individual operations within traces (hypertable)';
COMMENT ON TABLE metrics_hourly IS 'Pre-aggregated metrics for dashboards';
COMMENT ON TABLE alert_rules IS 'User-defined alerting conditions';
COMMENT ON TABLE model_pricing IS 'LLM pricing reference for cost calculations';

"""Tests for AgentTrace models."""

from datetime import datetime, timezone, timedelta

from agenttrace.models import Span, SpanKind, SpanStatus, SpanType, SpanEvent


def test_span_creation():
    """Test basic span creation."""
    span = Span(operation_name="test_operation")

    assert span.operation_name == "test_operation"
    assert span.service_name == "default"
    assert span.status == SpanStatus.UNSET
    assert span.span_id is not None
    assert span.trace_id is not None
    assert span.started_at is not None
    assert span.ended_at is None


def test_span_with_all_fields():
    """Test span with all fields populated."""
    span = Span(
        operation_name="llm_call",
        service_name="my-agent",
        span_kind=SpanKind.CLIENT,
        span_type=SpanType.LLM_CALL,
        model_name="claude-3-opus",
        model_provider="anthropic",
        tokens_in=100,
        tokens_out=500,
        prompt_preview="Hello, world!",
        completion_preview="Hi there!",
    )

    assert span.model_name == "claude-3-opus"
    assert span.model_provider == "anthropic"
    assert span.tokens_in == 100
    assert span.tokens_out == 500


def test_span_end():
    """Test ending a span."""
    span = Span(operation_name="test")

    assert span.ended_at is None
    span.end(SpanStatus.OK)

    assert span.ended_at is not None
    assert span.status == SpanStatus.OK


def test_span_duration():
    """Test span duration calculation."""
    start = datetime.now(timezone.utc)
    span = Span(operation_name="test", started_at=start)
    span.ended_at = start + timedelta(milliseconds=150)

    assert span.duration_ms is not None
    assert 149 < span.duration_ms < 151


def test_span_set_error():
    """Test setting error on span."""
    span = Span(operation_name="test")
    error = ValueError("test error")
    span.set_error(error)

    assert span.status == SpanStatus.ERROR
    assert span.status_message == "test error"
    assert span.attributes["error.type"] == "ValueError"


def test_span_add_event():
    """Test adding events to span."""
    span = Span(operation_name="test")
    span.add_event("checkpoint", {"step": 1})

    assert len(span.events) == 1
    assert span.events[0].name == "checkpoint"
    assert span.events[0].attributes["step"] == 1


def test_span_set_attribute():
    """Test setting custom attributes."""
    span = Span(operation_name="test")
    span.set_attribute("custom_key", "custom_value")
    span.set_attribute("numeric", 42)

    assert span.attributes["custom_key"] == "custom_value"
    assert span.attributes["numeric"] == 42


def test_span_to_dict():
    """Test span serialization to dict."""
    span = Span(
        operation_name="test",
        model_name="gpt-4",
        tokens_in=100,
    )
    span.end()

    data = span.to_dict()

    assert data["operation_name"] == "test"
    assert data["model_name"] == "gpt-4"
    assert data["tokens_in"] == 100
    assert "started_at" in data
    assert "ended_at" in data
    # None values should be excluded
    assert "parent_span_id" not in data
    assert "tokens_reasoning" not in data


def test_span_event_creation():
    """Test SpanEvent creation."""
    event = SpanEvent(name="test_event", attributes={"key": "value"})

    assert event.name == "test_event"
    assert event.attributes["key"] == "value"
    assert event.timestamp is not None


def test_span_kind_values():
    """Test SpanKind enum values."""
    assert SpanKind.INTERNAL.value == "internal"
    assert SpanKind.CLIENT.value == "client"
    assert SpanKind.SERVER.value == "server"


def test_span_status_values():
    """Test SpanStatus enum values."""
    assert SpanStatus.UNSET.value == "unset"
    assert SpanStatus.OK.value == "ok"
    assert SpanStatus.ERROR.value == "error"


def test_span_type_values():
    """Test SpanType enum values."""
    assert SpanType.LLM_CALL.value == "llm_call"
    assert SpanType.TOOL_CALL.value == "tool_call"
    assert SpanType.AGENT_STEP.value == "agent_step"

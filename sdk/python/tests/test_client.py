"""Tests for AgentTrace client."""

import asyncio
from unittest.mock import AsyncMock, MagicMock

import pytest

from agenttrace import AgentTrace, Span, SpanStatus, SpanType
from agenttrace.context import get_current_span, get_current_trace_id
from agenttrace.exporters.base import Exporter


class MockExporter(Exporter):
    """Mock exporter for testing."""

    def __init__(self):
        self.exported_spans: list[Span] = []
        self.export_called = 0
        self.batch_export_called = 0

    async def export(self, span: Span) -> bool:
        self.export_called += 1
        self.exported_spans.append(span)
        return True

    async def export_batch(self, spans: list[Span]) -> int:
        self.batch_export_called += 1
        self.exported_spans.extend(spans)
        return len(spans)


@pytest.fixture
def mock_exporter():
    return MockExporter()


@pytest.fixture
def tracer(mock_exporter):
    # Reset global instance
    AgentTrace._instance = None
    return AgentTrace(
        service_name="test-service",
        exporter=mock_exporter,
        flush_interval=60.0,  # Long interval to prevent auto-flush
    )


def test_tracer_initialization(tracer):
    """Test tracer initialization."""
    assert tracer.service_name == "test-service"
    assert tracer.exporter is not None


def test_start_span(tracer):
    """Test starting a new span."""
    span = tracer.start_span("test_operation")

    assert span.operation_name == "test_operation"
    assert span.service_name == "test-service"
    assert span.trace_id is not None
    assert span.span_id is not None


def test_span_context_manager(tracer, mock_exporter):
    """Test span context manager."""
    with tracer.span("test_operation") as span:
        assert get_current_span() == span
        span.set_attribute("key", "value")

    # Span should be ended and exported
    tracer.flush()
    assert len(mock_exporter.exported_spans) >= 1
    exported = mock_exporter.exported_spans[-1]
    assert exported.status == SpanStatus.OK
    assert exported.ended_at is not None


def test_nested_spans(tracer):
    """Test nested span context with parent linking."""
    with tracer.span("parent") as parent_span:
        parent_trace_id = parent_span.trace_id

        with tracer.span("child") as child_span:
            # Child should inherit parent's trace_id
            assert child_span.trace_id == parent_trace_id
            # Child should have parent as parent_span_id
            assert child_span.parent_span_id == parent_span.span_id


def test_error_handling_in_span(tracer, mock_exporter):
    """Test error handling in span context."""
    try:
        with tracer.span("failing_operation") as span:
            raise ValueError("test error")
    except ValueError:
        pass

    tracer.flush()
    exported = mock_exporter.exported_spans[-1]
    assert exported.status == SpanStatus.ERROR
    assert "test error" in exported.status_message
    assert exported.attributes.get("error.type") == "ValueError"


def test_trace_decorator_sync(tracer, mock_exporter):
    """Test @trace decorator on sync function."""

    @tracer.trace("decorated_function")
    def my_function(x: int) -> int:
        return x * 2

    result = my_function(5)

    assert result == 10
    tracer.flush()
    assert len(mock_exporter.exported_spans) >= 1


def test_trace_decorator_async(tracer, mock_exporter):
    """Test @trace decorator on async function."""

    @tracer.trace("async_function")
    async def my_async_function(x: int) -> int:
        await asyncio.sleep(0.01)
        return x * 2

    result = asyncio.run(my_async_function(5))

    assert result == 10
    tracer.flush()
    assert len(mock_exporter.exported_spans) >= 1


def test_trace_decorator_with_error(tracer, mock_exporter):
    """Test @trace decorator handles errors."""

    @tracer.trace("failing_function")
    def failing_function():
        raise RuntimeError("oops")

    with pytest.raises(RuntimeError):
        failing_function()

    tracer.flush()
    exported = mock_exporter.exported_spans[-1]
    assert exported.status == SpanStatus.ERROR


def test_span_attributes(tracer):
    """Test setting span attributes."""
    span = tracer.start_span(
        "test",
        attributes={"initial": "value"},
    )

    span.set_attribute("key1", "value1")
    span.set_attribute("key2", 42)

    assert span.attributes["initial"] == "value"
    assert span.attributes["key1"] == "value1"
    assert span.attributes["key2"] == 42


def test_llm_span_fields(tracer):
    """Test LLM-specific span fields."""
    with tracer.span("llm_call", span_type=SpanType.LLM_CALL) as span:
        span.model_name = "claude-3-opus"
        span.model_provider = "anthropic"
        span.tokens_in = 100
        span.tokens_out = 500
        span.prompt_preview = "Hello"
        span.completion_preview = "Hi there!"

    assert span.model_name == "claude-3-opus"
    assert span.tokens_in == 100
    assert span.tokens_out == 500


def test_tool_span_fields(tracer):
    """Test tool-specific span fields."""
    with tracer.span("tool_call", span_type=SpanType.TOOL_CALL) as span:
        span.tool_name = "calculator"
        span.tool_input = {"expression": "2+2"}
        span.tool_output = {"result": 4}

    assert span.tool_name == "calculator"
    assert span.tool_input["expression"] == "2+2"
    assert span.tool_output["result"] == 4


def test_configure_singleton():
    """Test AgentTrace.configure() creates singleton."""
    AgentTrace._instance = None

    tracer1 = AgentTrace.configure(service_name="service1")
    tracer2 = AgentTrace.configure(service_name="service2")

    # Should return same instance
    assert tracer1 is tracer2
    # First configuration wins
    assert tracer1.service_name == "service1"


def test_get_instance():
    """Test AgentTrace.get_instance()."""
    AgentTrace._instance = None

    assert AgentTrace.get_instance() is None

    tracer = AgentTrace.configure(service_name="test")
    assert AgentTrace.get_instance() is tracer

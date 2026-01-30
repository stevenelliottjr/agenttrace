"""AgentTrace - Observability SDK for AI Agents.

Example usage:
    from agenttrace import AgentTrace, Span
    from agenttrace.integrations import instrument_anthropic

    # Configure the global tracer
    tracer = AgentTrace.configure(
        service_name="my-ai-agent",
        endpoint="http://localhost:8080",
    )

    # Auto-instrument Anthropic SDK
    instrument_anthropic(tracer)

    # Or manually trace operations
    with tracer.span("process_request") as span:
        span.set_attribute("user_id", "123")
        result = do_work()

    # Using decorator
    @tracer.trace("my_function")
    def my_function():
        pass
"""

__version__ = "0.1.0"

from agenttrace.client import AgentTrace
from agenttrace.models import Span, SpanEvent, SpanKind, SpanStatus, SpanType
from agenttrace.context import get_current_span, get_current_trace_id
from agenttrace.exporters import ConsoleExporter, HttpExporter, Exporter

__all__ = [
    "__version__",
    # Main client
    "AgentTrace",
    # Models
    "Span",
    "SpanEvent",
    "SpanKind",
    "SpanStatus",
    "SpanType",
    # Context
    "get_current_span",
    "get_current_trace_id",
    # Exporters
    "Exporter",
    "ConsoleExporter",
    "HttpExporter",
]


def configure(
    service_name: str = "default",
    endpoint: str = "http://localhost:8080",
    **kwargs,
) -> AgentTrace:
    """Configure and return the global AgentTrace instance.

    This is a convenience function equivalent to AgentTrace.configure().

    Args:
        service_name: Name of the service for all spans.
        endpoint: Collector endpoint URL.
        **kwargs: Additional arguments passed to AgentTrace.

    Returns:
        The configured AgentTrace instance.
    """
    return AgentTrace.configure(service_name=service_name, endpoint=endpoint, **kwargs)

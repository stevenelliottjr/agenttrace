"""AgentTrace - Observability SDK for AI Agents.

Example usage - Zero Config:
    from agenttrace import configure, auto_instrument

    # Configure and auto-instrument all available LLM libraries
    tracer = configure(service_name="my-ai-agent")
    auto_instrument(tracer)

    # All LLM calls are now automatically traced!
    from openai import OpenAI
    client = OpenAI()
    response = client.chat.completions.create(...)  # Traced!

Example usage - Manual:
    from agenttrace import AgentTrace, Span

    tracer = AgentTrace.configure(
        service_name="my-ai-agent",
        endpoint="http://localhost:8080",
    )

    # Manually trace operations
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
from agenttrace.integrations import auto_instrument, uninstrument_all

__all__ = [
    "__version__",
    # Main client
    "AgentTrace",
    # Auto-instrumentation
    "auto_instrument",
    "uninstrument_all",
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
    auto_instrument_libs: bool = False,
    **kwargs,
) -> AgentTrace:
    """Configure and return the global AgentTrace instance.

    This is a convenience function equivalent to AgentTrace.configure().

    Args:
        service_name: Name of the service for all spans.
        endpoint: Collector endpoint URL.
        auto_instrument_libs: If True, automatically instrument all
            available LLM libraries (OpenAI, Anthropic, LangChain, LiteLLM).
        **kwargs: Additional arguments passed to AgentTrace.

    Returns:
        The configured AgentTrace instance.

    Example:
        # Minimal zero-config setup
        from agenttrace import configure
        tracer = configure("my-agent", auto_instrument_libs=True)

        # Now all LLM calls are traced automatically
        from anthropic import Anthropic
        client = Anthropic()
        response = client.messages.create(...)  # Traced!
    """
    tracer = AgentTrace.configure(service_name=service_name, endpoint=endpoint, **kwargs)

    if auto_instrument_libs:
        auto_instrument(tracer)

    return tracer

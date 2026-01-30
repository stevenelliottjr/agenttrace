"""Auto-instrumentation for Anthropic Claude API."""

from __future__ import annotations

import functools
from typing import TYPE_CHECKING, Any

import structlog

from agenttrace.client import AgentTrace
from agenttrace.context import SpanContext
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

if TYPE_CHECKING:
    from anthropic import Anthropic, AsyncAnthropic

logger = structlog.get_logger(__name__)

_original_create: Any = None
_original_create_async: Any = None
_instrumented = False


def _extract_usage(response: Any) -> tuple[int | None, int | None]:
    """Extract token usage from Anthropic response."""
    usage = getattr(response, "usage", None)
    if usage:
        return (
            getattr(usage, "input_tokens", None),
            getattr(usage, "output_tokens", None),
        )
    return None, None


def _extract_content_preview(content: Any, max_length: int = 500) -> str | None:
    """Extract text content preview from Anthropic content blocks."""
    if isinstance(content, str):
        return content[:max_length] if len(content) > max_length else content

    if isinstance(content, list):
        texts = []
        for block in content:
            if hasattr(block, "text"):
                texts.append(block.text)
            elif isinstance(block, dict) and "text" in block:
                texts.append(block["text"])
        combined = "\n".join(texts)
        return combined[:max_length] if len(combined) > max_length else combined

    return None


def _build_prompt_preview(messages: list[dict[str, Any]], max_length: int = 500) -> str:
    """Build a preview of the prompt from messages."""
    previews = []
    for msg in messages[-3:]:  # Last 3 messages
        role = msg.get("role", "unknown")
        content = msg.get("content", "")
        if isinstance(content, list):
            content = _extract_content_preview(content, 100) or ""
        elif isinstance(content, str):
            content = content[:100] + "..." if len(content) > 100 else content
        previews.append(f"[{role}] {content}")
    result = "\n".join(previews)
    return result[:max_length] if len(result) > max_length else result


def _wrap_create(tracer: AgentTrace) -> Any:
    """Wrap the synchronous messages.create method."""

    @functools.wraps(_original_create)
    def wrapped_create(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])

        span = tracer.start_span(
            f"anthropic.messages.create",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "anthropic",
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
            },
        )
        span.model_name = model
        span.model_provider = "anthropic"
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = _original_create(self, *args, **kwargs)

                # Extract usage
                tokens_in, tokens_out = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out

                # Extract completion preview
                content = getattr(response, "content", None)
                if content:
                    span.completion_preview = _extract_content_preview(content)

                # Check for tool use
                if content:
                    for block in content:
                        if getattr(block, "type", None) == "tool_use":
                            span.tool_name = getattr(block, "name", None)
                            span.tool_input = getattr(block, "input", None)
                            break

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                tracer.export(span)

    return wrapped_create


def _wrap_create_async(tracer: AgentTrace) -> Any:
    """Wrap the async messages.create method."""

    @functools.wraps(_original_create_async)
    async def wrapped_create_async(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])

        span = tracer.start_span(
            f"anthropic.messages.create",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "anthropic",
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
            },
        )
        span.model_name = model
        span.model_provider = "anthropic"
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = await _original_create_async(self, *args, **kwargs)

                # Extract usage
                tokens_in, tokens_out = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out

                # Extract completion preview
                content = getattr(response, "content", None)
                if content:
                    span.completion_preview = _extract_content_preview(content)

                # Check for tool use
                if content:
                    for block in content:
                        if getattr(block, "type", None) == "tool_use":
                            span.tool_name = getattr(block, "name", None)
                            span.tool_input = getattr(block, "input", None)
                            break

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                tracer.export(span)

    return wrapped_create_async


def instrument_anthropic(tracer: AgentTrace | None = None) -> None:
    """Instrument the Anthropic SDK for automatic tracing.

    This patches the Anthropic client to automatically create spans
    for all API calls.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import instrument_anthropic
        from anthropic import Anthropic

        tracer = AgentTrace.configure(service_name="my-agent")
        instrument_anthropic(tracer)

        # All Anthropic calls are now traced
        client = Anthropic()
        response = client.messages.create(...)
    """
    global _original_create, _original_create_async, _instrumented

    if _instrumented:
        logger.warning("anthropic_already_instrumented")
        return

    if tracer is None:
        tracer = AgentTrace.get_instance()
        if tracer is None:
            raise RuntimeError(
                "No AgentTrace instance available. "
                "Call AgentTrace.configure() first or pass a tracer instance."
            )

    try:
        from anthropic import Anthropic, AsyncAnthropic
        from anthropic.resources import Messages, AsyncMessages
    except ImportError:
        raise ImportError(
            "anthropic package not installed. "
            "Install with: pip install agenttrace[anthropic]"
        )

    # Save original methods
    _original_create = Messages.create
    _original_create_async = AsyncMessages.create

    # Patch methods
    Messages.create = _wrap_create(tracer)
    AsyncMessages.create = _wrap_create_async(tracer)

    _instrumented = True
    logger.info("anthropic_instrumented")


def uninstrument_anthropic() -> None:
    """Remove Anthropic instrumentation."""
    global _original_create, _original_create_async, _instrumented

    if not _instrumented:
        return

    try:
        from anthropic.resources import Messages, AsyncMessages

        if _original_create:
            Messages.create = _original_create
        if _original_create_async:
            AsyncMessages.create = _original_create_async

        _instrumented = False
        logger.info("anthropic_uninstrumented")
    except ImportError:
        pass

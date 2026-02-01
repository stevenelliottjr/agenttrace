"""Auto-instrumentation for Anthropic Claude API."""

from __future__ import annotations

import functools
from typing import TYPE_CHECKING, Any, AsyncIterator, Iterator

import structlog

from agenttrace.client import AgentTrace
from agenttrace.context import SpanContext
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

if TYPE_CHECKING:
    from anthropic import Anthropic, AsyncAnthropic

logger = structlog.get_logger(__name__)

_original_create: Any = None
_original_create_async: Any = None
_original_stream: Any = None
_original_stream_async: Any = None
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


class StreamingSpanWrapper:
    """Wrapper for Anthropic streaming responses that captures content for tracing."""

    def __init__(self, stream: Any, span: Span, tracer: AgentTrace):
        self._stream = stream
        self._span = span
        self._tracer = tracer
        self._collected_content: list[str] = []
        self._tool_use: dict[str, Any] | None = None
        self._input_tokens: int | None = None
        self._output_tokens: int | None = None

    def __iter__(self) -> Iterator[Any]:
        return self

    def __next__(self) -> Any:
        try:
            event = next(self._stream)
            self._process_event(event)
            return event
        except StopIteration:
            self._finalize_span(success=True)
            raise
        except Exception as e:
            self._finalize_span(success=False, error=e)
            raise

    def __enter__(self) -> "StreamingSpanWrapper":
        if hasattr(self._stream, "__enter__"):
            self._stream.__enter__()
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        if exc_type is not None:
            self._finalize_span(success=False, error=exc_val)
        else:
            self._finalize_span(success=True)
        if hasattr(self._stream, "__exit__"):
            self._stream.__exit__(exc_type, exc_val, exc_tb)

    def _process_event(self, event: Any) -> None:
        """Process a streaming event to extract content."""
        event_type = getattr(event, "type", None)

        if event_type == "message_start":
            message = getattr(event, "message", None)
            if message:
                usage = getattr(message, "usage", None)
                if usage:
                    self._input_tokens = getattr(usage, "input_tokens", None)

        elif event_type == "content_block_delta":
            delta = getattr(event, "delta", None)
            if delta:
                delta_type = getattr(delta, "type", None)
                if delta_type == "text_delta":
                    text = getattr(delta, "text", "")
                    if text:
                        self._collected_content.append(text)
                elif delta_type == "input_json_delta":
                    # Tool use input accumulation
                    if self._tool_use is None:
                        self._tool_use = {"name": None, "input": ""}
                    partial = getattr(delta, "partial_json", "")
                    if partial:
                        self._tool_use["input"] += partial

        elif event_type == "content_block_start":
            block = getattr(event, "content_block", None)
            if block:
                block_type = getattr(block, "type", None)
                if block_type == "tool_use":
                    self._tool_use = {
                        "name": getattr(block, "name", None),
                        "input": "",
                    }

        elif event_type == "message_delta":
            delta = getattr(event, "delta", None)
            usage = getattr(event, "usage", None)
            if usage:
                self._output_tokens = getattr(usage, "output_tokens", None)

    def _finalize_span(self, success: bool, error: Exception | None = None) -> None:
        """Finalize the span with collected data."""
        # Set completion preview
        full_content = "".join(self._collected_content)
        if full_content:
            self._span.completion_preview = (
                full_content[:500] if len(full_content) > 500 else full_content
            )

        # Set token usage
        self._span.tokens_in = self._input_tokens
        self._span.tokens_out = self._output_tokens

        # Set tool use
        if self._tool_use:
            self._span.tool_name = self._tool_use.get("name")
            self._span.tool_input = self._tool_use.get("input")

        if success:
            self._span.end(SpanStatus.OK)
        else:
            self._span.set_error(error)
            self._span.end(SpanStatus.ERROR)

        self._tracer.export(self._span)


class AsyncStreamingSpanWrapper:
    """Async wrapper for Anthropic streaming responses that captures content for tracing."""

    def __init__(self, stream: Any, span: Span, tracer: AgentTrace):
        self._stream = stream
        self._span = span
        self._tracer = tracer
        self._collected_content: list[str] = []
        self._tool_use: dict[str, Any] | None = None
        self._input_tokens: int | None = None
        self._output_tokens: int | None = None

    def __aiter__(self) -> "AsyncStreamingSpanWrapper":
        return self

    async def __anext__(self) -> Any:
        try:
            event = await self._stream.__anext__()
            self._process_event(event)
            return event
        except StopAsyncIteration:
            self._finalize_span(success=True)
            raise
        except Exception as e:
            self._finalize_span(success=False, error=e)
            raise

    async def __aenter__(self) -> "AsyncStreamingSpanWrapper":
        if hasattr(self._stream, "__aenter__"):
            await self._stream.__aenter__()
        return self

    async def __aexit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        if exc_type is not None:
            self._finalize_span(success=False, error=exc_val)
        else:
            self._finalize_span(success=True)
        if hasattr(self._stream, "__aexit__"):
            await self._stream.__aexit__(exc_type, exc_val, exc_tb)

    def _process_event(self, event: Any) -> None:
        """Process a streaming event to extract content."""
        event_type = getattr(event, "type", None)

        if event_type == "message_start":
            message = getattr(event, "message", None)
            if message:
                usage = getattr(message, "usage", None)
                if usage:
                    self._input_tokens = getattr(usage, "input_tokens", None)

        elif event_type == "content_block_delta":
            delta = getattr(event, "delta", None)
            if delta:
                delta_type = getattr(delta, "type", None)
                if delta_type == "text_delta":
                    text = getattr(delta, "text", "")
                    if text:
                        self._collected_content.append(text)
                elif delta_type == "input_json_delta":
                    if self._tool_use is None:
                        self._tool_use = {"name": None, "input": ""}
                    partial = getattr(delta, "partial_json", "")
                    if partial:
                        self._tool_use["input"] += partial

        elif event_type == "content_block_start":
            block = getattr(event, "content_block", None)
            if block:
                block_type = getattr(block, "type", None)
                if block_type == "tool_use":
                    self._tool_use = {
                        "name": getattr(block, "name", None),
                        "input": "",
                    }

        elif event_type == "message_delta":
            usage = getattr(event, "usage", None)
            if usage:
                self._output_tokens = getattr(usage, "output_tokens", None)

    def _finalize_span(self, success: bool, error: Exception | None = None) -> None:
        """Finalize the span with collected data."""
        full_content = "".join(self._collected_content)
        if full_content:
            self._span.completion_preview = (
                full_content[:500] if len(full_content) > 500 else full_content
            )

        self._span.tokens_in = self._input_tokens
        self._span.tokens_out = self._output_tokens

        if self._tool_use:
            self._span.tool_name = self._tool_use.get("name")
            self._span.tool_input = self._tool_use.get("input")

        if success:
            self._span.end(SpanStatus.OK)
        else:
            self._span.set_error(error)
            self._span.end(SpanStatus.ERROR)

        self._tracer.export(self._span)


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


def _wrap_stream(tracer: AgentTrace) -> Any:
    """Wrap the synchronous messages.stream method."""

    @functools.wraps(_original_stream)
    def wrapped_stream(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])

        span = tracer.start_span(
            "anthropic.messages.stream",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "anthropic",
                "llm.request.type": "chat",
                "llm.request.stream": True,
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
            },
        )
        span.model_name = model
        span.model_provider = "anthropic"
        span.prompt_preview = _build_prompt_preview(messages)

        try:
            stream = _original_stream(self, *args, **kwargs)
            return StreamingSpanWrapper(stream, span, tracer)
        except Exception as e:
            span.set_error(e)
            span.end(SpanStatus.ERROR)
            tracer.export(span)
            raise

    return wrapped_stream


def _wrap_stream_async(tracer: AgentTrace) -> Any:
    """Wrap the async messages.stream method."""

    @functools.wraps(_original_stream_async)
    async def wrapped_stream_async(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])

        span = tracer.start_span(
            "anthropic.messages.stream",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "anthropic",
                "llm.request.type": "chat",
                "llm.request.stream": True,
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
            },
        )
        span.model_name = model
        span.model_provider = "anthropic"
        span.prompt_preview = _build_prompt_preview(messages)

        try:
            stream = await _original_stream_async(self, *args, **kwargs)
            return AsyncStreamingSpanWrapper(stream, span, tracer)
        except Exception as e:
            span.set_error(e)
            span.end(SpanStatus.ERROR)
            tracer.export(span)
            raise

    return wrapped_stream_async


def instrument_anthropic(tracer: AgentTrace | None = None) -> None:
    """Instrument the Anthropic SDK for automatic tracing.

    This patches the Anthropic client to automatically create spans
    for all API calls, including streaming.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import instrument_anthropic
        from anthropic import Anthropic

        tracer = AgentTrace.configure(service_name="my-agent")
        instrument_anthropic(tracer)

        # All Anthropic calls are now traced (including streaming)
        client = Anthropic()
        response = client.messages.create(...)

        # Streaming is also traced
        with client.messages.stream(...) as stream:
            for event in stream:
                ...
    """
    global _original_create, _original_create_async
    global _original_stream, _original_stream_async
    global _instrumented

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

    # Save stream methods if they exist
    if hasattr(Messages, "stream"):
        _original_stream = Messages.stream
        Messages.stream = _wrap_stream(tracer)

    if hasattr(AsyncMessages, "stream"):
        _original_stream_async = AsyncMessages.stream
        AsyncMessages.stream = _wrap_stream_async(tracer)

    # Patch create methods
    Messages.create = _wrap_create(tracer)
    AsyncMessages.create = _wrap_create_async(tracer)

    _instrumented = True
    logger.info("anthropic_instrumented")


def uninstrument_anthropic() -> None:
    """Remove Anthropic instrumentation."""
    global _original_create, _original_create_async
    global _original_stream, _original_stream_async
    global _instrumented

    if not _instrumented:
        return

    try:
        from anthropic.resources import Messages, AsyncMessages

        if _original_create:
            Messages.create = _original_create
        if _original_create_async:
            AsyncMessages.create = _original_create_async
        if _original_stream:
            Messages.stream = _original_stream
        if _original_stream_async:
            AsyncMessages.stream = _original_stream_async

        _instrumented = False
        logger.info("anthropic_uninstrumented")
    except ImportError:
        pass

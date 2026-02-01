"""Auto-instrumentation for OpenAI API."""

from __future__ import annotations

import functools
from typing import TYPE_CHECKING, Any, AsyncIterator, Iterator

import structlog

from agenttrace.client import AgentTrace
from agenttrace.context import SpanContext
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

if TYPE_CHECKING:
    from openai import OpenAI, AsyncOpenAI

logger = structlog.get_logger(__name__)

_original_create: Any = None
_original_create_async: Any = None
_instrumented = False
_tracer_ref: AgentTrace | None = None


def _extract_usage(response: Any) -> tuple[int | None, int | None, int | None]:
    """Extract token usage from OpenAI response."""
    usage = getattr(response, "usage", None)
    if usage:
        # Handle reasoning tokens for o1/o3 models
        completion_details = getattr(usage, "completion_tokens_details", None)
        reasoning_tokens = None
        if completion_details:
            reasoning_tokens = getattr(completion_details, "reasoning_tokens", None)

        return (
            getattr(usage, "prompt_tokens", None),
            getattr(usage, "completion_tokens", None),
            reasoning_tokens,
        )
    return None, None, None


def _extract_completion_preview(response: Any, max_length: int = 500) -> str | None:
    """Extract completion text from OpenAI response."""
    choices = getattr(response, "choices", None)
    if choices and len(choices) > 0:
        message = getattr(choices[0], "message", None)
        if message:
            content = getattr(message, "content", None)
            if content:
                return content[:max_length] if len(content) > max_length else content
    return None


def _extract_tool_calls(response: Any) -> list[dict[str, Any]] | None:
    """Extract tool calls from OpenAI response."""
    choices = getattr(response, "choices", None)
    if choices and len(choices) > 0:
        message = getattr(choices[0], "message", None)
        if message:
            tool_calls = getattr(message, "tool_calls", None)
            if tool_calls:
                return [
                    {
                        "id": getattr(tc, "id", None),
                        "type": getattr(tc, "type", None),
                        "function": {
                            "name": getattr(getattr(tc, "function", None), "name", None),
                            "arguments": getattr(getattr(tc, "function", None), "arguments", None),
                        },
                    }
                    for tc in tool_calls
                ]
    return None


def _build_prompt_preview(messages: list[dict[str, Any]], max_length: int = 500) -> str:
    """Build a preview of the prompt from messages."""
    previews = []
    for msg in messages[-3:]:  # Last 3 messages
        role = msg.get("role", "unknown")
        content = msg.get("content", "")
        if isinstance(content, list):
            # Handle content parts (text + images)
            texts = [p.get("text", "") for p in content if p.get("type") == "text"]
            content = " ".join(texts)
        if isinstance(content, str):
            content = content[:100] + "..." if len(content) > 100 else content
        previews.append(f"[{role}] {content}")
    result = "\n".join(previews)
    return result[:max_length] if len(result) > max_length else result


class StreamingSpanWrapper:
    """Wrapper for OpenAI streaming responses that captures content for tracing."""

    def __init__(self, stream: Iterator[Any], span: Span, tracer: AgentTrace):
        self._stream = stream
        self._span = span
        self._tracer = tracer
        self._collected_content: list[str] = []
        self._collected_tool_calls: list[dict[str, Any]] = []
        self._usage: dict[str, int] = {}

    def __iter__(self) -> Iterator[Any]:
        return self

    def __next__(self) -> Any:
        try:
            chunk = next(self._stream)
            self._process_chunk(chunk)
            return chunk
        except StopIteration:
            self._finalize_span(success=True)
            raise
        except Exception as e:
            self._finalize_span(success=False, error=e)
            raise

    def _process_chunk(self, chunk: Any) -> None:
        """Process a streaming chunk to extract content."""
        choices = getattr(chunk, "choices", None)
        if choices and len(choices) > 0:
            delta = getattr(choices[0], "delta", None)
            if delta:
                # Collect text content
                content = getattr(delta, "content", None)
                if content:
                    self._collected_content.append(content)

                # Collect tool calls
                tool_calls = getattr(delta, "tool_calls", None)
                if tool_calls:
                    for tc in tool_calls:
                        idx = getattr(tc, "index", 0)
                        while len(self._collected_tool_calls) <= idx:
                            self._collected_tool_calls.append(
                                {"id": None, "type": None, "function": {"name": "", "arguments": ""}}
                            )
                        if getattr(tc, "id", None):
                            self._collected_tool_calls[idx]["id"] = tc.id
                        if getattr(tc, "type", None):
                            self._collected_tool_calls[idx]["type"] = tc.type
                        func = getattr(tc, "function", None)
                        if func:
                            if getattr(func, "name", None):
                                self._collected_tool_calls[idx]["function"]["name"] = func.name
                            if getattr(func, "arguments", None):
                                self._collected_tool_calls[idx]["function"]["arguments"] += func.arguments

        # Check for usage in final chunk (OpenAI includes this with stream_options)
        usage = getattr(chunk, "usage", None)
        if usage:
            self._usage["prompt_tokens"] = getattr(usage, "prompt_tokens", 0)
            self._usage["completion_tokens"] = getattr(usage, "completion_tokens", 0)

    def _finalize_span(self, success: bool, error: Exception | None = None) -> None:
        """Finalize the span with collected data."""
        # Set completion preview
        full_content = "".join(self._collected_content)
        if full_content:
            self._span.completion_preview = (
                full_content[:500] if len(full_content) > 500 else full_content
            )

        # Set token usage
        if self._usage:
            self._span.tokens_in = self._usage.get("prompt_tokens")
            self._span.tokens_out = self._usage.get("completion_tokens")

        # Set tool calls
        if self._collected_tool_calls:
            self._span.tool_name = self._collected_tool_calls[0]["function"]["name"]
            self._span.tool_input = self._collected_tool_calls[0]["function"]["arguments"]
            self._span.attributes["llm.tool_calls"] = self._collected_tool_calls

        if success:
            self._span.end(SpanStatus.OK)
        else:
            self._span.set_error(error)
            self._span.end(SpanStatus.ERROR)

        self._tracer.export(self._span)


class AsyncStreamingSpanWrapper:
    """Async wrapper for OpenAI streaming responses that captures content for tracing."""

    def __init__(self, stream: AsyncIterator[Any], span: Span, tracer: AgentTrace):
        self._stream = stream
        self._span = span
        self._tracer = tracer
        self._collected_content: list[str] = []
        self._collected_tool_calls: list[dict[str, Any]] = []
        self._usage: dict[str, int] = {}

    def __aiter__(self) -> "AsyncStreamingSpanWrapper":
        return self

    async def __anext__(self) -> Any:
        try:
            chunk = await self._stream.__anext__()
            self._process_chunk(chunk)
            return chunk
        except StopAsyncIteration:
            self._finalize_span(success=True)
            raise
        except Exception as e:
            self._finalize_span(success=False, error=e)
            raise

    def _process_chunk(self, chunk: Any) -> None:
        """Process a streaming chunk to extract content."""
        choices = getattr(chunk, "choices", None)
        if choices and len(choices) > 0:
            delta = getattr(choices[0], "delta", None)
            if delta:
                content = getattr(delta, "content", None)
                if content:
                    self._collected_content.append(content)

                tool_calls = getattr(delta, "tool_calls", None)
                if tool_calls:
                    for tc in tool_calls:
                        idx = getattr(tc, "index", 0)
                        while len(self._collected_tool_calls) <= idx:
                            self._collected_tool_calls.append(
                                {"id": None, "type": None, "function": {"name": "", "arguments": ""}}
                            )
                        if getattr(tc, "id", None):
                            self._collected_tool_calls[idx]["id"] = tc.id
                        if getattr(tc, "type", None):
                            self._collected_tool_calls[idx]["type"] = tc.type
                        func = getattr(tc, "function", None)
                        if func:
                            if getattr(func, "name", None):
                                self._collected_tool_calls[idx]["function"]["name"] = func.name
                            if getattr(func, "arguments", None):
                                self._collected_tool_calls[idx]["function"]["arguments"] += func.arguments

        usage = getattr(chunk, "usage", None)
        if usage:
            self._usage["prompt_tokens"] = getattr(usage, "prompt_tokens", 0)
            self._usage["completion_tokens"] = getattr(usage, "completion_tokens", 0)

    def _finalize_span(self, success: bool, error: Exception | None = None) -> None:
        """Finalize the span with collected data."""
        full_content = "".join(self._collected_content)
        if full_content:
            self._span.completion_preview = (
                full_content[:500] if len(full_content) > 500 else full_content
            )

        if self._usage:
            self._span.tokens_in = self._usage.get("prompt_tokens")
            self._span.tokens_out = self._usage.get("completion_tokens")

        if self._collected_tool_calls:
            self._span.tool_name = self._collected_tool_calls[0]["function"]["name"]
            self._span.tool_input = self._collected_tool_calls[0]["function"]["arguments"]
            self._span.attributes["llm.tool_calls"] = self._collected_tool_calls

        if success:
            self._span.end(SpanStatus.OK)
        else:
            self._span.set_error(error)
            self._span.end(SpanStatus.ERROR)

        self._tracer.export(self._span)


def _wrap_create(tracer: AgentTrace) -> Any:
    """Wrap the synchronous chat.completions.create method."""

    @functools.wraps(_original_create)
    def wrapped_create(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])
        is_streaming = kwargs.get("stream", False)

        span = tracer.start_span(
            "openai.chat.completions.create",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "openai",
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
                "llm.request.top_p": kwargs.get("top_p"),
                "llm.request.stream": is_streaming,
            },
        )
        span.model_name = model
        span.model_provider = "openai"
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = _original_create(self, *args, **kwargs)

                # Handle streaming response
                if is_streaming:
                    return StreamingSpanWrapper(response, span, tracer)

                # Non-streaming: extract usage
                tokens_in, tokens_out, tokens_reasoning = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out
                span.tokens_reasoning = tokens_reasoning

                # Extract completion preview
                span.completion_preview = _extract_completion_preview(response)

                # Extract tool calls
                tool_calls = _extract_tool_calls(response)
                if tool_calls and len(tool_calls) > 0:
                    span.tool_name = tool_calls[0]["function"]["name"]
                    span.tool_input = tool_calls[0]["function"]["arguments"]
                    span.attributes["llm.tool_calls"] = tool_calls

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                # Only export for non-streaming; streaming wrapper handles export
                if not is_streaming:
                    tracer.export(span)

    return wrapped_create


def _wrap_create_async(tracer: AgentTrace) -> Any:
    """Wrap the async chat.completions.create method."""

    @functools.wraps(_original_create_async)
    async def wrapped_create_async(self: Any, *args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", "unknown")
        messages = kwargs.get("messages", [])
        is_streaming = kwargs.get("stream", False)

        span = tracer.start_span(
            "openai.chat.completions.create",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": "openai",
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
                "llm.request.top_p": kwargs.get("top_p"),
                "llm.request.stream": is_streaming,
            },
        )
        span.model_name = model
        span.model_provider = "openai"
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = await _original_create_async(self, *args, **kwargs)

                # Handle streaming response
                if is_streaming:
                    return AsyncStreamingSpanWrapper(response, span, tracer)

                # Non-streaming: extract usage
                tokens_in, tokens_out, tokens_reasoning = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out
                span.tokens_reasoning = tokens_reasoning

                # Extract completion preview
                span.completion_preview = _extract_completion_preview(response)

                # Extract tool calls
                tool_calls = _extract_tool_calls(response)
                if tool_calls and len(tool_calls) > 0:
                    span.tool_name = tool_calls[0]["function"]["name"]
                    span.tool_input = tool_calls[0]["function"]["arguments"]
                    span.attributes["llm.tool_calls"] = tool_calls

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                # Only export for non-streaming; streaming wrapper handles export
                if not is_streaming:
                    tracer.export(span)

    return wrapped_create_async


def instrument_openai(tracer: AgentTrace | None = None) -> None:
    """Instrument the OpenAI SDK for automatic tracing.

    This patches the OpenAI client to automatically create spans
    for all chat completion API calls.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import instrument_openai
        from openai import OpenAI

        tracer = AgentTrace.configure(service_name="my-agent")
        instrument_openai(tracer)

        # All OpenAI calls are now traced
        client = OpenAI()
        response = client.chat.completions.create(...)
    """
    global _original_create, _original_create_async, _instrumented

    if _instrumented:
        logger.warning("openai_already_instrumented")
        return

    if tracer is None:
        tracer = AgentTrace.get_instance()
        if tracer is None:
            raise RuntimeError(
                "No AgentTrace instance available. "
                "Call AgentTrace.configure() first or pass a tracer instance."
            )

    try:
        from openai import OpenAI, AsyncOpenAI
        from openai.resources.chat import Completions, AsyncCompletions
    except ImportError:
        raise ImportError(
            "openai package not installed. "
            "Install with: pip install agenttrace[openai]"
        )

    # Save original methods
    _original_create = Completions.create
    _original_create_async = AsyncCompletions.create

    # Patch methods
    Completions.create = _wrap_create(tracer)
    AsyncCompletions.create = _wrap_create_async(tracer)

    _instrumented = True
    logger.info("openai_instrumented")


def uninstrument_openai() -> None:
    """Remove OpenAI instrumentation."""
    global _original_create, _original_create_async, _instrumented

    if not _instrumented:
        return

    try:
        from openai.resources.chat import Completions, AsyncCompletions

        if _original_create:
            Completions.create = _original_create
        if _original_create_async:
            AsyncCompletions.create = _original_create_async

        _instrumented = False
        logger.info("openai_uninstrumented")
    except ImportError:
        pass

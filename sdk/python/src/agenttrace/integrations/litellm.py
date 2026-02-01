"""Auto-instrumentation for LiteLLM unified LLM interface."""

from __future__ import annotations

import functools
from typing import TYPE_CHECKING, Any, AsyncIterator, Iterator

import structlog

from agenttrace.client import AgentTrace
from agenttrace.context import SpanContext
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

if TYPE_CHECKING:
    pass

logger = structlog.get_logger(__name__)

_original_completion: Any = None
_original_acompletion: Any = None
_original_embedding: Any = None
_original_aembedding: Any = None
_instrumented = False


def _extract_provider(model: str) -> str:
    """Extract provider from LiteLLM model string.

    LiteLLM uses format: provider/model or just model for OpenAI.
    Examples: "anthropic/claude-3-opus", "gpt-4", "azure/gpt-4"
    """
    if "/" in model:
        return model.split("/")[0]

    # OpenAI models without prefix
    if model.startswith(("gpt-", "o1", "o3", "text-", "davinci")):
        return "openai"

    # Claude models without prefix
    if model.startswith("claude"):
        return "anthropic"

    return "unknown"


def _extract_usage(response: Any) -> tuple[int | None, int | None]:
    """Extract token usage from LiteLLM response."""
    usage = getattr(response, "usage", None)
    if usage:
        return (
            getattr(usage, "prompt_tokens", None),
            getattr(usage, "completion_tokens", None),
        )
    return None, None


def _extract_completion_preview(response: Any, max_length: int = 500) -> str | None:
    """Extract completion text from LiteLLM response."""
    choices = getattr(response, "choices", None)
    if choices and len(choices) > 0:
        message = getattr(choices[0], "message", None)
        if message:
            content = getattr(message, "content", None)
            if content:
                return content[:max_length] if len(content) > max_length else content
    return None


def _extract_tool_calls(response: Any) -> list[dict[str, Any]] | None:
    """Extract tool calls from LiteLLM response."""
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
                            "arguments": getattr(
                                getattr(tc, "function", None), "arguments", None
                            ),
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


def _wrap_completion(tracer: AgentTrace) -> Any:
    """Wrap the synchronous litellm.completion function."""

    @functools.wraps(_original_completion)
    def wrapped_completion(*args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", args[0] if args else "unknown")
        messages = kwargs.get("messages", [])
        provider = _extract_provider(model)

        span = tracer.start_span(
            "litellm.completion",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
                "llm.request.top_p": kwargs.get("top_p"),
                "litellm.api_base": kwargs.get("api_base"),
            },
        )
        span.model_name = model
        span.model_provider = provider
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = _original_completion(*args, **kwargs)

                # Handle streaming responses
                if kwargs.get("stream", False):
                    return _wrap_stream_response(span, tracer, response)

                # Extract usage
                tokens_in, tokens_out = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out

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
                if not kwargs.get("stream", False):
                    tracer.export(span)

    return wrapped_completion


def _wrap_acompletion(tracer: AgentTrace) -> Any:
    """Wrap the async litellm.acompletion function."""

    @functools.wraps(_original_acompletion)
    async def wrapped_acompletion(*args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", args[0] if args else "unknown")
        messages = kwargs.get("messages", [])
        provider = _extract_provider(model)

        span = tracer.start_span(
            "litellm.acompletion",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "chat",
                "llm.request.max_tokens": kwargs.get("max_tokens"),
                "llm.request.temperature": kwargs.get("temperature"),
                "llm.request.top_p": kwargs.get("top_p"),
                "litellm.api_base": kwargs.get("api_base"),
            },
        )
        span.model_name = model
        span.model_provider = provider
        span.prompt_preview = _build_prompt_preview(messages)

        with SpanContext(span):
            try:
                response = await _original_acompletion(*args, **kwargs)

                # Handle streaming responses
                if kwargs.get("stream", False):
                    return _wrap_async_stream_response(span, tracer, response)

                # Extract usage
                tokens_in, tokens_out = _extract_usage(response)
                span.tokens_in = tokens_in
                span.tokens_out = tokens_out

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
                if not kwargs.get("stream", False):
                    tracer.export(span)

    return wrapped_acompletion


def _wrap_stream_response(
    span: Span, tracer: AgentTrace, response: Iterator[Any]
) -> Iterator[Any]:
    """Wrap a streaming response to capture the full output."""
    collected_content = []
    tokens_in = None
    tokens_out = None

    try:
        for chunk in response:
            # Collect content
            choices = getattr(chunk, "choices", None)
            if choices and len(choices) > 0:
                delta = getattr(choices[0], "delta", None)
                if delta:
                    content = getattr(delta, "content", None)
                    if content:
                        collected_content.append(content)

            # Check for usage in final chunk
            usage = getattr(chunk, "usage", None)
            if usage:
                tokens_in = getattr(usage, "prompt_tokens", None)
                tokens_out = getattr(usage, "completion_tokens", None)

            yield chunk

        # Finalize span
        full_content = "".join(collected_content)
        span.completion_preview = (
            full_content[:500] if len(full_content) > 500 else full_content
        )
        span.tokens_in = tokens_in
        span.tokens_out = tokens_out
        span.end(SpanStatus.OK)

    except Exception as e:
        span.set_error(e)
        span.end(SpanStatus.ERROR)
        raise

    finally:
        tracer.export(span)


async def _wrap_async_stream_response(
    span: Span, tracer: AgentTrace, response: AsyncIterator[Any]
) -> AsyncIterator[Any]:
    """Wrap an async streaming response to capture the full output."""
    collected_content = []
    tokens_in = None
    tokens_out = None

    try:
        async for chunk in response:
            # Collect content
            choices = getattr(chunk, "choices", None)
            if choices and len(choices) > 0:
                delta = getattr(choices[0], "delta", None)
                if delta:
                    content = getattr(delta, "content", None)
                    if content:
                        collected_content.append(content)

            # Check for usage in final chunk
            usage = getattr(chunk, "usage", None)
            if usage:
                tokens_in = getattr(usage, "prompt_tokens", None)
                tokens_out = getattr(usage, "completion_tokens", None)

            yield chunk

        # Finalize span
        full_content = "".join(collected_content)
        span.completion_preview = (
            full_content[:500] if len(full_content) > 500 else full_content
        )
        span.tokens_in = tokens_in
        span.tokens_out = tokens_out
        span.end(SpanStatus.OK)

    except Exception as e:
        span.set_error(e)
        span.end(SpanStatus.ERROR)
        raise

    finally:
        tracer.export(span)


def _wrap_embedding(tracer: AgentTrace) -> Any:
    """Wrap the synchronous litellm.embedding function."""

    @functools.wraps(_original_embedding)
    def wrapped_embedding(*args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", args[0] if args else "unknown")
        input_text = kwargs.get("input", "")
        provider = _extract_provider(model)

        # Calculate input preview
        if isinstance(input_text, list):
            preview = str(input_text[:3])[:500]
        else:
            preview = str(input_text)[:500]

        span = tracer.start_span(
            "litellm.embedding",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "embedding",
                "embedding.input_preview": preview,
            },
        )
        span.model_name = model
        span.model_provider = provider

        with SpanContext(span):
            try:
                response = _original_embedding(*args, **kwargs)

                # Extract usage
                usage = getattr(response, "usage", None)
                if usage:
                    span.tokens_in = getattr(usage, "prompt_tokens", None)
                    span.tokens_out = getattr(usage, "total_tokens", None)

                # Add embedding dimensions
                data = getattr(response, "data", None)
                if data and len(data) > 0:
                    embedding = getattr(data[0], "embedding", None)
                    if embedding:
                        span.attributes["embedding.dimensions"] = len(embedding)

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                tracer.export(span)

    return wrapped_embedding


def _wrap_aembedding(tracer: AgentTrace) -> Any:
    """Wrap the async litellm.aembedding function."""

    @functools.wraps(_original_aembedding)
    async def wrapped_aembedding(*args: Any, **kwargs: Any) -> Any:
        model = kwargs.get("model", args[0] if args else "unknown")
        input_text = kwargs.get("input", "")
        provider = _extract_provider(model)

        # Calculate input preview
        if isinstance(input_text, list):
            preview = str(input_text[:3])[:500]
        else:
            preview = str(input_text)[:500]

        span = tracer.start_span(
            "litellm.aembedding",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "embedding",
                "embedding.input_preview": preview,
            },
        )
        span.model_name = model
        span.model_provider = provider

        with SpanContext(span):
            try:
                response = await _original_aembedding(*args, **kwargs)

                # Extract usage
                usage = getattr(response, "usage", None)
                if usage:
                    span.tokens_in = getattr(usage, "prompt_tokens", None)
                    span.tokens_out = getattr(usage, "total_tokens", None)

                # Add embedding dimensions
                data = getattr(response, "data", None)
                if data and len(data) > 0:
                    embedding = getattr(data[0], "embedding", None)
                    if embedding:
                        span.attributes["embedding.dimensions"] = len(embedding)

                span.end(SpanStatus.OK)
                return response

            except Exception as e:
                span.set_error(e)
                span.end(SpanStatus.ERROR)
                raise
            finally:
                tracer.export(span)

    return wrapped_aembedding


def instrument_litellm(tracer: AgentTrace | None = None) -> None:
    """Instrument LiteLLM for automatic tracing.

    This patches LiteLLM's completion and embedding functions to
    automatically create spans for all API calls.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import instrument_litellm
        import litellm

        tracer = AgentTrace.configure(service_name="my-agent")
        instrument_litellm(tracer)

        # All LiteLLM calls are now traced
        response = litellm.completion(
            model="gpt-4o-mini",
            messages=[{"role": "user", "content": "Hello"}]
        )
    """
    global _original_completion, _original_acompletion
    global _original_embedding, _original_aembedding
    global _instrumented

    if _instrumented:
        logger.warning("litellm_already_instrumented")
        return

    if tracer is None:
        tracer = AgentTrace.get_instance()
        if tracer is None:
            raise RuntimeError(
                "No AgentTrace instance available. "
                "Call AgentTrace.configure() first or pass a tracer instance."
            )

    try:
        import litellm
    except ImportError:
        raise ImportError(
            "litellm package not installed. "
            "Install with: pip install agenttrace[litellm]"
        )

    # Save original functions
    _original_completion = litellm.completion
    _original_acompletion = litellm.acompletion
    _original_embedding = litellm.embedding
    _original_aembedding = litellm.aembedding

    # Patch functions
    litellm.completion = _wrap_completion(tracer)
    litellm.acompletion = _wrap_acompletion(tracer)
    litellm.embedding = _wrap_embedding(tracer)
    litellm.aembedding = _wrap_aembedding(tracer)

    _instrumented = True
    logger.info("litellm_instrumented")


def uninstrument_litellm() -> None:
    """Remove LiteLLM instrumentation."""
    global _original_completion, _original_acompletion
    global _original_embedding, _original_aembedding
    global _instrumented

    if not _instrumented:
        return

    try:
        import litellm

        if _original_completion:
            litellm.completion = _original_completion
        if _original_acompletion:
            litellm.acompletion = _original_acompletion
        if _original_embedding:
            litellm.embedding = _original_embedding
        if _original_aembedding:
            litellm.aembedding = _original_aembedding

        _instrumented = False
        logger.info("litellm_uninstrumented")
    except ImportError:
        pass

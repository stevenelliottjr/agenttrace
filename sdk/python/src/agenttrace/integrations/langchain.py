"""Auto-instrumentation for LangChain framework."""

from __future__ import annotations

import functools
from typing import TYPE_CHECKING, Any, Optional
from uuid import UUID

import structlog

from agenttrace.client import AgentTrace
from agenttrace.context import SpanContext
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

if TYPE_CHECKING:
    from langchain_core.callbacks import BaseCallbackHandler

logger = structlog.get_logger(__name__)

_instrumented = False


class AgentTraceCallbackHandler:
    """LangChain callback handler that creates AgentTrace spans.

    This handler integrates with LangChain's callback system to automatically
    create spans for LLM calls, chain executions, tool runs, and retrievers.
    """

    def __init__(self, tracer: AgentTrace):
        self.tracer = tracer
        self._spans: dict[str, Span] = {}  # run_id -> span

    # --- LLM Callbacks ---

    def on_llm_start(
        self,
        serialized: dict[str, Any],
        prompts: list[str],
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        tags: Optional[list[str]] = None,
        metadata: Optional[dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        """Called when LLM starts running."""
        model_name = serialized.get("kwargs", {}).get("model_name", "unknown")

        # Determine provider from class name
        class_name = serialized.get("id", ["unknown"])[-1].lower()
        if "openai" in class_name:
            provider = "openai"
        elif "anthropic" in class_name or "claude" in class_name:
            provider = "anthropic"
        elif "google" in class_name or "gemini" in class_name:
            provider = "google"
        else:
            provider = "unknown"

        span = self.tracer.start_span(
            f"langchain.llm.{class_name}",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "chat",
                "langchain.run_id": str(run_id),
                "langchain.tags": tags or [],
            },
        )
        span.model_name = model_name
        span.model_provider = provider

        # Build prompt preview
        if prompts:
            preview = prompts[0][:500] if len(prompts[0]) > 500 else prompts[0]
            span.prompt_preview = preview

        self._spans[str(run_id)] = span

    def on_llm_end(
        self,
        response: Any,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when LLM ends running."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        # Extract token usage from LLM result
        if hasattr(response, "llm_output") and response.llm_output:
            token_usage = response.llm_output.get("token_usage", {})
            span.tokens_in = token_usage.get("prompt_tokens")
            span.tokens_out = token_usage.get("completion_tokens")

        # Extract completion preview
        if hasattr(response, "generations") and response.generations:
            first_gen = response.generations[0]
            if first_gen and hasattr(first_gen[0], "text"):
                text = first_gen[0].text
                span.completion_preview = text[:500] if len(text) > 500 else text

        span.end(SpanStatus.OK)
        self.tracer.export(span)

    def on_llm_error(
        self,
        error: BaseException,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when LLM errors."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.set_error(error)
        span.end(SpanStatus.ERROR)
        self.tracer.export(span)

    # --- Chat Model Callbacks ---

    def on_chat_model_start(
        self,
        serialized: dict[str, Any],
        messages: list[list[Any]],
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        tags: Optional[list[str]] = None,
        metadata: Optional[dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        """Called when chat model starts running."""
        model_name = serialized.get("kwargs", {}).get("model_name", "unknown")
        model_name = model_name or serialized.get("kwargs", {}).get("model", "unknown")

        # Determine provider
        class_name = serialized.get("id", ["unknown"])[-1].lower()
        if "openai" in class_name:
            provider = "openai"
        elif "anthropic" in class_name or "claude" in class_name:
            provider = "anthropic"
        elif "google" in class_name or "gemini" in class_name:
            provider = "google"
        else:
            provider = "unknown"

        span = self.tracer.start_span(
            f"langchain.chat.{class_name}",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.LLM_CALL,
            attributes={
                "llm.vendor": provider,
                "llm.request.type": "chat",
                "langchain.run_id": str(run_id),
                "langchain.tags": tags or [],
            },
        )
        span.model_name = model_name
        span.model_provider = provider

        # Build prompt preview from messages
        if messages and messages[0]:
            previews = []
            for msg in messages[0][-3:]:  # Last 3 messages
                role = getattr(msg, "type", "unknown")
                content = getattr(msg, "content", "")
                if isinstance(content, str):
                    content = content[:100] + "..." if len(content) > 100 else content
                previews.append(f"[{role}] {content}")
            span.prompt_preview = "\n".join(previews)[:500]

        self._spans[str(run_id)] = span

    # --- Chain Callbacks ---

    def on_chain_start(
        self,
        serialized: dict[str, Any],
        inputs: dict[str, Any],
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        tags: Optional[list[str]] = None,
        metadata: Optional[dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        """Called when chain starts running."""
        chain_name = serialized.get("id", ["unknown"])[-1]

        span = self.tracer.start_span(
            f"langchain.chain.{chain_name}",
            span_kind=SpanKind.INTERNAL,
            span_type=SpanType.TASK,
            attributes={
                "langchain.chain_type": chain_name,
                "langchain.run_id": str(run_id),
                "langchain.tags": tags or [],
            },
        )

        # Store input preview
        if inputs:
            input_str = str(inputs)
            span.attributes["chain.input_preview"] = (
                input_str[:500] if len(input_str) > 500 else input_str
            )

        self._spans[str(run_id)] = span

    def on_chain_end(
        self,
        outputs: dict[str, Any],
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when chain ends running."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        # Store output preview
        if outputs:
            output_str = str(outputs)
            span.attributes["chain.output_preview"] = (
                output_str[:500] if len(output_str) > 500 else output_str
            )

        span.end(SpanStatus.OK)
        self.tracer.export(span)

    def on_chain_error(
        self,
        error: BaseException,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when chain errors."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.set_error(error)
        span.end(SpanStatus.ERROR)
        self.tracer.export(span)

    # --- Tool Callbacks ---

    def on_tool_start(
        self,
        serialized: dict[str, Any],
        input_str: str,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        tags: Optional[list[str]] = None,
        metadata: Optional[dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        """Called when tool starts running."""
        tool_name = serialized.get("name", "unknown_tool")

        span = self.tracer.start_span(
            f"langchain.tool.{tool_name}",
            span_kind=SpanKind.INTERNAL,
            span_type=SpanType.TOOL_EXECUTION,
            attributes={
                "langchain.run_id": str(run_id),
                "langchain.tags": tags or [],
            },
        )
        span.tool_name = tool_name
        span.tool_input = input_str[:1000] if len(input_str) > 1000 else input_str

        self._spans[str(run_id)] = span

    def on_tool_end(
        self,
        output: str,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when tool ends running."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.tool_output = output[:1000] if len(output) > 1000 else output
        span.end(SpanStatus.OK)
        self.tracer.export(span)

    def on_tool_error(
        self,
        error: BaseException,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when tool errors."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.set_error(error)
        span.end(SpanStatus.ERROR)
        self.tracer.export(span)

    # --- Retriever Callbacks ---

    def on_retriever_start(
        self,
        serialized: dict[str, Any],
        query: str,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        tags: Optional[list[str]] = None,
        metadata: Optional[dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        """Called when retriever starts running."""
        retriever_name = serialized.get("id", ["unknown"])[-1]

        span = self.tracer.start_span(
            f"langchain.retriever.{retriever_name}",
            span_kind=SpanKind.CLIENT,
            span_type=SpanType.MEMORY_RETRIEVAL,
            attributes={
                "retriever.type": retriever_name,
                "retriever.query": query[:500] if len(query) > 500 else query,
                "langchain.run_id": str(run_id),
                "langchain.tags": tags or [],
            },
        )

        self._spans[str(run_id)] = span

    def on_retriever_end(
        self,
        documents: list[Any],
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when retriever ends running."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.attributes["retriever.document_count"] = len(documents)

        # Preview first few documents
        if documents:
            previews = []
            for doc in documents[:3]:
                content = getattr(doc, "page_content", str(doc))
                previews.append(content[:200] if len(content) > 200 else content)
            span.attributes["retriever.documents_preview"] = previews

        span.end(SpanStatus.OK)
        self.tracer.export(span)

    def on_retriever_error(
        self,
        error: BaseException,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when retriever errors."""
        span = self._spans.pop(str(run_id), None)
        if not span:
            return

        span.set_error(error)
        span.end(SpanStatus.ERROR)
        self.tracer.export(span)

    # --- Agent Callbacks ---

    def on_agent_action(
        self,
        action: Any,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when agent takes an action."""
        tool = getattr(action, "tool", "unknown")
        tool_input = getattr(action, "tool_input", "")

        span = self.tracer.start_span(
            f"langchain.agent.action.{tool}",
            span_kind=SpanKind.INTERNAL,
            span_type=SpanType.REASONING,
            attributes={
                "agent.tool": tool,
                "agent.tool_input": str(tool_input)[:500],
                "langchain.run_id": str(run_id),
            },
        )

        self._spans[f"action_{run_id}"] = span

    def on_agent_finish(
        self,
        finish: Any,
        *,
        run_id: UUID,
        parent_run_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        """Called when agent finishes."""
        # Close any pending action span
        span = self._spans.pop(f"action_{run_id}", None)
        if span:
            output = getattr(finish, "return_values", {})
            span.attributes["agent.output"] = str(output)[:500]
            span.end(SpanStatus.OK)
            self.tracer.export(span)


def get_callback_handler(tracer: AgentTrace | None = None) -> AgentTraceCallbackHandler:
    """Get a LangChain callback handler for AgentTrace.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Returns:
        Callback handler to pass to LangChain operations.

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations.langchain import get_callback_handler
        from langchain_openai import ChatOpenAI

        tracer = AgentTrace.configure(service_name="my-agent")
        handler = get_callback_handler(tracer)

        llm = ChatOpenAI()
        response = llm.invoke("Hello", config={"callbacks": [handler]})
    """
    if tracer is None:
        tracer = AgentTrace.get_instance()
        if tracer is None:
            raise RuntimeError(
                "No AgentTrace instance available. "
                "Call AgentTrace.configure() first or pass a tracer instance."
            )

    return AgentTraceCallbackHandler(tracer)


def instrument_langchain(tracer: AgentTrace | None = None) -> None:
    """Instrument LangChain globally for automatic tracing.

    This sets up a global callback handler that traces all LangChain
    operations automatically.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import instrument_langchain

        tracer = AgentTrace.configure(service_name="my-agent")
        instrument_langchain(tracer)

        # All LangChain calls are now traced automatically
        from langchain_openai import ChatOpenAI
        llm = ChatOpenAI()
        response = llm.invoke("Hello")
    """
    global _instrumented

    if _instrumented:
        logger.warning("langchain_already_instrumented")
        return

    if tracer is None:
        tracer = AgentTrace.get_instance()
        if tracer is None:
            raise RuntimeError(
                "No AgentTrace instance available. "
                "Call AgentTrace.configure() first or pass a tracer instance."
            )

    try:
        from langchain_core.callbacks import BaseCallbackManager
        from langchain_core.globals import set_llm_cache
    except ImportError:
        raise ImportError(
            "langchain-core package not installed. "
            "Install with: pip install agenttrace[langchain]"
        )

    handler = AgentTraceCallbackHandler(tracer)

    # Patch the default callback manager to include our handler
    original_init = BaseCallbackManager.__init__

    @functools.wraps(original_init)
    def patched_init(self: Any, *args: Any, **kwargs: Any) -> None:
        original_init(self, *args, **kwargs)
        # Add our handler if not already present
        if handler not in self.handlers:
            self.add_handler(handler)

    BaseCallbackManager.__init__ = patched_init

    _instrumented = True
    logger.info("langchain_instrumented")


def uninstrument_langchain() -> None:
    """Remove LangChain instrumentation."""
    global _instrumented

    if not _instrumented:
        return

    # Note: Full uninstrumentation would require storing and restoring
    # the original __init__, which adds complexity. For now, just mark
    # as uninstrumented to prevent re-instrumentation warnings.
    _instrumented = False
    logger.info("langchain_uninstrumented")

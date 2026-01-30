"""Main AgentTrace client."""

from __future__ import annotations

import asyncio
import atexit
from collections import deque
from threading import Lock, Thread
from typing import Any, Callable
from uuid import uuid4

import structlog

from agenttrace.context import (
    SpanContext,
    get_current_span,
    get_current_trace_id,
    set_current_trace_id,
)
from agenttrace.exporters.base import Exporter
from agenttrace.exporters.http import HttpExporter
from agenttrace.models import Span, SpanKind, SpanStatus, SpanType

logger = structlog.get_logger(__name__)


class AgentTrace:
    """Main client for AgentTrace SDK.

    Example usage:
        tracer = AgentTrace(service_name="my-agent")

        # Using context manager
        with tracer.span("process_request") as span:
            span.set_attribute("user_id", "123")
            result = do_work()

        # Using decorator
        @tracer.trace("my_function")
        def my_function():
            pass

        # Manual span management
        span = tracer.start_span("manual_span")
        try:
            do_work()
            span.end(SpanStatus.OK)
        except Exception as e:
            span.set_error(e)
            span.end(SpanStatus.ERROR)
        finally:
            tracer.export(span)
    """

    _instance: AgentTrace | None = None
    _lock = Lock()

    def __init__(
        self,
        service_name: str = "default",
        exporter: Exporter | None = None,
        endpoint: str = "http://localhost:8080",
        batch_size: int = 100,
        flush_interval: float = 5.0,
        debug: bool = False,
    ) -> None:
        """Initialize AgentTrace client.

        Args:
            service_name: Name of the service for all spans.
            exporter: Custom exporter (defaults to HttpExporter).
            endpoint: Collector endpoint URL.
            batch_size: Number of spans to batch before flushing.
            flush_interval: Seconds between automatic flushes.
            debug: Enable debug logging.
        """
        self.service_name = service_name
        self.exporter = exporter or HttpExporter(endpoint=endpoint)
        self.batch_size = batch_size
        self.flush_interval = flush_interval
        self.debug = debug

        self._buffer: deque[Span] = deque(maxlen=10000)
        self._buffer_lock = Lock()
        self._shutdown = False
        self._flush_thread: Thread | None = None

        # Configure logging
        if debug:
            structlog.configure(
                wrapper_class=structlog.make_filtering_bound_logger(10),  # DEBUG
            )

        # Start background flush thread
        self._start_flush_thread()

        # Register cleanup on exit
        atexit.register(self.shutdown)

        logger.info(
            "agenttrace_initialized",
            service_name=service_name,
            endpoint=endpoint,
        )

    @classmethod
    def get_instance(cls) -> AgentTrace | None:
        """Get the global AgentTrace instance."""
        return cls._instance

    @classmethod
    def configure(
        cls,
        service_name: str = "default",
        endpoint: str = "http://localhost:8080",
        **kwargs: Any,
    ) -> AgentTrace:
        """Configure and return the global AgentTrace instance.

        This is the recommended way to initialize AgentTrace.
        """
        with cls._lock:
            if cls._instance is None:
                cls._instance = cls(
                    service_name=service_name,
                    endpoint=endpoint,
                    **kwargs,
                )
            return cls._instance

    def _start_flush_thread(self) -> None:
        """Start the background flush thread."""

        def flush_loop() -> None:
            while not self._shutdown:
                asyncio.run(self._flush_async())
                for _ in range(int(self.flush_interval * 10)):
                    if self._shutdown:
                        break
                    import time
                    time.sleep(0.1)

        self._flush_thread = Thread(target=flush_loop, daemon=True)
        self._flush_thread.start()

    async def _flush_async(self) -> int:
        """Flush buffered spans asynchronously."""
        spans_to_export: list[Span] = []

        with self._buffer_lock:
            while self._buffer and len(spans_to_export) < self.batch_size:
                spans_to_export.append(self._buffer.popleft())

        if not spans_to_export:
            return 0

        exported = await self.exporter.export_batch(spans_to_export)
        if exported < len(spans_to_export):
            logger.warning(
                "partial_export",
                exported=exported,
                total=len(spans_to_export),
            )
        return exported

    def start_span(
        self,
        operation_name: str,
        *,
        span_kind: SpanKind = SpanKind.INTERNAL,
        span_type: SpanType = SpanType.CUSTOM,
        trace_id: str | None = None,
        parent_span_id: str | None = None,
        attributes: dict[str, Any] | None = None,
    ) -> Span:
        """Start a new span.

        Args:
            operation_name: Name of the operation.
            span_kind: Kind of span (internal, client, server, etc.).
            span_type: High-level type (llm_call, tool_call, etc.).
            trace_id: Optional trace ID (auto-generated if not provided).
            parent_span_id: Optional parent span ID.
            attributes: Optional initial attributes.

        Returns:
            A new Span instance.
        """
        # Inherit trace context if available
        if trace_id is None:
            trace_id = get_current_trace_id() or uuid4().hex

        if parent_span_id is None:
            current = get_current_span()
            if current:
                parent_span_id = current.span_id

        span = Span(
            operation_name=operation_name,
            service_name=self.service_name,
            span_kind=span_kind,
            span_type=span_type,
            trace_id=trace_id,
            parent_span_id=parent_span_id,
            attributes=attributes or {},
        )

        if self.debug:
            logger.debug(
                "span_started",
                span_id=span.span_id,
                operation=operation_name,
                trace_id=trace_id,
            )

        return span

    def span(
        self,
        operation_name: str,
        *,
        span_kind: SpanKind = SpanKind.INTERNAL,
        span_type: SpanType = SpanType.CUSTOM,
        attributes: dict[str, Any] | None = None,
    ) -> _SpanContextManager:
        """Create a span context manager.

        Usage:
            with tracer.span("my_operation") as span:
                span.set_attribute("key", "value")
                do_work()
        """
        return _SpanContextManager(
            self,
            operation_name,
            span_kind=span_kind,
            span_type=span_type,
            attributes=attributes,
        )

    def export(self, span: Span) -> None:
        """Export a span (adds to buffer for batch export)."""
        if not span.ended_at:
            span.end()

        with self._buffer_lock:
            self._buffer.append(span)

            # Flush if buffer is full
            if len(self._buffer) >= self.batch_size:
                Thread(target=lambda: asyncio.run(self._flush_async())).start()

    def flush(self) -> int:
        """Synchronously flush all buffered spans."""
        return asyncio.run(self._flush_async())

    async def flush_async(self) -> int:
        """Asynchronously flush all buffered spans."""
        return await self._flush_async()

    def shutdown(self) -> None:
        """Shutdown the tracer and flush remaining spans."""
        if self._shutdown:
            return

        self._shutdown = True

        # Flush remaining spans
        try:
            remaining = self.flush()
            if remaining > 0:
                logger.info("final_flush", spans_exported=remaining)
        except Exception as e:
            logger.error("shutdown_flush_error", error=str(e))

        # Cleanup exporter
        try:
            asyncio.run(self.exporter.shutdown())
        except Exception as e:
            logger.error("exporter_shutdown_error", error=str(e))

        logger.info("agenttrace_shutdown")

    def trace(
        self,
        operation_name: str | None = None,
        *,
        span_kind: SpanKind = SpanKind.INTERNAL,
        span_type: SpanType = SpanType.CUSTOM,
    ) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        """Decorator to trace a function.

        Usage:
            @tracer.trace("my_function")
            def my_function(x, y):
                return x + y

            @tracer.trace()  # Uses function name
            async def my_async_function():
                pass
        """

        def decorator(func: Callable[..., Any]) -> Callable[..., Any]:
            name = operation_name or func.__name__

            if asyncio.iscoroutinefunction(func):

                async def async_wrapper(*args: Any, **kwargs: Any) -> Any:
                    span = self.start_span(name, span_kind=span_kind, span_type=span_type)
                    with SpanContext(span):
                        try:
                            result = await func(*args, **kwargs)
                            span.end(SpanStatus.OK)
                            return result
                        except Exception as e:
                            span.set_error(e)
                            span.end(SpanStatus.ERROR)
                            raise
                        finally:
                            self.export(span)

                return async_wrapper
            else:

                def sync_wrapper(*args: Any, **kwargs: Any) -> Any:
                    span = self.start_span(name, span_kind=span_kind, span_type=span_type)
                    with SpanContext(span):
                        try:
                            result = func(*args, **kwargs)
                            span.end(SpanStatus.OK)
                            return result
                        except Exception as e:
                            span.set_error(e)
                            span.end(SpanStatus.ERROR)
                            raise
                        finally:
                            self.export(span)

                return sync_wrapper

        return decorator


class _SpanContextManager:
    """Context manager for spans."""

    def __init__(
        self,
        tracer: AgentTrace,
        operation_name: str,
        span_kind: SpanKind,
        span_type: SpanType,
        attributes: dict[str, Any] | None,
    ) -> None:
        self.tracer = tracer
        self.operation_name = operation_name
        self.span_kind = span_kind
        self.span_type = span_type
        self.attributes = attributes
        self.span: Span | None = None
        self._context: SpanContext | None = None

    def __enter__(self) -> Span:
        self.span = self.tracer.start_span(
            self.operation_name,
            span_kind=self.span_kind,
            span_type=self.span_type,
            attributes=self.attributes,
        )
        self._context = SpanContext(self.span)
        self._context.__enter__()
        return self.span

    def __exit__(self, exc_type: type | None, exc_val: Exception | None, exc_tb: object) -> None:
        if self.span is None:
            return

        if self._context:
            self._context.__exit__(exc_type, exc_val, exc_tb)

        if exc_val is not None:
            self.span.set_error(exc_val)
            self.span.end(SpanStatus.ERROR)
        else:
            self.span.end(SpanStatus.OK)

        self.tracer.export(self.span)

    async def __aenter__(self) -> Span:
        return self.__enter__()

    async def __aexit__(
        self, exc_type: type | None, exc_val: Exception | None, exc_tb: object
    ) -> None:
        self.__exit__(exc_type, exc_val, exc_tb)

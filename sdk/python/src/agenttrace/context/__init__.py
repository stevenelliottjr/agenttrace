"""Context management for tracing."""

from __future__ import annotations

from contextvars import ContextVar
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from agenttrace.models import Span

# Context variable for the current span
_current_span: ContextVar[Span | None] = ContextVar("current_span", default=None)

# Context variable for the current trace ID
_current_trace_id: ContextVar[str | None] = ContextVar("current_trace_id", default=None)


def get_current_span() -> Span | None:
    """Get the current active span."""
    return _current_span.get()


def set_current_span(span: Span | None) -> None:
    """Set the current active span."""
    _current_span.set(span)


def get_current_trace_id() -> str | None:
    """Get the current trace ID."""
    return _current_trace_id.get()


def set_current_trace_id(trace_id: str | None) -> None:
    """Set the current trace ID."""
    _current_trace_id.set(trace_id)


class SpanContext:
    """Context manager for span execution."""

    def __init__(self, span: Span) -> None:
        self.span = span
        self._token: object | None = None
        self._trace_token: object | None = None
        self._previous_span: Span | None = None

    def __enter__(self) -> Span:
        self._previous_span = get_current_span()
        self._token = _current_span.set(self.span)
        self._trace_token = _current_trace_id.set(self.span.trace_id)
        return self.span

    def __exit__(self, exc_type: type | None, exc_val: Exception | None, exc_tb: object) -> None:
        if exc_val is not None:
            self.span.set_error(exc_val)
        _current_span.set(self._previous_span)
        if self._previous_span:
            _current_trace_id.set(self._previous_span.trace_id)
        else:
            _current_trace_id.set(None)


__all__ = [
    "get_current_span",
    "set_current_span",
    "get_current_trace_id",
    "set_current_trace_id",
    "SpanContext",
]

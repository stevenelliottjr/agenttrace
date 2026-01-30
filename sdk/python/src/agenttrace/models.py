"""Data models for AgentTrace SDK."""

from __future__ import annotations

from datetime import datetime, timezone
from enum import Enum
from typing import Any
from uuid import uuid4

from pydantic import BaseModel, Field


class SpanKind(str, Enum):
    """Type of span operation."""

    INTERNAL = "internal"
    CLIENT = "client"
    SERVER = "server"
    PRODUCER = "producer"
    CONSUMER = "consumer"


class SpanStatus(str, Enum):
    """Status of a span."""

    UNSET = "unset"
    OK = "ok"
    ERROR = "error"


class SpanType(str, Enum):
    """High-level categorization of span types."""

    LLM_CALL = "llm_call"
    TOOL_CALL = "tool_call"
    AGENT_STEP = "agent_step"
    RETRIEVAL = "retrieval"
    EMBEDDING = "embedding"
    CHAIN = "chain"
    CUSTOM = "custom"


class SpanEvent(BaseModel):
    """An event that occurred during a span."""

    name: str
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    attributes: dict[str, Any] = Field(default_factory=dict)


class Span(BaseModel):
    """A single trace span representing an operation."""

    span_id: str = Field(default_factory=lambda: uuid4().hex[:16])
    trace_id: str = Field(default_factory=lambda: uuid4().hex)
    parent_span_id: str | None = None

    operation_name: str
    service_name: str = "default"
    span_kind: SpanKind = SpanKind.INTERNAL
    span_type: SpanType = SpanType.CUSTOM

    started_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    ended_at: datetime | None = None

    status: SpanStatus = SpanStatus.UNSET
    status_message: str | None = None

    # LLM-specific fields
    model_name: str | None = None
    model_provider: str | None = None
    tokens_in: int | None = None
    tokens_out: int | None = None
    tokens_reasoning: int | None = None

    # Tool-specific fields
    tool_name: str | None = None
    tool_input: dict[str, Any] | None = None
    tool_output: dict[str, Any] | None = None

    # Content previews (truncated for storage)
    prompt_preview: str | None = None
    completion_preview: str | None = None

    # Generic attributes
    attributes: dict[str, Any] = Field(default_factory=dict)
    events: list[SpanEvent] = Field(default_factory=list)

    def end(
        self,
        status: SpanStatus = SpanStatus.OK,
        status_message: str | None = None,
    ) -> Span:
        """Mark the span as ended."""
        self.ended_at = datetime.now(timezone.utc)
        self.status = status
        if status_message:
            self.status_message = status_message
        return self

    def add_event(self, name: str, attributes: dict[str, Any] | None = None) -> Span:
        """Add an event to the span."""
        self.events.append(SpanEvent(name=name, attributes=attributes or {}))
        return self

    def set_attribute(self, key: str, value: Any) -> Span:
        """Set a custom attribute."""
        self.attributes[key] = value
        return self

    def set_error(self, error: Exception) -> Span:
        """Mark the span as errored with exception details."""
        self.status = SpanStatus.ERROR
        self.status_message = str(error)
        self.attributes["error.type"] = type(error).__name__
        self.attributes["error.message"] = str(error)
        return self

    @property
    def duration_ms(self) -> float | None:
        """Calculate duration in milliseconds."""
        if self.ended_at is None:
            return None
        delta = self.ended_at - self.started_at
        return delta.total_seconds() * 1000

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        data = {
            "span_id": self.span_id,
            "trace_id": self.trace_id,
            "parent_span_id": self.parent_span_id,
            "operation_name": self.operation_name,
            "service_name": self.service_name,
            "started_at": self.started_at.isoformat(),
            "ended_at": self.ended_at.isoformat() if self.ended_at else None,
            "status": self.status.value,
            "status_message": self.status_message,
            "model_name": self.model_name,
            "model_provider": self.model_provider,
            "tokens_in": self.tokens_in,
            "tokens_out": self.tokens_out,
            "tokens_reasoning": self.tokens_reasoning,
            "tool_name": self.tool_name,
            "tool_input": self.tool_input,
            "tool_output": self.tool_output,
            "prompt_preview": self.prompt_preview,
            "completion_preview": self.completion_preview,
            "attributes": self.attributes,
        }
        return {k: v for k, v in data.items() if v is not None}

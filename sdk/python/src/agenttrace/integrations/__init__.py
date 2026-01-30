"""Auto-instrumentation integrations for LLM providers."""

from agenttrace.integrations.anthropic import instrument_anthropic
from agenttrace.integrations.openai import instrument_openai

__all__ = ["instrument_anthropic", "instrument_openai"]

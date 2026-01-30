"""Console exporter for debugging and development."""

from __future__ import annotations

import json
import sys
from typing import TextIO

from agenttrace.exporters.base import Exporter
from agenttrace.models import Span


class ConsoleExporter(Exporter):
    """Export spans to console for debugging."""

    def __init__(
        self,
        output: TextIO = sys.stdout,
        pretty: bool = True,
        color: bool = True,
    ) -> None:
        """Initialize console exporter.

        Args:
            output: Output stream (default: stdout).
            pretty: Use pretty formatting.
            color: Use colored output (ANSI codes).
        """
        self.output = output
        self.pretty = pretty
        self.color = color

    def _format_span(self, span: Span) -> str:
        """Format a span for console output."""
        if self.pretty:
            return self._format_pretty(span)
        return json.dumps(span.to_dict(), indent=2, default=str)

    def _format_pretty(self, span: Span) -> str:
        """Format span with colors and structure."""
        lines = []

        # Color codes
        if self.color:
            reset = "\033[0m"
            bold = "\033[1m"
            dim = "\033[2m"
            cyan = "\033[36m"
            green = "\033[32m"
            yellow = "\033[33m"
            red = "\033[31m"
            magenta = "\033[35m"
        else:
            reset = bold = dim = cyan = green = yellow = red = magenta = ""

        # Status color
        status_color = {
            "ok": green,
            "error": red,
            "unset": dim,
        }.get(span.status.value, dim)

        # Header
        lines.append(f"{bold}{cyan}{'─' * 60}{reset}")
        lines.append(f"{bold}SPAN{reset} {dim}[{span.span_id}]{reset}")
        lines.append(f"  {bold}operation:{reset} {span.operation_name}")
        lines.append(f"  {bold}trace_id:{reset} {dim}{span.trace_id}{reset}")

        if span.parent_span_id:
            lines.append(f"  {bold}parent:{reset} {dim}{span.parent_span_id}{reset}")

        lines.append(f"  {bold}service:{reset} {span.service_name}")
        lines.append(f"  {bold}status:{reset} {status_color}{span.status.value}{reset}")

        # Duration
        if span.duration_ms is not None:
            duration_color = yellow if span.duration_ms > 1000 else green
            lines.append(f"  {bold}duration:{reset} {duration_color}{span.duration_ms:.2f}ms{reset}")

        # LLM info
        if span.model_name:
            lines.append(f"  {bold}model:{reset} {magenta}{span.model_name}{reset}")
            if span.model_provider:
                lines.append(f"  {bold}provider:{reset} {span.model_provider}")

        # Tokens
        if any([span.tokens_in, span.tokens_out, span.tokens_reasoning]):
            tokens = []
            if span.tokens_in:
                tokens.append(f"in={span.tokens_in}")
            if span.tokens_out:
                tokens.append(f"out={span.tokens_out}")
            if span.tokens_reasoning:
                tokens.append(f"reasoning={span.tokens_reasoning}")
            lines.append(f"  {bold}tokens:{reset} {yellow}{', '.join(tokens)}{reset}")

        # Tool info
        if span.tool_name:
            lines.append(f"  {bold}tool:{reset} {cyan}{span.tool_name}{reset}")

        # Previews
        if span.prompt_preview:
            preview = span.prompt_preview[:100] + "..." if len(span.prompt_preview) > 100 else span.prompt_preview
            lines.append(f"  {bold}prompt:{reset} {dim}{preview}{reset}")

        if span.completion_preview:
            preview = span.completion_preview[:100] + "..." if len(span.completion_preview) > 100 else span.completion_preview
            lines.append(f"  {bold}completion:{reset} {dim}{preview}{reset}")

        # Error
        if span.status_message:
            lines.append(f"  {bold}message:{reset} {red}{span.status_message}{reset}")

        # Attributes
        if span.attributes:
            lines.append(f"  {bold}attributes:{reset}")
            for key, value in span.attributes.items():
                lines.append(f"    {dim}{key}:{reset} {value}")

        lines.append(f"{cyan}{'─' * 60}{reset}")

        return "\n".join(lines)

    async def export(self, span: Span) -> bool:
        """Export a single span to console."""
        try:
            formatted = self._format_span(span)
            print(formatted, file=self.output)
            return True
        except Exception:
            return False

    async def export_batch(self, spans: list[Span]) -> int:
        """Export a batch of spans to console."""
        count = 0
        for span in spans:
            if await self.export(span):
                count += 1
        return count

    def export_sync(self, span: Span) -> bool:
        """Synchronous span export."""
        try:
            formatted = self._format_span(span)
            print(formatted, file=self.output)
            return True
        except Exception:
            return False

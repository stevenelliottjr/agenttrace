"""Base exporter interface."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from agenttrace.models import Span


class Exporter(ABC):
    """Abstract base class for span exporters."""

    @abstractmethod
    async def export(self, span: Span) -> bool:
        """Export a single span.

        Args:
            span: The span to export.

        Returns:
            True if export was successful, False otherwise.
        """
        ...

    @abstractmethod
    async def export_batch(self, spans: list[Span]) -> int:
        """Export a batch of spans.

        Args:
            spans: List of spans to export.

        Returns:
            Number of spans successfully exported.
        """
        ...

    async def shutdown(self) -> None:
        """Cleanup resources. Override if needed."""
        pass

    def export_sync(self, span: Span) -> bool:
        """Synchronous wrapper for export. Override for better sync support."""
        import asyncio

        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            loop = None

        if loop is not None:
            # We're in an async context, create a task
            import concurrent.futures

            with concurrent.futures.ThreadPoolExecutor() as pool:
                future = pool.submit(asyncio.run, self.export(span))
                return future.result()
        else:
            return asyncio.run(self.export(span))

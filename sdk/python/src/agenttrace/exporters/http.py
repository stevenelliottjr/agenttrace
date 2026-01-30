"""HTTP exporter for sending spans to the AgentTrace collector."""

from __future__ import annotations

import structlog
import httpx

from agenttrace.exporters.base import Exporter
from agenttrace.models import Span

logger = structlog.get_logger(__name__)


class HttpExporter(Exporter):
    """Export spans via HTTP to the AgentTrace collector."""

    def __init__(
        self,
        endpoint: str = "http://localhost:8080",
        timeout: float = 10.0,
        max_retries: int = 3,
    ) -> None:
        """Initialize HTTP exporter.

        Args:
            endpoint: Base URL of the AgentTrace collector.
            timeout: Request timeout in seconds.
            max_retries: Maximum number of retry attempts.
        """
        self.endpoint = endpoint.rstrip("/")
        self.timeout = timeout
        self.max_retries = max_retries
        self._client: httpx.AsyncClient | None = None

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create the HTTP client."""
        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                timeout=httpx.Timeout(self.timeout),
                headers={"Content-Type": "application/json"},
            )
        return self._client

    async def export(self, span: Span) -> bool:
        """Export a single span to the collector.

        Args:
            span: The span to export.

        Returns:
            True if export was successful, False otherwise.
        """
        client = await self._get_client()
        url = f"{self.endpoint}/api/v1/spans"

        for attempt in range(self.max_retries):
            try:
                response = await client.post(url, json=span.to_dict())
                if response.status_code == 200:
                    logger.debug(
                        "span_exported",
                        span_id=span.span_id,
                        trace_id=span.trace_id,
                    )
                    return True
                else:
                    logger.warning(
                        "span_export_failed",
                        span_id=span.span_id,
                        status_code=response.status_code,
                        response=response.text[:200],
                        attempt=attempt + 1,
                    )
            except httpx.RequestError as e:
                logger.warning(
                    "span_export_error",
                    span_id=span.span_id,
                    error=str(e),
                    attempt=attempt + 1,
                )

        return False

    async def export_batch(self, spans: list[Span]) -> int:
        """Export a batch of spans to the collector.

        Args:
            spans: List of spans to export.

        Returns:
            Number of spans successfully exported.
        """
        if not spans:
            return 0

        client = await self._get_client()
        url = f"{self.endpoint}/api/v1/spans/batch"

        for attempt in range(self.max_retries):
            try:
                payload = {"spans": [span.to_dict() for span in spans]}
                response = await client.post(url, json=payload)

                if response.status_code == 200:
                    data = response.json()
                    accepted = data.get("accepted", 0)
                    logger.debug(
                        "batch_exported",
                        total=len(spans),
                        accepted=accepted,
                        rejected=data.get("rejected", 0),
                    )
                    return accepted
                else:
                    logger.warning(
                        "batch_export_failed",
                        total=len(spans),
                        status_code=response.status_code,
                        attempt=attempt + 1,
                    )
            except httpx.RequestError as e:
                logger.warning(
                    "batch_export_error",
                    total=len(spans),
                    error=str(e),
                    attempt=attempt + 1,
                )

        return 0

    async def shutdown(self) -> None:
        """Close the HTTP client."""
        if self._client is not None:
            await self._client.aclose()
            self._client = None

    def export_sync(self, span: Span) -> bool:
        """Synchronous span export using httpx sync client."""
        url = f"{self.endpoint}/api/v1/spans"

        for attempt in range(self.max_retries):
            try:
                response = httpx.post(
                    url,
                    json=span.to_dict(),
                    timeout=self.timeout,
                    headers={"Content-Type": "application/json"},
                )
                if response.status_code == 200:
                    return True
                logger.warning(
                    "span_export_failed_sync",
                    span_id=span.span_id,
                    status_code=response.status_code,
                    attempt=attempt + 1,
                )
            except httpx.RequestError as e:
                logger.warning(
                    "span_export_error_sync",
                    span_id=span.span_id,
                    error=str(e),
                    attempt=attempt + 1,
                )

        return False

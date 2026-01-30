#!/usr/bin/env python3
"""Basic usage example for AgentTrace SDK.

This example demonstrates:
1. Manual span creation with context managers
2. Using the @trace decorator
3. Nested spans with automatic parent linking
4. Console exporter for debugging
"""

import asyncio
import time

from agenttrace import AgentTrace, SpanType
from agenttrace.exporters import ConsoleExporter


def main():
    # Initialize tracer with console exporter for demo
    tracer = AgentTrace(
        service_name="example-agent",
        exporter=ConsoleExporter(pretty=True, color=True),
    )

    # Example 1: Manual span with context manager
    print("\n=== Example 1: Manual span with context manager ===\n")
    with tracer.span("process_user_request", span_type=SpanType.AGENT_STEP) as span:
        span.set_attribute("user_id", "user_123")
        span.set_attribute("request_type", "question")
        time.sleep(0.1)  # Simulate work

        # Nested span - automatically inherits trace_id and parent
        with tracer.span("call_llm", span_type=SpanType.LLM_CALL) as llm_span:
            llm_span.model_name = "claude-3-opus-20240229"
            llm_span.model_provider = "anthropic"
            llm_span.tokens_in = 150
            llm_span.tokens_out = 423
            llm_span.prompt_preview = "What is the capital of France?"
            llm_span.completion_preview = "The capital of France is Paris..."
            time.sleep(0.05)

        # Another nested span for tool use
        with tracer.span("execute_tool", span_type=SpanType.TOOL_CALL) as tool_span:
            tool_span.tool_name = "web_search"
            tool_span.tool_input = {"query": "Paris population 2024"}
            time.sleep(0.03)
            tool_span.tool_output = {"result": "Population: 2.1 million"}

    # Example 2: Using the @trace decorator
    print("\n=== Example 2: Using the @trace decorator ===\n")

    @tracer.trace("decorated_function")
    def my_function(x: int, y: int) -> int:
        time.sleep(0.02)
        return x + y

    result = my_function(5, 3)
    print(f"Function result: {result}")

    # Example 3: Async function tracing
    print("\n=== Example 3: Async function tracing ===\n")

    @tracer.trace("async_operation", span_type=SpanType.CHAIN)
    async def async_operation():
        await asyncio.sleep(0.05)
        return "async result"

    async_result = asyncio.run(async_operation())
    print(f"Async result: {async_result}")

    # Example 4: Error handling
    print("\n=== Example 4: Error handling ===\n")
    try:
        with tracer.span("risky_operation") as span:
            span.set_attribute("attempt", 1)
            raise ValueError("Something went wrong!")
    except ValueError:
        pass  # Error is captured in span

    # Flush any remaining spans
    tracer.flush()
    print("\n=== Done! ===\n")


if __name__ == "__main__":
    main()

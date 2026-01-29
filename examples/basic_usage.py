"""
Basic AgentTrace Usage Example

This example demonstrates how to use AgentTrace to trace an AI agent
that uses the Anthropic API.

Prerequisites:
1. Start AgentTrace: `agenttrace up`
2. Install SDK: `pip install agenttrace[anthropic]`
3. Set ANTHROPIC_API_KEY environment variable
"""

import asyncio
import os
from anthropic import Anthropic

# When implemented, import from agenttrace
# from agenttrace import AgentTrace, SpanType


async def main():
    # Initialize AgentTrace
    # tracer = AgentTrace(
    #     session_name="example-agent",
    #     framework="custom",
    #     tags=["example", "demo"],
    # )
    # await tracer.start()

    # Initialize Anthropic client (will be auto-instrumented)
    client = Anthropic()

    print("Starting agent task...")

    # Example: Simple agent that reads a file and asks Claude to analyze it
    # with tracer.start_span("analyze_code", SpanType.TASK) as task:

    # Step 1: Read a file
    # with tracer.start_span("read_file", SpanType.FILE_READ) as span:
    #     span.set_attribute("file.path", "example.py")
    content = "def hello(): return 'Hello, World!'"

    # Step 2: Ask Claude to analyze (auto-traced)
    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=1024,
        messages=[
            {
                "role": "user",
                "content": f"Analyze this Python code:\n\n```python\n{content}\n```",
            }
        ],
    )

    print(f"Analysis: {response.content[0].text[:200]}...")

    # Step 3: Process response
    # with tracer.start_span("process_response", SpanType.REASONING):
    #     # Custom processing logic
    #     pass

    # Stop tracer and flush remaining spans
    # await tracer.stop()

    print("\nTrace complete! View at http://localhost:3000")


if __name__ == "__main__":
    asyncio.run(main())

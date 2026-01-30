#!/usr/bin/env python3
"""Example of auto-instrumentation with Anthropic SDK.

This example demonstrates automatic tracing of Anthropic API calls.

Prerequisites:
    pip install agenttrace[anthropic]
    export ANTHROPIC_API_KEY=your-api-key

Usage:
    python anthropic_tracing.py
"""

import os

from agenttrace import AgentTrace
from agenttrace.integrations import instrument_anthropic


def main():
    # Check for API key
    if not os.environ.get("ANTHROPIC_API_KEY"):
        print("Please set ANTHROPIC_API_KEY environment variable")
        return

    # Configure AgentTrace
    # For development, you can use ConsoleExporter:
    # from agenttrace.exporters import ConsoleExporter
    # tracer = AgentTrace(service_name="my-agent", exporter=ConsoleExporter())

    # For production, send to the collector:
    tracer = AgentTrace.configure(
        service_name="anthropic-demo",
        endpoint="http://localhost:8080",  # AgentTrace collector
        debug=True,
    )

    # Instrument Anthropic SDK - all API calls will be traced automatically
    instrument_anthropic(tracer)

    # Now use Anthropic as normal
    from anthropic import Anthropic

    client = Anthropic()

    # This call will be automatically traced
    print("Making API call (check collector for traces)...")
    response = client.messages.create(
        model="claude-3-haiku-20240307",
        max_tokens=100,
        messages=[{"role": "user", "content": "Say hello in 3 languages"}],
    )

    print(f"\nResponse: {response.content[0].text}")
    print(f"Tokens used: {response.usage.input_tokens} in, {response.usage.output_tokens} out")

    # Nested tracing works too - manual span with auto-traced API call
    with tracer.span("my_agent_turn") as span:
        span.set_attribute("turn", 1)

        response = client.messages.create(
            model="claude-3-haiku-20240307",
            max_tokens=50,
            messages=[{"role": "user", "content": "What is 2+2?"}],
        )
        span.set_attribute("got_response", True)

    print(f"\nNested response: {response.content[0].text}")

    # Flush and cleanup
    tracer.shutdown()
    print("\nDone! Check your AgentTrace collector for the traces.")


if __name__ == "__main__":
    main()

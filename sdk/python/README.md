# AgentTrace Python SDK

Observability SDK for AI Agents. Automatically trace LLM calls, tool executions, and agent reasoning.

## Installation

```bash
pip install agenttrace
```

With optional integrations:

```bash
pip install agenttrace[anthropic]  # Anthropic auto-instrumentation
pip install agenttrace[openai]     # OpenAI auto-instrumentation
pip install agenttrace[all]        # All integrations
```

## Quick Start

```python
from agenttrace import AgentTrace
import anthropic

# Initialize tracer (auto-instruments Anthropic)
tracer = AgentTrace(session_name="my-agent")
await tracer.start()

# Your code - traces are captured automatically
client = anthropic.Anthropic()
response = client.messages.create(
    model="claude-sonnet-4-20250514",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Hello!"}]
)

# View traces at http://localhost:3000
await tracer.stop()
```

## Manual Instrumentation

```python
from agenttrace import AgentTrace, SpanType

tracer = AgentTrace()

with tracer.start_span("my_operation", SpanType.TOOL_EXECUTION) as span:
    span.set_attribute("tool.name", "calculator")
    result = do_something()
    span.set_attribute("tool.result", result)
```

## Documentation

See the [main repository](https://github.com/stevenelliottjr/agenttrace) for full documentation.

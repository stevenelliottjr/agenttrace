# AgentTrace Python SDK

Observability SDK for AI Agents. Trace LLM calls, tool usage, and agent workflows with minimal code changes.

## Installation

```bash
pip install agenttrace
```

With auto-instrumentation support:
```bash
pip install agenttrace[anthropic]  # For Anthropic SDK
pip install agenttrace[openai]     # For OpenAI SDK
pip install agenttrace[all]        # All integrations
```

## Quick Start

### Basic Usage

```python
from agenttrace import AgentTrace, SpanType

# Configure the global tracer
tracer = AgentTrace.configure(
    service_name="my-ai-agent",
    endpoint="http://localhost:8080",  # AgentTrace collector
)

# Use context manager for manual tracing
with tracer.span("process_request") as span:
    span.set_attribute("user_id", "123")
    result = do_work()

# Use decorator for function tracing
@tracer.trace("my_function")
def my_function(x, y):
    return x + y

# Async functions work too
@tracer.trace("async_operation")
async def async_operation():
    await do_async_work()
```

### Auto-Instrumentation (Recommended)

The SDK can automatically trace all API calls to Anthropic or OpenAI:

```python
from agenttrace import AgentTrace
from agenttrace.integrations import instrument_anthropic
from anthropic import Anthropic

# Configure tracer
tracer = AgentTrace.configure(service_name="my-agent")

# Enable auto-instrumentation
instrument_anthropic(tracer)

# All Anthropic calls are now automatically traced!
client = Anthropic()
response = client.messages.create(
    model="claude-3-opus-20240229",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### Console Exporter (Development)

For local development without a collector:

```python
from agenttrace import AgentTrace
from agenttrace.exporters import ConsoleExporter

tracer = AgentTrace(
    service_name="dev-agent",
    exporter=ConsoleExporter(pretty=True, color=True),
)
```

## Features

### LLM Call Tracing

```python
with tracer.span("llm_call", span_type=SpanType.LLM_CALL) as span:
    span.model_name = "claude-3-opus-20240229"
    span.model_provider = "anthropic"
    span.tokens_in = 150
    span.tokens_out = 423
    span.prompt_preview = "What is the capital of France?"
    span.completion_preview = "The capital of France is Paris..."

    # Make your API call
    response = client.messages.create(...)
```

### Tool Call Tracing

```python
with tracer.span("tool_call", span_type=SpanType.TOOL_CALL) as span:
    span.tool_name = "web_search"
    span.tool_input = {"query": "Paris population"}

    result = execute_tool(...)

    span.tool_output = {"result": result}
```

### Nested Spans

Spans automatically inherit trace context:

```python
with tracer.span("agent_turn") as parent:
    # Child spans link to parent
    with tracer.span("think") as think_span:
        pass

    with tracer.span("act") as act_span:
        # Grandchild spans also link correctly
        with tracer.span("call_api") as api_span:
            pass
```

### Error Handling

Errors are automatically captured:

```python
try:
    with tracer.span("risky_operation") as span:
        raise ValueError("Something went wrong!")
except ValueError:
    pass  # Error details captured in span
```

Or manually set errors:

```python
with tracer.span("operation") as span:
    try:
        risky_call()
    except Exception as e:
        span.set_error(e)
        # Handle error...
```

### Custom Attributes

Add any metadata to spans:

```python
with tracer.span("operation") as span:
    span.set_attribute("user_id", "123")
    span.set_attribute("request_type", "chat")
    span.set_attribute("temperature", 0.7)
```

### Events

Record notable moments within a span:

```python
with tracer.span("long_operation") as span:
    span.add_event("checkpoint_1", {"progress": 25})
    do_work()
    span.add_event("checkpoint_2", {"progress": 50})
    do_more_work()
```

## API Reference

### AgentTrace

The main client class.

```python
AgentTrace(
    service_name: str = "default",
    exporter: Exporter | None = None,      # Defaults to HttpExporter
    endpoint: str = "http://localhost:8080",
    batch_size: int = 100,                  # Spans before auto-flush
    flush_interval: float = 5.0,            # Seconds between flushes
    debug: bool = False,                    # Enable debug logging
)
```

**Class Methods:**
- `AgentTrace.configure(**kwargs)` - Configure global singleton
- `AgentTrace.get_instance()` - Get the global instance

**Instance Methods:**
- `start_span(name, ...)` - Create a new span
- `span(name, ...)` - Context manager for spans
- `trace(name, ...)` - Decorator for function tracing
- `export(span)` - Export a span
- `flush()` - Flush buffered spans
- `shutdown()` - Cleanup and flush remaining spans

### Span

Represents a single traced operation.

**Fields:**
- `span_id`, `trace_id`, `parent_span_id` - Identifiers
- `operation_name`, `service_name` - Names
- `span_kind`, `span_type` - Classification
- `started_at`, `ended_at` - Timestamps
- `status`, `status_message` - Status
- `model_name`, `model_provider` - LLM info
- `tokens_in`, `tokens_out`, `tokens_reasoning` - Token usage
- `tool_name`, `tool_input`, `tool_output` - Tool info
- `prompt_preview`, `completion_preview` - Content previews
- `attributes` - Custom key-value pairs
- `events` - List of span events

**Methods:**
- `end(status, message)` - End the span
- `set_attribute(key, value)` - Add custom attribute
- `set_error(exception)` - Mark as errored
- `add_event(name, attributes)` - Add event
- `to_dict()` - Serialize for export

### Enums

```python
from agenttrace import SpanKind, SpanStatus, SpanType

# SpanKind
SpanKind.INTERNAL  # Default
SpanKind.CLIENT    # Outbound call
SpanKind.SERVER    # Handling request

# SpanStatus
SpanStatus.UNSET   # Default
SpanStatus.OK      # Success
SpanStatus.ERROR   # Failure

# SpanType
SpanType.LLM_CALL     # LLM API call
SpanType.TOOL_CALL    # Tool execution
SpanType.AGENT_STEP   # Agent step
SpanType.RETRIEVAL    # RAG retrieval
SpanType.EMBEDDING    # Embedding generation
SpanType.CHAIN        # Chain execution
SpanType.CUSTOM       # Default
```

## Configuration

### Environment Variables

The SDK respects these environment variables:

- `AGENTTRACE_ENDPOINT` - Collector endpoint URL
- `AGENTTRACE_SERVICE_NAME` - Default service name
- `AGENTTRACE_DEBUG` - Enable debug mode

### Exporters

**HttpExporter** (Default)
```python
from agenttrace.exporters import HttpExporter

exporter = HttpExporter(
    endpoint="http://localhost:8080",
    timeout=10.0,
    max_retries=3,
)
```

**ConsoleExporter** (Development)
```python
from agenttrace.exporters import ConsoleExporter

exporter = ConsoleExporter(
    pretty=True,   # Formatted output
    color=True,    # ANSI colors
)
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.

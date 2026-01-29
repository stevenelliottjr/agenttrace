# AgentTrace Examples

This directory contains example code showing how to use AgentTrace.

## Examples

### basic_usage.py

Basic example showing manual and automatic instrumentation with the Anthropic SDK.

```bash
# Prerequisites
agenttrace up
pip install agenttrace[anthropic]

# Run
python basic_usage.py
```

## Coming Soon

- `langchain_agent.py` - Tracing a LangChain agent
- `openai_example.py` - OpenAI SDK auto-instrumentation
- `custom_spans.py` - Advanced manual instrumentation
- `async_agent.py` - Async agent patterns

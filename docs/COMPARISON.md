# AgentTrace vs Alternatives

> How AgentTrace compares to other observability and tracing solutions

---

## Quick Comparison

| Feature | AgentTrace | LangSmith | Helicone | OpenTelemetry | Datadog |
|---------|------------|-----------|----------|---------------|---------|
| **AI Agent Focus** | Yes | Yes | Partial | No | No |
| **Local-First** | Yes | No | No | Yes | No |
| **Token Tracking** | Built-in | Yes | Yes | Manual | Manual |
| **Cost Attribution** | Built-in | Yes | Yes | Manual | Manual |
| **Reasoning Chains** | Native | Native | No | Manual | Manual |
| **Open Source** | MIT | No | Partial | Apache-2.0 | No |
| **Self-Hosted** | Yes | Enterprise | No | Yes | No |
| **Cloud Required** | No | Yes | Yes | Optional | Yes |
| **Pricing** | Free | Paid tiers | Paid tiers | Free | Paid |

---

## Detailed Comparison

### vs LangSmith

[LangSmith](https://smith.langchain.com/) is LangChain's observability platform.

| Aspect | AgentTrace | LangSmith |
|--------|------------|-----------|
| **Data Location** | Your machine | LangChain cloud |
| **Privacy** | Full control | Data sent to cloud |
| **Vendor Lock-in** | None (MIT) | LangChain ecosystem |
| **Framework Support** | Any framework | LangChain-focused |
| **Pricing** | Free forever | Free tier + paid |
| **Self-Hosted** | Always | Enterprise only |

**Choose AgentTrace if:**
- Privacy is critical (sensitive code, conversations)
- You use multiple frameworks (not just LangChain)
- You want full control over your data
- You prefer open source

**Choose LangSmith if:**
- You're deeply invested in LangChain
- You want managed infrastructure
- Collaboration features are important

---

### vs Helicone

[Helicone](https://helicone.ai/) provides LLM observability as a proxy.

| Aspect | AgentTrace | Helicone |
|--------|------------|----------|
| **Architecture** | SDK-based | Proxy-based |
| **Data Location** | Local | Helicone cloud |
| **Latency Impact** | <1ms | Network hop added |
| **Setup** | One-liner install | Change API endpoint |
| **Agent Semantics** | Native (traces, spans) | Request-focused |
| **Open Source** | Fully | Partially |

**Choose AgentTrace if:**
- You need full trace visualization (not just requests)
- Local-first privacy matters
- You want sub-millisecond overhead

**Choose Helicone if:**
- You want quick setup without code changes
- Cloud-managed solution is preferred
- You mainly need request logging

---

### vs OpenTelemetry

[OpenTelemetry](https://opentelemetry.io/) is the standard for application observability.

| Aspect | AgentTrace | OpenTelemetry |
|--------|------------|---------------|
| **Focus** | AI agents | General apps |
| **Token Tracking** | Built-in | Manual |
| **Cost Calculation** | Automatic | Not included |
| **Semantic Conventions** | AI-specific | Generic spans |
| **Learning Curve** | Low | High |
| **Dashboard** | Included | Bring your own |

**Choose AgentTrace if:**
- You're building AI agents specifically
- You want batteries-included (dashboard, cost tracking)
- You want AI-specific semantics out of the box

**Choose OpenTelemetry if:**
- You have existing OTel infrastructure
- You need to trace non-AI components too
- You want maximum flexibility

**Note:** AgentTrace can export to OpenTelemetry collectors, giving you the best of both worlds.

---

### vs Datadog / New Relic

Traditional APM tools can trace applications but lack AI-specific features.

| Aspect | AgentTrace | Traditional APM |
|--------|------------|-----------------|
| **AI Focus** | Native | None |
| **Token Metrics** | Built-in | Manual instrumentation |
| **Cost Attribution** | Automatic | Custom dashboards |
| **Reasoning Viz** | Purpose-built | Generic traces |
| **Pricing** | Free | $$$$ |
| **Self-Hosted** | Yes | Limited/No |

**Choose AgentTrace if:**
- You're building AI agents
- You want cost tracking without custom work
- Budget is a concern

**Choose Traditional APM if:**
- You have existing APM investment
- You need to trace full application stack
- Enterprise support is required

---

### vs Build Your Own

Many teams start with custom logging and dashboards.

| Aspect | AgentTrace | DIY |
|--------|------------|-----|
| **Time to Value** | Minutes | Weeks/Months |
| **Maintenance** | Community | Your team |
| **Features** | Complete | What you build |
| **Performance** | Optimized | Variable |
| **Best Practices** | Built-in | Learned over time |

**Choose AgentTrace if:**
- You want to focus on your agent, not infrastructure
- You value proven patterns
- You want to avoid reinventing the wheel

**Choose DIY if:**
- You have very specific requirements
- You have dedicated infrastructure team
- You enjoy building observability tools

---

## Feature Matrix

### Data Collection

| Feature | AgentTrace | LangSmith | Helicone | OTel |
|---------|------------|-----------|----------|------|
| Auto-instrumentation | Yes | Yes | Via proxy | Manual |
| Manual spans | Yes | Yes | Limited | Yes |
| Streaming support | Yes | Yes | Yes | Yes |
| Batch ingestion | Yes | Yes | No | Yes |
| UDP support | Yes | No | No | Yes |

### Storage & Query

| Feature | AgentTrace | LangSmith | Helicone | OTel |
|---------|------------|-----------|----------|------|
| Time-series optimized | Yes | Unknown | Unknown | Backend-dependent |
| Full-text search | Yes | Yes | Yes | Backend-dependent |
| Trace aggregations | Yes | Yes | Limited | Backend-dependent |
| Data retention control | Full | Limited | Limited | Backend-dependent |
| Export capability | Yes | Limited | Yes | Yes |

### Visualization

| Feature | AgentTrace | LangSmith | Helicone | OTel |
|---------|------------|-----------|----------|------|
| Trace waterfall | Yes | Yes | No | Jaeger/Tempo |
| Cost dashboards | Yes | Yes | Yes | Manual |
| Real-time updates | Yes | Yes | Yes | Backend-dependent |
| Terminal UI | Yes | No | No | No |
| Custom dashboards | Planned | Yes | Limited | Backend-dependent |

### AI-Specific

| Feature | AgentTrace | LangSmith | Helicone | OTel |
|---------|------------|-----------|----------|------|
| Token counting | Native | Native | Native | Manual |
| Cost calculation | 100+ models | LangChain models | Major providers | Manual |
| Prompt versioning | Planned | Yes | No | No |
| A/B testing | Planned | Yes | No | No |
| Reasoning chains | Native | Native | No | Manual |

---

## Migration Paths

### From LangSmith

```python
# Before (LangSmith)
from langsmith import traceable

@traceable
def my_function():
    ...

# After (AgentTrace)
from agenttrace import trace

@trace
def my_function():
    ...
```

### From Helicone

```python
# Before (Helicone proxy)
client = OpenAI(
    base_url="https://oai.hconeai.com/v1"
)

# After (AgentTrace)
from agenttrace import auto_instrument
auto_instrument()

client = OpenAI()  # Normal usage, traces captured
```

### From OpenTelemetry

```python
# AgentTrace can export to OTel collectors
from agenttrace import AgentTrace

tracer = AgentTrace(
    exporters=["local", "otlp://collector:4317"]
)
```

---

## Summary

**AgentTrace is the right choice when:**

1. **Privacy matters** — Data stays on your machine
2. **You use multiple frameworks** — Not locked to one ecosystem
3. **You want AI-specific features** — Token tracking, cost attribution built-in
4. **Open source is important** — MIT licensed, community-driven
5. **Budget is a concern** — Free forever, self-hosted

**Consider alternatives when:**

1. You're deeply invested in a specific ecosystem (LangSmith for LangChain)
2. You need managed cloud infrastructure
3. You have existing APM investment you want to leverage
4. Enterprise support contracts are required

---

<p align="center">
<em>Have questions about which tool is right for you?<br>
Open a <a href="https://github.com/stevenelliottjr/agenttrace/discussions">GitHub Discussion</a>!</em>
</p>

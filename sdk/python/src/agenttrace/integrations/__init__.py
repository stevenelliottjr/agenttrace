"""Auto-instrumentation integrations for LLM providers."""

from __future__ import annotations

from typing import TYPE_CHECKING

import structlog

from agenttrace.integrations.anthropic import (
    instrument_anthropic,
    uninstrument_anthropic,
)
from agenttrace.integrations.openai import (
    instrument_openai,
    uninstrument_openai,
)
from agenttrace.integrations.langchain import (
    instrument_langchain,
    uninstrument_langchain,
    get_callback_handler,
    AgentTraceCallbackHandler,
)
from agenttrace.integrations.litellm import (
    instrument_litellm,
    uninstrument_litellm,
)

if TYPE_CHECKING:
    from agenttrace.client import AgentTrace

logger = structlog.get_logger(__name__)

__all__ = [
    "instrument_anthropic",
    "instrument_openai",
    "instrument_langchain",
    "instrument_litellm",
    "uninstrument_anthropic",
    "uninstrument_openai",
    "uninstrument_langchain",
    "uninstrument_litellm",
    "get_callback_handler",
    "AgentTraceCallbackHandler",
    "auto_instrument",
    "uninstrument_all",
]


def auto_instrument(tracer: "AgentTrace | None" = None) -> dict[str, bool]:
    """Automatically instrument all available LLM libraries.

    This function attempts to instrument all supported libraries that are
    installed. It silently skips libraries that aren't available.

    Args:
        tracer: Optional AgentTrace instance. If not provided,
            uses the global instance from AgentTrace.configure().

    Returns:
        A dictionary mapping library names to whether they were
        successfully instrumented.

    Example:
        from agenttrace import AgentTrace
        from agenttrace.integrations import auto_instrument

        tracer = AgentTrace.configure(service_name="my-agent")
        results = auto_instrument(tracer)
        # results = {"openai": True, "anthropic": True, "langchain": False, "litellm": False}

        # Now all installed LLM libraries are automatically traced
        from openai import OpenAI
        client = OpenAI()
        response = client.chat.completions.create(...)  # Traced!
    """
    results: dict[str, bool] = {}

    # Try OpenAI
    try:
        import openai  # noqa: F401
        try:
            instrument_openai(tracer)
            results["openai"] = True
            logger.debug("auto_instrument_openai_success")
        except Exception as e:
            results["openai"] = False
            logger.debug("auto_instrument_openai_failed", error=str(e))
    except ImportError:
        results["openai"] = False

    # Try Anthropic
    try:
        import anthropic  # noqa: F401
        try:
            instrument_anthropic(tracer)
            results["anthropic"] = True
            logger.debug("auto_instrument_anthropic_success")
        except Exception as e:
            results["anthropic"] = False
            logger.debug("auto_instrument_anthropic_failed", error=str(e))
    except ImportError:
        results["anthropic"] = False

    # Try LangChain
    try:
        import langchain_core  # noqa: F401
        try:
            instrument_langchain(tracer)
            results["langchain"] = True
            logger.debug("auto_instrument_langchain_success")
        except Exception as e:
            results["langchain"] = False
            logger.debug("auto_instrument_langchain_failed", error=str(e))
    except ImportError:
        results["langchain"] = False

    # Try LiteLLM
    try:
        import litellm  # noqa: F401
        try:
            instrument_litellm(tracer)
            results["litellm"] = True
            logger.debug("auto_instrument_litellm_success")
        except Exception as e:
            results["litellm"] = False
            logger.debug("auto_instrument_litellm_failed", error=str(e))
    except ImportError:
        results["litellm"] = False

    instrumented = [k for k, v in results.items() if v]
    if instrumented:
        logger.info("auto_instrument_complete", libraries=instrumented)
    else:
        logger.warning("auto_instrument_none", message="No LLM libraries found to instrument")

    return results


def uninstrument_all() -> None:
    """Remove instrumentation from all libraries.

    This is useful for testing or when you need to restore original behavior.
    """
    uninstrument_openai()
    uninstrument_anthropic()
    uninstrument_langchain()
    uninstrument_litellm()
    logger.info("uninstrument_all_complete")

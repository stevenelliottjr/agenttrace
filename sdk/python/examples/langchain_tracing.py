"""Example of using AgentTrace with LangChain.

This example shows two ways to integrate AgentTrace with LangChain:
1. Global instrumentation (automatic for all LangChain calls)
2. Per-call callback handler (explicit control)
"""

import asyncio

from agenttrace import AgentTrace
from agenttrace.integrations.langchain import (
    instrument_langchain,
    get_callback_handler,
)


async def example_global_instrumentation():
    """Example using global instrumentation."""
    # Initialize AgentTrace
    tracer = AgentTrace.configure(
        service_name="langchain-agent",
        collector_url="localhost:4317",
    )

    # Instrument LangChain globally - all calls will be traced
    instrument_langchain(tracer)

    # Import LangChain after instrumentation
    from langchain_openai import ChatOpenAI
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.output_parsers import StrOutputParser

    # Create a simple chain
    llm = ChatOpenAI(model="gpt-4o-mini")
    prompt = ChatPromptTemplate.from_messages([
        ("system", "You are a helpful assistant."),
        ("human", "{input}"),
    ])
    chain = prompt | llm | StrOutputParser()

    # All calls are automatically traced
    response = await chain.ainvoke({"input": "What is the capital of France?"})
    print(f"Response: {response}")

    await tracer.shutdown()


async def example_callback_handler():
    """Example using explicit callback handler."""
    from langchain_openai import ChatOpenAI
    from langchain_anthropic import ChatAnthropic
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.output_parsers import StrOutputParser

    # Initialize AgentTrace
    tracer = AgentTrace.configure(
        service_name="langchain-agent",
        collector_url="localhost:4317",
    )

    # Get a callback handler
    handler = get_callback_handler(tracer)

    # Create chains with different LLMs
    openai_llm = ChatOpenAI(model="gpt-4o-mini")
    anthropic_llm = ChatAnthropic(model="claude-3-5-haiku-latest")

    prompt = ChatPromptTemplate.from_messages([
        ("system", "You are a helpful assistant. Be concise."),
        ("human", "{input}"),
    ])

    openai_chain = prompt | openai_llm | StrOutputParser()
    anthropic_chain = prompt | anthropic_llm | StrOutputParser()

    # Pass the callback handler explicitly
    config = {"callbacks": [handler]}

    # Trace OpenAI call
    response1 = await openai_chain.ainvoke(
        {"input": "What is 2+2?"},
        config=config
    )
    print(f"OpenAI: {response1}")

    # Trace Anthropic call
    response2 = await anthropic_chain.ainvoke(
        {"input": "What is 3+3?"},
        config=config
    )
    print(f"Anthropic: {response2}")

    await tracer.shutdown()


async def example_with_tools():
    """Example tracing LangChain tools."""
    from langchain_openai import ChatOpenAI
    from langchain_core.tools import tool
    from langchain.agents import AgentExecutor, create_tool_calling_agent
    from langchain_core.prompts import ChatPromptTemplate

    # Initialize AgentTrace
    tracer = AgentTrace.configure(
        service_name="langchain-tool-agent",
        collector_url="localhost:4317",
    )
    handler = get_callback_handler(tracer)

    # Define tools
    @tool
    def get_weather(location: str) -> str:
        """Get the weather for a location."""
        return f"The weather in {location} is sunny and 72°F."

    @tool
    def search_web(query: str) -> str:
        """Search the web for information."""
        return f"Search results for '{query}': Found 10 relevant articles."

    # Create agent
    llm = ChatOpenAI(model="gpt-4o-mini")
    tools = [get_weather, search_web]

    prompt = ChatPromptTemplate.from_messages([
        ("system", "You are a helpful assistant with access to tools."),
        ("human", "{input}"),
        ("placeholder", "{agent_scratchpad}"),
    ])

    agent = create_tool_calling_agent(llm, tools, prompt)
    agent_executor = AgentExecutor(agent=agent, tools=tools, verbose=True)

    # Run with tracing
    response = await agent_executor.ainvoke(
        {"input": "What's the weather like in San Francisco?"},
        config={"callbacks": [handler]},
    )
    print(f"Agent response: {response['output']}")

    await tracer.shutdown()


async def example_with_retriever():
    """Example tracing LangChain retrievers."""
    from langchain_openai import ChatOpenAI, OpenAIEmbeddings
    from langchain_community.vectorstores import FAISS
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.runnables import RunnablePassthrough
    from langchain_core.output_parsers import StrOutputParser

    # Initialize AgentTrace
    tracer = AgentTrace.configure(
        service_name="langchain-rag-agent",
        collector_url="localhost:4317",
    )
    handler = get_callback_handler(tracer)

    # Create a simple vector store
    texts = [
        "Paris is the capital of France.",
        "London is the capital of the United Kingdom.",
        "Berlin is the capital of Germany.",
        "Rome is the capital of Italy.",
    ]
    embeddings = OpenAIEmbeddings()
    vectorstore = FAISS.from_texts(texts, embeddings)
    retriever = vectorstore.as_retriever()

    # Create RAG chain
    llm = ChatOpenAI(model="gpt-4o-mini")
    prompt = ChatPromptTemplate.from_messages([
        ("system", "Answer based on the context: {context}"),
        ("human", "{question}"),
    ])

    def format_docs(docs):
        return "\n".join(doc.page_content for doc in docs)

    rag_chain = (
        {"context": retriever | format_docs, "question": RunnablePassthrough()}
        | prompt
        | llm
        | StrOutputParser()
    )

    # Run with tracing - retriever calls will be traced
    response = await rag_chain.ainvoke(
        "What is the capital of France?",
        config={"callbacks": [handler]},
    )
    print(f"RAG response: {response}")

    await tracer.shutdown()


if __name__ == "__main__":
    print("=== Global Instrumentation Example ===")
    asyncio.run(example_global_instrumentation())

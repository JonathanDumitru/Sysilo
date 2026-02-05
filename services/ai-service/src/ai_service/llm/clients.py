"""LLM client implementations."""

from abc import ABC, abstractmethod
from typing import Any, AsyncGenerator

import structlog
from openai import AsyncOpenAI
from anthropic import AsyncAnthropic

from ai_service.config import get_settings

settings = get_settings()
logger = structlog.get_logger()

# Client instances
_openai_client: AsyncOpenAI | None = None
_anthropic_client: AsyncAnthropic | None = None


class LLMClient(ABC):
    """Abstract base class for LLM clients."""

    @abstractmethod
    async def generate(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> str:
        """Generate a response from the LLM."""
        pass

    @abstractmethod
    async def generate_stream(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> AsyncGenerator[str, None]:
        """Generate a streaming response from the LLM."""
        pass

    @abstractmethod
    async def embed(
        self,
        text: str | list[str],
        model: str | None = None,
    ) -> list[list[float]]:
        """Generate embeddings for text."""
        pass


class OpenAIClient(LLMClient):
    """OpenAI LLM client."""

    def __init__(self, client: AsyncOpenAI):
        self.client = client

    async def generate(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> str:
        response = await self.client.chat.completions.create(
            model=model or settings.default_model,
            messages=messages,  # type: ignore
            temperature=temperature or settings.temperature,
            max_tokens=max_tokens or settings.max_tokens,
            **kwargs,
        )
        return response.choices[0].message.content or ""

    async def generate_stream(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> AsyncGenerator[str, None]:
        stream = await self.client.chat.completions.create(
            model=model or settings.default_model,
            messages=messages,  # type: ignore
            temperature=temperature or settings.temperature,
            max_tokens=max_tokens or settings.max_tokens,
            stream=True,
            **kwargs,
        )
        async for chunk in stream:
            if chunk.choices[0].delta.content:
                yield chunk.choices[0].delta.content

    async def embed(
        self,
        text: str | list[str],
        model: str | None = None,
    ) -> list[list[float]]:
        if isinstance(text, str):
            text = [text]

        response = await self.client.embeddings.create(
            model=model or settings.embedding_model,
            input=text,
        )
        return [item.embedding for item in response.data]


class AnthropicClient(LLMClient):
    """Anthropic LLM client."""

    def __init__(self, client: AsyncAnthropic):
        self.client = client

    async def generate(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> str:
        # Extract system message if present
        system_message = None
        filtered_messages = []
        for msg in messages:
            if msg["role"] == "system":
                system_message = msg["content"]
            else:
                filtered_messages.append(msg)

        response = await self.client.messages.create(
            model=model or "claude-3-sonnet-20240229",
            messages=filtered_messages,  # type: ignore
            system=system_message or "",
            temperature=temperature or settings.temperature,
            max_tokens=max_tokens or settings.max_tokens,
            **kwargs,
        )
        return response.content[0].text if response.content else ""

    async def generate_stream(
        self,
        messages: list[dict[str, str]],
        model: str | None = None,
        temperature: float | None = None,
        max_tokens: int | None = None,
        **kwargs: Any,
    ) -> AsyncGenerator[str, None]:
        # Extract system message if present
        system_message = None
        filtered_messages = []
        for msg in messages:
            if msg["role"] == "system":
                system_message = msg["content"]
            else:
                filtered_messages.append(msg)

        async with self.client.messages.stream(
            model=model or "claude-3-sonnet-20240229",
            messages=filtered_messages,  # type: ignore
            system=system_message or "",
            temperature=temperature or settings.temperature,
            max_tokens=max_tokens or settings.max_tokens,
            **kwargs,
        ) as stream:
            async for text in stream.text_stream:
                yield text

    async def embed(
        self,
        text: str | list[str],
        model: str | None = None,
    ) -> list[list[float]]:
        # Anthropic doesn't have embeddings - fallback to OpenAI
        raise NotImplementedError("Anthropic does not support embeddings. Use OpenAI client.")


async def init_llm_clients() -> None:
    """Initialize LLM clients based on configuration."""
    global _openai_client, _anthropic_client

    if settings.openai_api_key:
        _openai_client = AsyncOpenAI(api_key=settings.openai_api_key)
        logger.info("Initialized OpenAI client")

    if settings.anthropic_api_key:
        _anthropic_client = AsyncAnthropic(api_key=settings.anthropic_api_key)
        logger.info("Initialized Anthropic client")

    if not _openai_client and not _anthropic_client:
        logger.warning("No LLM clients initialized - API keys not configured")


def get_llm_client(provider: str | None = None) -> LLMClient:
    """Get an LLM client for the specified provider."""
    provider = provider or settings.default_llm_provider

    if provider == "openai" and _openai_client:
        return OpenAIClient(_openai_client)
    elif provider == "anthropic" and _anthropic_client:
        return AnthropicClient(_anthropic_client)
    elif _openai_client:
        return OpenAIClient(_openai_client)
    elif _anthropic_client:
        return AnthropicClient(_anthropic_client)
    else:
        raise RuntimeError("No LLM client available - configure API keys")


def get_embedding_client() -> LLMClient:
    """Get the embedding client (always OpenAI)."""
    if _openai_client:
        return OpenAIClient(_openai_client)
    raise RuntimeError("OpenAI client required for embeddings - configure OPENAI_API_KEY")

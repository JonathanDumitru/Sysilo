"""Conversational AI chat endpoints."""

from typing import Literal
from uuid import UUID, uuid4

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field
import structlog

from ai_service.llm import get_llm_client, PromptManager

router = APIRouter()
logger = structlog.get_logger()


class ChatMessage(BaseModel):
    """A single chat message."""

    role: Literal["user", "assistant"]
    content: str


class ChatRequest(BaseModel):
    """Chat request payload."""

    message: str = Field(..., min_length=1, max_length=10000)
    conversation_id: UUID | None = None
    context: str = "general"
    history: list[ChatMessage] = Field(default_factory=list)
    stream: bool = False
    tenant_id: UUID | None = None


class ChatResponse(BaseModel):
    """Chat response payload."""

    conversation_id: UUID
    message: str
    context: str


@router.post("", response_model=ChatResponse)
async def chat(request: ChatRequest) -> ChatResponse | StreamingResponse:
    """Send a message and get an AI response."""
    conversation_id = request.conversation_id or uuid4()

    logger.info(
        "Processing chat request",
        conversation_id=str(conversation_id),
        context=request.context,
        message_length=len(request.message),
    )

    try:
        client = get_llm_client()

        # Build message history
        history = [{"role": msg.role, "content": msg.content} for msg in request.history]

        messages = PromptManager.build_messages(
            user_message=request.message,
            context=request.context,
            conversation_history=history,
        )

        if request.stream:
            async def generate():
                async for chunk in client.generate_stream(messages):
                    yield f"data: {chunk}\n\n"
                yield "data: [DONE]\n\n"

            return StreamingResponse(
                generate(),
                media_type="text/event-stream",
            )

        response = await client.generate(messages)

        return ChatResponse(
            conversation_id=conversation_id,
            message=response,
            context=request.context,
        )

    except Exception as e:
        logger.error("Chat request failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class QueryRequest(BaseModel):
    """Natural language query request."""

    question: str = Field(..., min_length=1, max_length=2000)
    query_type: Literal["cypher", "sql"] = "cypher"
    execute: bool = False
    tenant_id: UUID | None = None


class QueryResponse(BaseModel):
    """Query generation response."""

    question: str
    query: str
    query_type: str
    results: list[dict] | None = None


@router.post("/query", response_model=QueryResponse)
async def generate_query(request: QueryRequest) -> QueryResponse:
    """Generate a database query from natural language."""
    logger.info(
        "Generating query",
        query_type=request.query_type,
        question_length=len(request.question),
    )

    try:
        client = get_llm_client()

        # Select appropriate context
        context = "cypher_generation" if request.query_type == "cypher" else "sql_generation"

        messages = PromptManager.build_messages(
            user_message=request.question,
            context=context,
        )

        query = await client.generate(messages, temperature=0.0)

        # Clean up the query
        query = query.strip()
        if query.startswith("```"):
            lines = query.split("\n")
            query = "\n".join(lines[1:-1] if lines[-1] == "```" else lines[1:])

        response = QueryResponse(
            question=request.question,
            query=query,
            query_type=request.query_type,
        )

        # Execute query if requested
        if request.execute:
            if request.query_type == "cypher":
                response.results = await _execute_cypher(query)
            else:
                response.results = await _execute_sql(query)

        return response

    except Exception as e:
        logger.error("Query generation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


async def _execute_cypher(query: str) -> list[dict]:
    """Execute a Cypher query against Neo4j."""
    from ai_service.db import get_neo4j

    driver = await get_neo4j()
    if not driver:
        raise HTTPException(status_code=503, detail="Neo4j not available")

    async with driver.session() as session:
        result = await session.run(query)
        records = await result.data()
        return records


async def _execute_sql(query: str) -> list[dict]:
    """Execute a SQL query against PostgreSQL."""
    from ai_service.db import engine
    from sqlalchemy import text

    async with engine.connect() as conn:
        result = await conn.execute(text(query))
        rows = result.fetchall()
        columns = result.keys()
        return [dict(zip(columns, row)) for row in rows]

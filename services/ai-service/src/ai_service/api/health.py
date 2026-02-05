"""Health check endpoints."""

from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter()


class HealthResponse(BaseModel):
    """Health check response."""

    status: str
    service: str
    version: str


class ReadyResponse(BaseModel):
    """Readiness check response."""

    status: str
    postgres: bool
    neo4j: bool
    redis: bool
    llm: bool


@router.get("/health", response_model=HealthResponse)
async def health_check() -> HealthResponse:
    """Basic health check endpoint."""
    return HealthResponse(
        status="healthy",
        service="ai-service",
        version="0.1.0",
    )


@router.get("/ready", response_model=ReadyResponse)
async def readiness_check() -> ReadyResponse:
    """Readiness check with dependency status."""
    from ai_service.db import get_neo4j, get_redis, engine
    from ai_service.llm import get_llm_client

    # Check PostgreSQL
    postgres_ok = False
    try:
        async with engine.connect() as conn:
            await conn.execute("SELECT 1")
            postgres_ok = True
    except Exception:
        pass

    # Check Neo4j
    neo4j_ok = False
    try:
        driver = await get_neo4j()
        if driver:
            async with driver.session() as session:
                await session.run("RETURN 1")
                neo4j_ok = True
    except Exception:
        pass

    # Check Redis
    redis_ok = False
    try:
        redis = await get_redis()
        if redis:
            await redis.ping()
            redis_ok = True
    except Exception:
        pass

    # Check LLM availability
    llm_ok = False
    try:
        get_llm_client()
        llm_ok = True
    except Exception:
        pass

    return ReadyResponse(
        status="ready" if all([postgres_ok, llm_ok]) else "degraded",
        postgres=postgres_ok,
        neo4j=neo4j_ok,
        redis=redis_ok,
        llm=llm_ok,
    )

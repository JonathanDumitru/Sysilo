"""Main FastAPI application for Sysilo AI Service."""

import structlog
from contextlib import asynccontextmanager
from typing import AsyncGenerator

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from ai_service.config import get_settings
from ai_service.api import (
    chat_router,
    recommendations_router,
    insights_router,
    embeddings_router,
    health_router,
    traces_router,
    prompts_api_router,
    drift_api_router,
)
from ai_service.db import init_db, close_db
from ai_service.llm import init_llm_clients


settings = get_settings()

# Configure structured logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.JSONRenderer(),
    ],
    wrapper_class=structlog.stdlib.BoundLogger,
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
)

logger = structlog.get_logger()


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan manager for startup/shutdown."""
    # Startup
    logger.info("Starting AI Service", environment=settings.environment)

    await init_db()
    await init_llm_clients()

    logger.info("AI Service started successfully", port=settings.port)

    yield

    # Shutdown
    logger.info("Shutting down AI Service")
    await close_db()
    logger.info("AI Service shutdown complete")


app = FastAPI(
    title="Sysilo AI Service",
    description="Conversational AI, recommendations, and intelligent insights for the Sysilo platform",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(health_router, tags=["Health"])
app.include_router(chat_router, prefix="/chat", tags=["Chat"])
app.include_router(recommendations_router, prefix="/recommendations", tags=["Recommendations"])
app.include_router(insights_router, prefix="/insights", tags=["Insights"])
app.include_router(embeddings_router, prefix="/embeddings", tags=["Embeddings"])
app.include_router(traces_router, prefix="/traces", tags=["Traces"])
app.include_router(prompts_api_router, prefix="/prompts", tags=["Prompts"])
app.include_router(drift_api_router, prefix="/drift", tags=["Drift Detection"])


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "ai_service.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.debug,
    )

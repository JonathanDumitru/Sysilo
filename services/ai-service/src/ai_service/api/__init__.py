"""API routers for AI Service."""

from ai_service.api.chat import router as chat_router
from ai_service.api.recommendations import router as recommendations_router
from ai_service.api.insights import router as insights_router
from ai_service.api.embeddings import router as embeddings_router
from ai_service.api.health import router as health_router
from ai_service.api.traces import router as traces_router
from ai_service.api.prompts_api import router as prompts_api_router

__all__ = [
    "chat_router",
    "recommendations_router",
    "insights_router",
    "embeddings_router",
    "health_router",
    "traces_router",
    "prompts_api_router",
]

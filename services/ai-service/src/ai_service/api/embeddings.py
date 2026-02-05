"""Embedding generation and semantic search endpoints."""

from uuid import UUID

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field
import structlog

from ai_service.llm import get_embedding_client

router = APIRouter()
logger = structlog.get_logger()


class EmbeddingRequest(BaseModel):
    """Embedding generation request."""

    tenant_id: UUID
    texts: list[str] = Field(..., min_length=1, max_length=100)
    model: str | None = None


class EmbeddingResponse(BaseModel):
    """Embedding generation response."""

    embeddings: list[list[float]]
    model: str
    dimensions: int


@router.post("/generate", response_model=EmbeddingResponse)
async def generate_embeddings(request: EmbeddingRequest) -> EmbeddingResponse:
    """Generate embeddings for text."""
    logger.info(
        "Generating embeddings",
        tenant_id=str(request.tenant_id),
        text_count=len(request.texts),
    )

    try:
        client = get_embedding_client()
        embeddings = await client.embed(request.texts, model=request.model)

        return EmbeddingResponse(
            embeddings=embeddings,
            model=request.model or "text-embedding-3-small",
            dimensions=len(embeddings[0]) if embeddings else 0,
        )

    except Exception as e:
        logger.error("Embedding generation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class SemanticSearchRequest(BaseModel):
    """Semantic search request."""

    tenant_id: UUID
    query: str = Field(..., min_length=1, max_length=1000)
    collection: str  # e.g., "applications", "integrations", "documentation"
    top_k: int = Field(default=10, ge=1, le=100)
    min_score: float = Field(default=0.7, ge=0.0, le=1.0)


class SearchResult(BaseModel):
    """A single search result."""

    id: str
    score: float
    content: str
    metadata: dict


class SemanticSearchResponse(BaseModel):
    """Semantic search response."""

    query: str
    results: list[SearchResult]
    total_found: int


@router.post("/search", response_model=SemanticSearchResponse)
async def semantic_search(request: SemanticSearchRequest) -> SemanticSearchResponse:
    """Perform semantic search using embeddings."""
    logger.info(
        "Performing semantic search",
        tenant_id=str(request.tenant_id),
        collection=request.collection,
        query_length=len(request.query),
    )

    try:
        # Generate query embedding
        client = get_embedding_client()
        query_embeddings = await client.embed([request.query])
        query_embedding = query_embeddings[0]

        # In production, this would search a vector database (Pinecone, Weaviate, pgvector, etc.)
        # For now, return mock results
        results = [
            SearchResult(
                id="1",
                score=0.95,
                content=f"Sample result for query: {request.query[:50]}",
                metadata={"collection": request.collection, "type": "example"},
            )
        ]

        return SemanticSearchResponse(
            query=request.query,
            results=results,
            total_found=len(results),
        )

    except Exception as e:
        logger.error("Semantic search failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class SimilarityRequest(BaseModel):
    """Similarity calculation request."""

    tenant_id: UUID
    text1: str = Field(..., min_length=1, max_length=5000)
    text2: str = Field(..., min_length=1, max_length=5000)


class SimilarityResponse(BaseModel):
    """Similarity calculation response."""

    similarity: float
    text1_length: int
    text2_length: int


@router.post("/similarity", response_model=SimilarityResponse)
async def calculate_similarity(request: SimilarityRequest) -> SimilarityResponse:
    """Calculate semantic similarity between two texts."""
    logger.info(
        "Calculating similarity",
        tenant_id=str(request.tenant_id),
        text1_length=len(request.text1),
        text2_length=len(request.text2),
    )

    try:
        client = get_embedding_client()
        embeddings = await client.embed([request.text1, request.text2])

        # Calculate cosine similarity
        import numpy as np

        vec1 = np.array(embeddings[0])
        vec2 = np.array(embeddings[1])

        similarity = float(np.dot(vec1, vec2) / (np.linalg.norm(vec1) * np.linalg.norm(vec2)))

        return SimilarityResponse(
            similarity=similarity,
            text1_length=len(request.text1),
            text2_length=len(request.text2),
        )

    except Exception as e:
        logger.error("Similarity calculation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))

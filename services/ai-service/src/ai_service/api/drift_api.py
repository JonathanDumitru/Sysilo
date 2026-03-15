"""Semantic drift detection and business glossary API endpoints."""

from typing import Optional

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field
import structlog

from ai_service.db import async_session
from ai_service.llm import get_llm_client
from ai_service.drift.detector import SemanticDriftDetector
from ai_service.drift.glossary import BusinessGlossary

router = APIRouter()
logger = structlog.get_logger()

# ---------------------------------------------------------------------------
# Shared service instances
# ---------------------------------------------------------------------------

_detector: Optional[SemanticDriftDetector] = None
_glossary: Optional[BusinessGlossary] = None


def get_detector() -> SemanticDriftDetector:
    """Get or create the shared drift detector instance."""
    global _detector
    if _detector is None:
        llm = get_llm_client()
        _detector = SemanticDriftDetector(db_pool=async_session, llm_client=llm)
    return _detector


def get_glossary() -> BusinessGlossary:
    """Get or create the shared glossary instance."""
    global _glossary
    if _glossary is None:
        try:
            llm = get_llm_client()
        except RuntimeError:
            llm = None
        _glossary = BusinessGlossary(db_pool=async_session, llm_client=llm)
    return _glossary


# ---------------------------------------------------------------------------
# Request / Response models
# ---------------------------------------------------------------------------

class ScanRequest(BaseModel):
    tenant_id: str
    scope: Optional[str] = Field(None, description="Optional scope filter (system or dataset)")


class FieldDefinitionResponse(BaseModel):
    source_system: str
    dataset: str
    field_name: str
    data_type: str
    description: Optional[str] = None
    sample_values: list[str] = Field(default_factory=list)
    business_glossary_term: Optional[str] = None
    last_updated: Optional[str] = None


class ConflictResponse(BaseModel):
    id: str
    concept_name: str
    definitions: list[FieldDefinitionResponse] = Field(default_factory=list)
    drift_score: float
    drift_type: str
    explanation: str
    detected_at: Optional[str] = None
    lineage_path: Optional[str] = None
    severity: str
    status: str


class ScanResultResponse(BaseModel):
    scan_id: str
    tenant_id: str
    scanned_at: Optional[str] = None
    total_fields_scanned: int
    conflicts_found: int
    conflicts: list[ConflictResponse] = Field(default_factory=list)
    overall_drift_score: float


class ConflictListResponse(BaseModel):
    conflicts: list[ConflictResponse]
    total: int


class UpdateStatusRequest(BaseModel):
    status: str = Field(..., description="New status: open, acknowledged, resolved, false_positive")
    resolution_note: Optional[str] = None


class TrendsResponse(BaseModel):
    dates: list[str] = Field(default_factory=list)
    scores: list[float] = Field(default_factory=list)
    conflict_counts: list[int] = Field(default_factory=list)


# -- Glossary models -------------------------------------------------------

class CreateGlossaryTermRequest(BaseModel):
    tenant_id: str
    term: str = Field(..., min_length=1, max_length=255)
    canonical_definition: str = Field(..., min_length=1)
    synonyms: list[str] = Field(default_factory=list)
    domain: str = Field(default="")
    owner: Optional[str] = None
    approved: bool = False


class UpdateGlossaryTermRequest(BaseModel):
    term: Optional[str] = None
    canonical_definition: Optional[str] = None
    synonyms: Optional[list[str]] = None
    domain: Optional[str] = None
    owner: Optional[str] = None
    approved: Optional[bool] = None


class GlossaryTermResponse(BaseModel):
    id: str
    tenant_id: str
    term: str
    canonical_definition: str
    synonyms: list[str] = Field(default_factory=list)
    domain: str
    owner: Optional[str] = None
    approved: bool
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class GlossaryListResponse(BaseModel):
    terms: list[GlossaryTermResponse]
    total: int


# ---------------------------------------------------------------------------
# Drift detection endpoints
# ---------------------------------------------------------------------------

@router.post("/scan", response_model=ScanResultResponse)
async def trigger_drift_scan(request: ScanRequest) -> ScanResultResponse:
    """Trigger a drift detection scan across connected systems."""
    logger.info(
        "Triggering drift scan",
        tenant_id=request.tenant_id,
        scope=request.scope,
    )

    try:
        detector = get_detector()
        result = await detector.scan_for_drift(
            tenant_id=request.tenant_id,
            scope=request.scope,
        )

        result_dict = result.to_dict()
        return ScanResultResponse(**result_dict)
    except Exception as e:
        logger.error("Failed to run drift scan", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/conflicts", response_model=ConflictListResponse)
async def list_conflicts(
    tenant_id: str = Query(..., description="Tenant ID"),
    status: Optional[str] = Query(None, description="Filter by status"),
    min_severity: Optional[str] = Query(None, description="Minimum severity level"),
) -> ConflictListResponse:
    """List detected semantic conflicts with optional filters."""
    logger.info(
        "Listing drift conflicts",
        tenant_id=tenant_id,
        status=status,
        min_severity=min_severity,
    )

    try:
        detector = get_detector()
        conflicts = await detector.get_conflicts(
            tenant_id=tenant_id,
            status=status,
            min_severity=min_severity,
        )

        conflict_responses = [
            ConflictResponse(**c.to_dict()) for c in conflicts
        ]

        return ConflictListResponse(
            conflicts=conflict_responses,
            total=len(conflict_responses),
        )
    except Exception as e:
        logger.error("Failed to list conflicts", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/conflicts/{conflict_id}", response_model=ConflictResponse)
async def get_conflict(conflict_id: str) -> ConflictResponse:
    """Get a single conflict with full details."""
    logger.info("Getting conflict", conflict_id=conflict_id)

    try:
        detector = get_detector()
        conflict = await detector.get_conflict(conflict_id)

        if not conflict:
            raise HTTPException(
                status_code=404,
                detail=f"Conflict {conflict_id} not found",
            )

        return ConflictResponse(**conflict.to_dict())
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to get conflict", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.put("/conflicts/{conflict_id}/status")
async def update_conflict_status(
    conflict_id: str,
    request: UpdateStatusRequest,
) -> dict:
    """Update the status of a conflict (acknowledge, resolve, mark false positive)."""
    logger.info(
        "Updating conflict status",
        conflict_id=conflict_id,
        status=request.status,
    )

    try:
        detector = get_detector()

        # Verify conflict exists
        conflict = await detector.get_conflict(conflict_id)
        if not conflict:
            raise HTTPException(
                status_code=404,
                detail=f"Conflict {conflict_id} not found",
            )

        await detector.update_conflict_status(
            conflict_id=conflict_id,
            status=request.status,
            resolution_note=request.resolution_note,
        )

        return {"status": "updated", "conflict_id": conflict_id, "new_status": request.status}
    except HTTPException:
        raise
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error("Failed to update conflict status", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/trends", response_model=TrendsResponse)
async def get_drift_trends(
    tenant_id: str = Query(..., description="Tenant ID"),
    days: int = Query(30, ge=1, le=365, description="Number of days to look back"),
) -> TrendsResponse:
    """Get drift score trends over time."""
    logger.info(
        "Getting drift trends",
        tenant_id=tenant_id,
        days=days,
    )

    try:
        detector = get_detector()
        trends = await detector.get_drift_trends(tenant_id=tenant_id, days=days)

        return TrendsResponse(**trends)
    except Exception as e:
        logger.error("Failed to get drift trends", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


# ---------------------------------------------------------------------------
# Glossary endpoints
# ---------------------------------------------------------------------------

@router.get("/glossary", response_model=GlossaryListResponse)
async def list_glossary_terms(
    tenant_id: str = Query(..., description="Tenant ID"),
    domain: Optional[str] = Query(None, description="Filter by domain"),
) -> GlossaryListResponse:
    """List business glossary terms."""
    logger.info("Listing glossary terms", tenant_id=tenant_id, domain=domain)

    try:
        glossary = get_glossary()
        terms = await glossary.list_terms(tenant_id=tenant_id, domain=domain)

        term_responses = [GlossaryTermResponse(**t.to_dict()) for t in terms]

        return GlossaryListResponse(
            terms=term_responses,
            total=len(term_responses),
        )
    except Exception as e:
        logger.error("Failed to list glossary terms", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/glossary", response_model=GlossaryTermResponse)
async def create_glossary_term(request: CreateGlossaryTermRequest) -> GlossaryTermResponse:
    """Create a new business glossary term."""
    logger.info(
        "Creating glossary term",
        tenant_id=request.tenant_id,
        term=request.term,
    )

    try:
        glossary = get_glossary()
        term = await glossary.create_term(
            tenant_id=request.tenant_id,
            term_data={
                "term": request.term,
                "canonical_definition": request.canonical_definition,
                "synonyms": request.synonyms,
                "domain": request.domain,
                "owner": request.owner,
                "approved": request.approved,
            },
        )

        return GlossaryTermResponse(**term.to_dict())
    except Exception as e:
        logger.error("Failed to create glossary term", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.put("/glossary/{term_id}", response_model=GlossaryTermResponse)
async def update_glossary_term(
    term_id: str,
    request: UpdateGlossaryTermRequest,
) -> GlossaryTermResponse:
    """Update an existing glossary term."""
    logger.info("Updating glossary term", term_id=term_id)

    try:
        glossary = get_glossary()

        # Build updates dict from non-None fields
        updates = {k: v for k, v in request.model_dump().items() if v is not None}
        if not updates:
            raise HTTPException(status_code=400, detail="No fields to update")

        term = await glossary.update_term(term_id=term_id, updates=updates)
        return GlossaryTermResponse(**term.to_dict())
    except HTTPException:
        raise
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.error("Failed to update glossary term", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.delete("/glossary/{term_id}")
async def delete_glossary_term(term_id: str) -> dict:
    """Delete a glossary term."""
    logger.info("Deleting glossary term", term_id=term_id)

    try:
        glossary = get_glossary()

        # Verify it exists
        existing = await glossary.get_term(term_id)
        if not existing:
            raise HTTPException(
                status_code=404,
                detail=f"Glossary term {term_id} not found",
            )

        await glossary.delete_term(term_id)
        return {"status": "deleted", "term_id": term_id}
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to delete glossary term", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/glossary/search", response_model=GlossaryListResponse)
async def search_glossary_terms(
    tenant_id: str = Query(..., description="Tenant ID"),
    q: str = Query(..., min_length=1, description="Search query"),
) -> GlossaryListResponse:
    """Search glossary terms by name, definition, or synonyms."""
    logger.info("Searching glossary terms", tenant_id=tenant_id, query=q)

    try:
        glossary = get_glossary()
        terms = await glossary.search_terms(tenant_id=tenant_id, query=q)

        term_responses = [GlossaryTermResponse(**t.to_dict()) for t in terms]

        return GlossaryListResponse(
            terms=term_responses,
            total=len(term_responses),
        )
    except Exception as e:
        logger.error("Failed to search glossary terms", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))

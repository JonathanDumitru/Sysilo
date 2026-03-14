"""Trace observability API endpoints."""

from typing import Optional

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field
import structlog

from ai_service.db import async_session
from ai_service.observability.tracer import AgentTracer

router = APIRouter()
logger = structlog.get_logger()

# Shared tracer instance
_tracer: Optional[AgentTracer] = None


def get_tracer() -> AgentTracer:
    """Get or create the shared tracer instance."""
    global _tracer
    if _tracer is None:
        _tracer = AgentTracer(db_pool=async_session)
    return _tracer


# ---- Response models ----

class SpanResponse(BaseModel):
    span_id: str
    parent_span_id: Optional[str] = None
    name: str
    span_type: str
    start_time: Optional[str] = None
    end_time: Optional[str] = None
    duration_ms: Optional[float] = None
    status: str
    input_data: Optional[dict] = None
    output_data: Optional[dict] = None
    error: Optional[str] = None
    metadata: dict = Field(default_factory=dict)
    model: Optional[str] = None
    input_tokens: int = 0
    output_tokens: int = 0
    total_tokens: int = 0
    cost_usd: Optional[float] = None


class TraceResponse(BaseModel):
    trace_id: str
    tenant_id: str
    session_id: Optional[str] = None
    name: str
    start_time: Optional[str] = None
    end_time: Optional[str] = None
    total_duration_ms: Optional[float] = None
    status: str
    spans: list = Field(default_factory=list)
    total_tokens: int = 0
    total_cost_usd: float = 0.0
    metadata: dict = Field(default_factory=dict)


class TraceListResponse(BaseModel):
    traces: list[TraceResponse]
    total: int


class CostSummaryResponse(BaseModel):
    total_cost: float
    by_model: dict = Field(default_factory=dict)
    by_operation: dict = Field(default_factory=dict)
    token_counts: dict = Field(default_factory=dict)
    trace_count: int


# ---- Endpoints ----

@router.get("/costs", response_model=CostSummaryResponse)
async def get_cost_summary(
    tenant_id: str = Query(..., description="Tenant ID"),
    start_date: Optional[str] = Query(None, description="Start date (ISO format)"),
    end_date: Optional[str] = Query(None, description="End date (ISO format)"),
) -> CostSummaryResponse:
    """Get cost summary for a tenant."""
    logger.info(
        "Getting cost summary",
        tenant_id=tenant_id,
        start_date=start_date,
        end_date=end_date,
    )

    try:
        tracer = get_tracer()
        summary = await tracer.get_cost_summary(
            tenant_id=tenant_id,
            start_date=start_date,
            end_date=end_date,
        )

        return CostSummaryResponse(
            total_cost=summary["total_cost"],
            by_model=summary["by_model"],
            by_operation=summary["by_operation"],
            token_counts=summary["token_counts"],
            trace_count=summary["trace_count"],
        )
    except Exception as e:
        logger.error("Failed to get cost summary", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/{trace_id}", response_model=TraceResponse)
async def get_trace(
    trace_id: str,
    tenant_id: str = Query(..., description="Tenant ID"),
) -> TraceResponse:
    """Get a trace with all spans."""
    logger.info("Getting trace", trace_id=trace_id, tenant_id=tenant_id)

    try:
        tracer = get_tracer()
        trace = await tracer.get_trace(tenant_id=tenant_id, trace_id=trace_id)

        if not trace:
            raise HTTPException(status_code=404, detail=f"Trace {trace_id} not found")

        trace_dict = trace.to_dict()
        return TraceResponse(**trace_dict)
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to get trace", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("", response_model=TraceListResponse)
async def list_traces(
    tenant_id: str = Query(..., description="Tenant ID"),
    limit: int = Query(50, ge=1, le=200, description="Max results"),
    offset: int = Query(0, ge=0, description="Offset for pagination"),
    session_id: Optional[str] = Query(None, description="Filter by session ID"),
) -> TraceListResponse:
    """List traces for a tenant."""
    logger.info(
        "Listing traces",
        tenant_id=tenant_id,
        limit=limit,
        offset=offset,
        session_id=session_id,
    )

    try:
        tracer = get_tracer()
        traces = await tracer.list_traces(
            tenant_id=tenant_id,
            limit=limit,
            offset=offset,
            session_id=session_id,
        )

        trace_responses = []
        for t in traces:
            trace_responses.append(TraceResponse(**t.to_dict()))

        return TraceListResponse(
            traces=trace_responses,
            total=len(trace_responses),
        )
    except Exception as e:
        logger.error("Failed to list traces", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))

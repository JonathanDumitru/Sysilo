"""Agent execution tracing and observability."""

from dataclasses import dataclass, field, asdict
from datetime import datetime
from typing import Optional
import uuid
import time
import json

import structlog

logger = structlog.get_logger()

# Cost per token for supported models
MODEL_COSTS = {
    "gpt-4": {"input": 0.03 / 1000, "output": 0.06 / 1000},
    "gpt-3.5-turbo": {"input": 0.0005 / 1000, "output": 0.0015 / 1000},
    "claude-sonnet-4-20250514": {"input": 0.003 / 1000, "output": 0.015 / 1000},
    "claude-opus-4-20250514": {"input": 0.015 / 1000, "output": 0.075 / 1000},
}


def _calculate_cost(model: str, input_tokens: int, output_tokens: int) -> Optional[float]:
    """Calculate cost in USD for a given model and token counts."""
    costs = MODEL_COSTS.get(model)
    if not costs:
        return None
    return (input_tokens * costs["input"]) + (output_tokens * costs["output"])


@dataclass
class TraceSpan:
    span_id: str
    parent_span_id: Optional[str]
    name: str
    span_type: str  # "llm_call", "tool_call", "retrieval", "processing"
    start_time: datetime
    end_time: Optional[datetime] = None
    duration_ms: Optional[float] = None
    status: str = "running"  # running, completed, error
    input_data: Optional[dict] = None
    output_data: Optional[dict] = None
    error: Optional[str] = None
    metadata: dict = field(default_factory=dict)
    # LLM-specific
    model: Optional[str] = None
    input_tokens: int = 0
    output_tokens: int = 0
    total_tokens: int = 0
    cost_usd: Optional[float] = None

    def to_dict(self) -> dict:
        """Serialize to a JSON-safe dictionary."""
        d = asdict(self)
        d["start_time"] = self.start_time.isoformat() if self.start_time else None
        d["end_time"] = self.end_time.isoformat() if self.end_time else None
        return d


@dataclass
class Trace:
    trace_id: str
    tenant_id: str
    session_id: Optional[str]
    name: str
    start_time: datetime
    end_time: Optional[datetime] = None
    total_duration_ms: Optional[float] = None
    status: str = "running"
    spans: list = field(default_factory=list)
    total_tokens: int = 0
    total_cost_usd: float = 0.0
    metadata: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        """Serialize to a JSON-safe dictionary."""
        d = asdict(self)
        d["start_time"] = self.start_time.isoformat() if self.start_time else None
        d["end_time"] = self.end_time.isoformat() if self.end_time else None
        d["spans"] = [
            s.to_dict() if isinstance(s, TraceSpan) else s for s in self.spans
        ]
        return d

    @classmethod
    def from_dict(cls, data: dict) -> "Trace":
        """Deserialize from a dictionary."""
        spans_data = data.pop("spans", [])
        if data.get("start_time") and isinstance(data["start_time"], str):
            data["start_time"] = datetime.fromisoformat(data["start_time"])
        if data.get("end_time") and isinstance(data["end_time"], str):
            data["end_time"] = datetime.fromisoformat(data["end_time"])
        trace = cls(**data)
        trace.spans = spans_data  # Keep as dicts from DB
        return trace


class AgentTracer:
    """Traces AI agent execution with spans for each step."""

    def __init__(self, db_pool=None):
        self.db_pool = db_pool
        self.active_traces: dict[str, Trace] = {}
        self._span_start_times: dict[str, float] = {}
        self._trace_start_times: dict[str, float] = {}

    def start_trace(
        self,
        tenant_id: str,
        name: str,
        session_id: str = None,
        metadata: dict = None,
    ) -> str:
        """Start a new trace, returns trace_id."""
        trace_id = str(uuid.uuid4())
        now = datetime.utcnow()

        trace = Trace(
            trace_id=trace_id,
            tenant_id=tenant_id,
            session_id=session_id,
            name=name,
            start_time=now,
            metadata=metadata or {},
        )

        self.active_traces[trace_id] = trace
        self._trace_start_times[trace_id] = time.monotonic()

        logger.info(
            "Trace started",
            trace_id=trace_id,
            tenant_id=tenant_id,
            name=name,
        )

        return trace_id

    def start_span(
        self,
        trace_id: str,
        name: str,
        span_type: str,
        parent_span_id: str = None,
        input_data: dict = None,
        metadata: dict = None,
    ) -> str:
        """Start a span within a trace, returns span_id."""
        trace = self.active_traces.get(trace_id)
        if not trace:
            raise ValueError(f"Trace {trace_id} not found or not active")

        span_id = str(uuid.uuid4())
        now = datetime.utcnow()

        span = TraceSpan(
            span_id=span_id,
            parent_span_id=parent_span_id,
            name=name,
            span_type=span_type,
            start_time=now,
            input_data=input_data,
            metadata=metadata or {},
        )

        trace.spans.append(span)
        self._span_start_times[span_id] = time.monotonic()

        logger.debug(
            "Span started",
            trace_id=trace_id,
            span_id=span_id,
            name=name,
            span_type=span_type,
        )

        return span_id

    def end_span(
        self,
        trace_id: str,
        span_id: str,
        output_data: dict = None,
        error: str = None,
        tokens: dict = None,
        cost: float = None,
    ):
        """End a span with results."""
        trace = self.active_traces.get(trace_id)
        if not trace:
            raise ValueError(f"Trace {trace_id} not found or not active")

        span: Optional[TraceSpan] = None
        for s in trace.spans:
            if isinstance(s, TraceSpan) and s.span_id == span_id:
                span = s
                break

        if not span:
            raise ValueError(f"Span {span_id} not found in trace {trace_id}")

        now = datetime.utcnow()
        span.end_time = now

        start_mono = self._span_start_times.pop(span_id, None)
        if start_mono is not None:
            span.duration_ms = round((time.monotonic() - start_mono) * 1000, 2)

        if error:
            span.status = "error"
            span.error = error
        else:
            span.status = "completed"

        span.output_data = output_data

        if tokens:
            span.input_tokens = tokens.get("input", 0)
            span.output_tokens = tokens.get("output", 0)
            span.total_tokens = tokens.get("total", span.input_tokens + span.output_tokens)
            span.model = tokens.get("model")

        # Calculate cost
        if cost is not None:
            span.cost_usd = cost
        elif span.model and (span.input_tokens or span.output_tokens):
            span.cost_usd = _calculate_cost(span.model, span.input_tokens, span.output_tokens)

        logger.debug(
            "Span ended",
            trace_id=trace_id,
            span_id=span_id,
            status=span.status,
            duration_ms=span.duration_ms,
        )

    def end_trace(self, trace_id: str, status: str = "completed"):
        """End a trace, calculate totals, persist to DB."""
        trace = self.active_traces.get(trace_id)
        if not trace:
            raise ValueError(f"Trace {trace_id} not found or not active")

        now = datetime.utcnow()
        trace.end_time = now
        trace.status = status

        start_mono = self._trace_start_times.pop(trace_id, None)
        if start_mono is not None:
            trace.total_duration_ms = round((time.monotonic() - start_mono) * 1000, 2)

        # Calculate totals from spans
        total_tokens = 0
        total_cost = 0.0
        for span in trace.spans:
            if isinstance(span, TraceSpan):
                total_tokens += span.total_tokens
                if span.cost_usd:
                    total_cost += span.cost_usd
                # Mark any still-running spans as error
                if span.status == "running":
                    span.status = "error"
                    span.error = "Trace ended before span completed"
                    span.end_time = now

        trace.total_tokens = total_tokens
        trace.total_cost_usd = round(total_cost, 8)

        logger.info(
            "Trace ended",
            trace_id=trace_id,
            status=status,
            duration_ms=trace.total_duration_ms,
            total_tokens=trace.total_tokens,
            total_cost_usd=trace.total_cost_usd,
        )

        # Remove from active and persist
        del self.active_traces[trace_id]
        return trace

    async def _persist_trace(self, trace: Trace):
        """Persist a completed trace to the database."""
        if not self.db_pool:
            logger.warning("No db_pool configured, trace not persisted", trace_id=trace.trace_id)
            return

        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(
                text(
                    """
                    INSERT INTO ai_traces (trace_id, tenant_id, session_id, trace_data, created_at)
                    VALUES (:trace_id, :tenant_id, :session_id, :trace_data, :created_at)
                    ON CONFLICT (trace_id) DO UPDATE SET trace_data = :trace_data
                    """
                ),
                {
                    "trace_id": trace.trace_id,
                    "tenant_id": trace.tenant_id,
                    "session_id": trace.session_id,
                    "trace_data": json.dumps(trace.to_dict()),
                    "created_at": trace.start_time,
                },
            )
            await session.commit()

    async def get_trace(self, tenant_id: str, trace_id: str) -> Optional[Trace]:
        """Retrieve a stored trace."""
        if not self.db_pool:
            return None

        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text(
                    """
                    SELECT trace_data FROM ai_traces
                    WHERE trace_id = :trace_id AND tenant_id = :tenant_id
                    """
                ),
                {"trace_id": trace_id, "tenant_id": tenant_id},
            )
            row = result.fetchone()
            if not row:
                return None

            data = row[0] if isinstance(row[0], dict) else json.loads(row[0])
            return Trace.from_dict(data)

    async def list_traces(
        self,
        tenant_id: str,
        limit: int = 50,
        offset: int = 0,
        session_id: str = None,
    ) -> list[Trace]:
        """List traces for a tenant."""
        if not self.db_pool:
            return []

        async with self.db_pool() as session:
            from sqlalchemy import text

            if session_id:
                result = await session.execute(
                    text(
                        """
                        SELECT trace_data FROM ai_traces
                        WHERE tenant_id = :tenant_id AND session_id = :session_id
                        ORDER BY created_at DESC
                        LIMIT :limit OFFSET :offset
                        """
                    ),
                    {
                        "tenant_id": tenant_id,
                        "session_id": session_id,
                        "limit": limit,
                        "offset": offset,
                    },
                )
            else:
                result = await session.execute(
                    text(
                        """
                        SELECT trace_data FROM ai_traces
                        WHERE tenant_id = :tenant_id
                        ORDER BY created_at DESC
                        LIMIT :limit OFFSET :offset
                        """
                    ),
                    {"tenant_id": tenant_id, "limit": limit, "offset": offset},
                )

            rows = result.fetchall()
            traces = []
            for row in rows:
                data = row[0] if isinstance(row[0], dict) else json.loads(row[0])
                traces.append(Trace.from_dict(data))
            return traces

    async def get_cost_summary(
        self,
        tenant_id: str,
        start_date: str = None,
        end_date: str = None,
    ) -> dict:
        """Get cost summary: total_cost, by_model, by_operation, token_counts."""
        if not self.db_pool:
            return {
                "total_cost": 0.0,
                "by_model": {},
                "by_operation": {},
                "token_counts": {"input": 0, "output": 0, "total": 0},
                "trace_count": 0,
            }

        async with self.db_pool() as session:
            from sqlalchemy import text

            params: dict = {"tenant_id": tenant_id}
            date_filter = ""

            if start_date:
                date_filter += " AND created_at >= :start_date"
                params["start_date"] = start_date
            if end_date:
                date_filter += " AND created_at <= :end_date"
                params["end_date"] = end_date

            result = await session.execute(
                text(
                    f"""
                    SELECT trace_data FROM ai_traces
                    WHERE tenant_id = :tenant_id{date_filter}
                    ORDER BY created_at DESC
                    """
                ),
                params,
            )

            rows = result.fetchall()

            total_cost = 0.0
            by_model: dict[str, float] = {}
            by_operation: dict[str, float] = {}
            total_input_tokens = 0
            total_output_tokens = 0
            total_tokens = 0

            for row in rows:
                data = row[0] if isinstance(row[0], dict) else json.loads(row[0])
                trace_cost = data.get("total_cost_usd", 0.0)
                total_cost += trace_cost

                for span in data.get("spans", []):
                    model = span.get("model")
                    span_cost = span.get("cost_usd", 0.0) or 0.0
                    span_type = span.get("span_type", "unknown")

                    if model:
                        by_model[model] = by_model.get(model, 0.0) + span_cost
                    by_operation[span_type] = by_operation.get(span_type, 0.0) + span_cost

                    total_input_tokens += span.get("input_tokens", 0)
                    total_output_tokens += span.get("output_tokens", 0)
                    total_tokens += span.get("total_tokens", 0)

            return {
                "total_cost": round(total_cost, 6),
                "by_model": {k: round(v, 6) for k, v in by_model.items()},
                "by_operation": {k: round(v, 6) for k, v in by_operation.items()},
                "token_counts": {
                    "input": total_input_tokens,
                    "output": total_output_tokens,
                    "total": total_tokens,
                },
                "trace_count": len(rows),
            }

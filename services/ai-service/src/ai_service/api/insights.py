"""AI-powered insights and explanations endpoints."""

from typing import Any
from uuid import UUID

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field
import structlog

from ai_service.llm import get_llm_client, PromptManager

router = APIRouter()
logger = structlog.get_logger()


class ErrorExplanationRequest(BaseModel):
    """Error explanation request."""

    tenant_id: UUID
    error_type: str
    error_message: str
    context: str = ""
    timestamp: str = ""
    resource_type: str = ""
    resource_name: str = ""


class ErrorExplanationResponse(BaseModel):
    """Error explanation response."""

    explanation: str
    likely_causes: list[str] = Field(default_factory=list)
    recommended_actions: list[str] = Field(default_factory=list)
    prevention_tips: list[str] = Field(default_factory=list)


@router.post("/explain-error", response_model=ErrorExplanationResponse)
async def explain_error(request: ErrorExplanationRequest) -> ErrorExplanationResponse:
    """Get an AI explanation for an error."""
    logger.info(
        "Explaining error",
        tenant_id=str(request.tenant_id),
        error_type=request.error_type,
    )

    try:
        client = get_llm_client()

        prompt = PromptManager.format_prompt(
            "error_explanation",
            error_type=request.error_type,
            error_message=request.error_message,
            context=request.context or "Not provided",
            timestamp=request.timestamp or "Not provided",
            resource_type=request.resource_type or "Unknown",
            resource_name=request.resource_name or "Unknown",
        )

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="general",
        )

        response = await client.generate(messages)

        return ErrorExplanationResponse(
            explanation=response,
            likely_causes=[],  # Would be extracted from response
            recommended_actions=[],
            prevention_tips=[],
        )

    except Exception as e:
        logger.error("Error explanation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class DocumentationRequest(BaseModel):
    """Documentation generation request."""

    tenant_id: UUID
    resource_type: str
    resource_data: dict[str, Any]
    doc_type: str = "overview"  # overview, technical, runbook


class DocumentationResponse(BaseModel):
    """Documentation generation response."""

    title: str
    content: str
    doc_type: str


@router.post("/generate-docs", response_model=DocumentationResponse)
async def generate_documentation(request: DocumentationRequest) -> DocumentationResponse:
    """Generate documentation for a resource."""
    logger.info(
        "Generating documentation",
        tenant_id=str(request.tenant_id),
        resource_type=request.resource_type,
        doc_type=request.doc_type,
    )

    try:
        client = get_llm_client()

        # Build prompt based on resource type and doc type
        prompt = f"""Generate {request.doc_type} documentation for the following {request.resource_type}:

{_format_resource_data(request.resource_data)}

Generate well-structured markdown documentation that includes:
- Overview/Description
- Key details and specifications
- Usage information (if applicable)
- Related resources or dependencies
- Notes and considerations"""

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="documentation",
        )

        content = await client.generate(messages)

        # Extract title from content or generate one
        title = request.resource_data.get("name", f"{request.resource_type} Documentation")

        return DocumentationResponse(
            title=title,
            content=content,
            doc_type=request.doc_type,
        )

    except Exception as e:
        logger.error("Documentation generation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class SummarizeRequest(BaseModel):
    """Summarization request."""

    tenant_id: UUID
    content: str = Field(..., max_length=50000)
    summary_type: str = "brief"  # brief, detailed, bullets
    max_length: int = 500


class SummarizeResponse(BaseModel):
    """Summarization response."""

    summary: str
    summary_type: str
    original_length: int


@router.post("/summarize", response_model=SummarizeResponse)
async def summarize_content(request: SummarizeRequest) -> SummarizeResponse:
    """Summarize content using AI."""
    logger.info(
        "Summarizing content",
        tenant_id=str(request.tenant_id),
        summary_type=request.summary_type,
        content_length=len(request.content),
    )

    try:
        client = get_llm_client()

        style_instructions = {
            "brief": "Provide a concise 2-3 sentence summary capturing the key points.",
            "detailed": "Provide a comprehensive summary covering all important details.",
            "bullets": "Provide a bullet-point summary with key takeaways.",
        }

        prompt = f"""{style_instructions.get(request.summary_type, style_instructions['brief'])}

Content to summarize:
{request.content[:10000]}  # Limit input length

Keep the summary under {request.max_length} characters."""

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="general",
        )

        summary = await client.generate(messages)

        return SummarizeResponse(
            summary=summary[:request.max_length],
            summary_type=request.summary_type,
            original_length=len(request.content),
        )

    except Exception as e:
        logger.error("Summarization failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class AnalyzeMetricsRequest(BaseModel):
    """Metrics analysis request."""

    tenant_id: UUID
    metrics: list[dict[str, Any]]
    time_range: str = "24h"
    analysis_focus: str = "anomalies"  # anomalies, trends, comparison


class AnalyzeMetricsResponse(BaseModel):
    """Metrics analysis response."""

    analysis: str
    anomalies: list[dict[str, Any]] = Field(default_factory=list)
    recommendations: list[str] = Field(default_factory=list)


@router.post("/analyze-metrics", response_model=AnalyzeMetricsResponse)
async def analyze_metrics(request: AnalyzeMetricsRequest) -> AnalyzeMetricsResponse:
    """Analyze metrics data with AI."""
    logger.info(
        "Analyzing metrics",
        tenant_id=str(request.tenant_id),
        metric_count=len(request.metrics),
        focus=request.analysis_focus,
    )

    try:
        client = get_llm_client()

        metrics_text = "\n".join([
            f"- {m.get('name', 'Unknown')}: {m.get('value', 'N/A')} (previous: {m.get('previous', 'N/A')})"
            for m in request.metrics[:20]  # Limit to 20 metrics
        ])

        prompt = f"""Analyze the following system metrics for the past {request.time_range}:

{metrics_text}

Focus on: {request.analysis_focus}

Provide:
1. Overall assessment
2. Notable patterns or anomalies
3. Potential concerns
4. Recommended actions if any"""

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="general",
        )

        analysis = await client.generate(messages)

        return AnalyzeMetricsResponse(
            analysis=analysis,
        )

    except Exception as e:
        logger.error("Metrics analysis failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


def _format_resource_data(data: dict[str, Any], indent: int = 0) -> str:
    """Format resource data for prompt."""
    lines = []
    prefix = "  " * indent

    for key, value in data.items():
        if isinstance(value, dict):
            lines.append(f"{prefix}{key}:")
            lines.append(_format_resource_data(value, indent + 1))
        elif isinstance(value, list):
            lines.append(f"{prefix}{key}: {', '.join(str(v) for v in value[:10])}")
        else:
            lines.append(f"{prefix}{key}: {value}")

    return "\n".join(lines)

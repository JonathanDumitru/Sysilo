"""AI-powered recommendation endpoints."""

from typing import Any, Literal
from uuid import UUID

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field
import structlog

from ai_service.llm import get_llm_client, PromptManager

router = APIRouter()
logger = structlog.get_logger()


class ApplicationContext(BaseModel):
    """Application context for recommendations."""

    id: UUID
    name: str
    application_type: str | None = None
    criticality: str = "operational"
    lifecycle_stage: str = "production"
    value_score: float = 5.0
    health_score: float = 5.0
    complexity_score: float = 5.0
    cost_score: float = 5.0
    fit_score: float = 5.0
    quadrant: str | None = None
    annual_cost: float = 0
    dependency_count: int = 0


class PortfolioContext(BaseModel):
    """Portfolio context for recommendations."""

    total_applications: int = 0
    eliminate_count: int = 0
    migrate_count: int = 0
    invest_count: int = 0
    tolerate_count: int = 0
    total_spend: float = 0


class RecommendationRequest(BaseModel):
    """Recommendation generation request."""

    tenant_id: UUID
    application: ApplicationContext | None = None
    portfolio: PortfolioContext
    focus_areas: list[str] = Field(default_factory=list)


class Recommendation(BaseModel):
    """A single recommendation."""

    type: str
    title: str
    summary: str
    detailed_analysis: str
    confidence_score: float
    estimated_savings: float | None = None
    estimated_effort: Literal["low", "medium", "high"]
    risk_assessment: Literal["low", "medium", "high"]
    reasoning: dict[str, Any] = Field(default_factory=dict)


class RecommendationResponse(BaseModel):
    """Recommendation response."""

    recommendations: list[Recommendation]
    context_summary: str


@router.post("/generate", response_model=RecommendationResponse)
async def generate_recommendations(request: RecommendationRequest) -> RecommendationResponse:
    """Generate AI-powered rationalization recommendations."""
    logger.info(
        "Generating recommendations",
        tenant_id=str(request.tenant_id),
        has_application=request.application is not None,
    )

    try:
        client = get_llm_client()

        # Build the prompt
        if request.application:
            prompt = PromptManager.format_prompt(
                "recommendation",
                application_name=request.application.name,
                application_type=request.application.application_type or "Unknown",
                criticality=request.application.criticality,
                lifecycle_stage=request.application.lifecycle_stage,
                value_score=request.application.value_score,
                health_score=request.application.health_score,
                complexity_score=request.application.complexity_score,
                cost_score=request.application.cost_score,
                fit_score=request.application.fit_score,
                quadrant=request.application.quadrant or "Unknown",
                annual_cost=request.application.annual_cost,
                dependency_count=request.application.dependency_count,
                total_applications=request.portfolio.total_applications,
                eliminate_count=request.portfolio.eliminate_count,
                total_spend=request.portfolio.total_spend,
            )
        else:
            # Portfolio-level recommendations
            prompt = f"""Analyze the following application portfolio and provide strategic recommendations:

Portfolio Summary:
- Total Applications: {request.portfolio.total_applications}
- Applications to Eliminate: {request.portfolio.eliminate_count}
- Applications to Migrate: {request.portfolio.migrate_count}
- Applications to Invest: {request.portfolio.invest_count}
- Applications to Tolerate: {request.portfolio.tolerate_count}
- Total Annual IT Spend: ${request.portfolio.total_spend:,.0f}

Provide 3-5 strategic recommendations for portfolio optimization with:
1. Action type
2. Clear rationale
3. Estimated impact/savings
4. Effort level
5. Risk assessment"""

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="rationalization",
        )

        response = await client.generate(messages)

        # Parse the response into structured recommendations
        recommendations = _parse_recommendations(response)

        return RecommendationResponse(
            recommendations=recommendations,
            context_summary=f"Analysis based on {'application ' + request.application.name if request.application else 'portfolio'} context",
        )

    except Exception as e:
        logger.error("Recommendation generation failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class ScenarioAnalysisRequest(BaseModel):
    """Scenario analysis request."""

    tenant_id: UUID
    scenario_name: str
    scenario_description: str
    applications: list[dict[str, Any]]


class ScenarioAnalysisResponse(BaseModel):
    """Scenario analysis response."""

    scenario_name: str
    analysis: str
    estimated_cost: float | None = None
    estimated_savings: float | None = None
    payback_months: int | None = None
    roi_percent: float | None = None
    risk_level: str = "medium"
    key_risks: list[str] = Field(default_factory=list)


@router.post("/analyze-scenario", response_model=ScenarioAnalysisResponse)
async def analyze_scenario(request: ScenarioAnalysisRequest) -> ScenarioAnalysisResponse:
    """Analyze a rationalization scenario with AI."""
    logger.info(
        "Analyzing scenario",
        tenant_id=str(request.tenant_id),
        scenario_name=request.scenario_name,
        application_count=len(request.applications),
    )

    try:
        client = get_llm_client()

        # Build applications list
        apps_text = "\n".join([
            f"- {app.get('name', 'Unknown')} ({app.get('action', 'unknown')}): ${app.get('cost', 0):,.0f}"
            for app in request.applications
        ])

        prompt = PromptManager.format_prompt(
            "scenario_analysis",
            scenario_name=request.scenario_name,
            scenario_description=request.scenario_description,
            applications_list=apps_text,
        )

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="rationalization",
        )

        analysis = await client.generate(messages)

        return ScenarioAnalysisResponse(
            scenario_name=request.scenario_name,
            analysis=analysis,
            risk_level="medium",  # Would be extracted from analysis
        )

    except Exception as e:
        logger.error("Scenario analysis failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


class ImpactAnalysisRequest(BaseModel):
    """Impact analysis request."""

    tenant_id: UUID
    application_name: str
    change_type: str
    dependents: list[str] = Field(default_factory=list)
    upstream: list[str] = Field(default_factory=list)


class ImpactAnalysisResponse(BaseModel):
    """Impact analysis response."""

    application_name: str
    change_type: str
    analysis: str
    risk_level: str = "medium"
    affected_systems: list[str] = Field(default_factory=list)


@router.post("/impact-analysis", response_model=ImpactAnalysisResponse)
async def analyze_impact(request: ImpactAnalysisRequest) -> ImpactAnalysisResponse:
    """Analyze the impact of application changes."""
    logger.info(
        "Analyzing impact",
        tenant_id=str(request.tenant_id),
        application=request.application_name,
        change_type=request.change_type,
    )

    try:
        client = get_llm_client()

        prompt = PromptManager.format_prompt(
            "impact_analysis",
            application_name=request.application_name,
            change_type=request.change_type,
            dependents_list="\n".join([f"- {d}" for d in request.dependents]) or "None",
            upstream_list="\n".join([f"- {u}" for u in request.upstream]) or "None",
        )

        messages = PromptManager.build_messages(
            user_message=prompt,
            context="rationalization",
        )

        analysis = await client.generate(messages)

        return ImpactAnalysisResponse(
            application_name=request.application_name,
            change_type=request.change_type,
            analysis=analysis,
            affected_systems=request.dependents + request.upstream,
        )

    except Exception as e:
        logger.error("Impact analysis failed", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


def _parse_recommendations(text: str) -> list[Recommendation]:
    """Parse LLM response into structured recommendations."""
    # Simple parsing - in production, use more sophisticated extraction
    # or request structured JSON output from the LLM
    recommendations = []

    # Split by numbered items or recommendation headers
    sections = text.split("\n\n")

    for i, section in enumerate(sections[:5]):  # Limit to 5 recommendations
        if not section.strip():
            continue

        # Extract title from first line
        lines = section.strip().split("\n")
        title = lines[0].strip("# -*1234567890. ")

        if len(title) < 5:
            continue

        recommendations.append(
            Recommendation(
                type="optimization",  # Would be extracted properly
                title=title[:100],
                summary=section[:200],
                detailed_analysis=section,
                confidence_score=0.75,
                estimated_savings=None,
                estimated_effort="medium",
                risk_assessment="medium",
            )
        )

    return recommendations or [
        Recommendation(
            type="analysis",
            title="Portfolio Analysis Complete",
            summary=text[:200],
            detailed_analysis=text,
            confidence_score=0.7,
            estimated_effort="medium",
            risk_assessment="low",
        )
    ]

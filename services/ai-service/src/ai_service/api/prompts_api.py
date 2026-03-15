"""Prompt versioning API endpoints."""

from typing import Optional

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field
import structlog

from ai_service.db import async_session
from ai_service.prompts.versioning import PromptRegistry

router = APIRouter()
logger = structlog.get_logger()

# Shared registry instance
_registry: Optional[PromptRegistry] = None


def get_registry() -> PromptRegistry:
    """Get or create the shared prompt registry instance."""
    global _registry
    if _registry is None:
        _registry = PromptRegistry(db_pool=async_session)
    return _registry


# ---- Request/Response models ----

class CreatePromptRequest(BaseModel):
    tenant_id: str
    name: str = Field(..., min_length=1, max_length=255)
    template: str = Field(..., min_length=1)
    variables: list[str] = Field(default_factory=list)
    system_prompt: Optional[str] = None
    model_config: dict = Field(default_factory=dict)
    created_by: Optional[str] = None
    metadata: dict = Field(default_factory=dict)


class PromptVersionResponse(BaseModel):
    id: str
    tenant_id: str
    prompt_name: str
    version: int
    template: str
    variables: list[str]
    system_prompt: Optional[str] = None
    model_config: dict = Field(default_factory=dict)
    is_active: bool
    created_by: Optional[str] = None
    created_at: Optional[str] = None
    metadata: dict = Field(default_factory=dict)


class PromptListResponse(BaseModel):
    prompts: list[PromptVersionResponse]
    total: int


class RenderPromptRequest(BaseModel):
    tenant_id: str
    variables: dict = Field(default_factory=dict)


class RenderPromptResponse(BaseModel):
    rendered_template: str
    system_prompt: Optional[str] = None
    model_config: dict = Field(default_factory=dict)


# ---- Endpoints ----

@router.get("", response_model=PromptListResponse)
async def list_prompts(
    tenant_id: str = Query(..., description="Tenant ID"),
) -> PromptListResponse:
    """List all prompts (latest active versions)."""
    logger.info("Listing prompts", tenant_id=tenant_id)

    try:
        registry = get_registry()
        prompts = await registry.list_prompts(tenant_id)

        prompt_responses = []
        for p in prompts:
            d = p.to_dict()
            prompt_responses.append(PromptVersionResponse(**d))

        return PromptListResponse(
            prompts=prompt_responses,
            total=len(prompt_responses),
        )
    except Exception as e:
        logger.error("Failed to list prompts", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.post("", response_model=PromptVersionResponse)
async def create_prompt(request: CreatePromptRequest) -> PromptVersionResponse:
    """Create a new prompt or a new version of an existing prompt."""
    logger.info(
        "Creating prompt",
        tenant_id=request.tenant_id,
        name=request.name,
    )

    try:
        registry = get_registry()
        prompt = await registry.create_prompt(
            tenant_id=request.tenant_id,
            name=request.name,
            template=request.template,
            variables=request.variables,
            system_prompt=request.system_prompt,
            model_config=request.model_config,
            created_by=request.created_by,
            metadata=request.metadata,
        )

        return PromptVersionResponse(**prompt.to_dict())
    except Exception as e:
        logger.error("Failed to create prompt", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/{name}", response_model=PromptVersionResponse)
async def get_active_prompt(
    name: str,
    tenant_id: str = Query(..., description="Tenant ID"),
) -> PromptVersionResponse:
    """Get the currently active version of a prompt."""
    logger.info("Getting active prompt", tenant_id=tenant_id, name=name)

    try:
        registry = get_registry()
        prompt = await registry.get_active_prompt(tenant_id, name)

        if not prompt:
            raise HTTPException(
                status_code=404,
                detail=f"No active prompt found for '{name}'",
            )

        return PromptVersionResponse(**prompt.to_dict())
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to get prompt", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/{name}/versions", response_model=PromptListResponse)
async def list_versions(
    name: str,
    tenant_id: str = Query(..., description="Tenant ID"),
) -> PromptListResponse:
    """List all versions of a prompt."""
    logger.info("Listing prompt versions", tenant_id=tenant_id, name=name)

    try:
        registry = get_registry()
        versions = await registry.list_versions(tenant_id, name)

        version_responses = []
        for v in versions:
            version_responses.append(PromptVersionResponse(**v.to_dict()))

        return PromptListResponse(
            prompts=version_responses,
            total=len(version_responses),
        )
    except Exception as e:
        logger.error("Failed to list versions", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/{name}/activate/{version}", response_model=PromptVersionResponse)
async def activate_version(
    name: str,
    version: int,
    tenant_id: str = Query(..., description="Tenant ID"),
) -> PromptVersionResponse:
    """Activate a specific version of a prompt (rollback support)."""
    logger.info(
        "Activating prompt version",
        tenant_id=tenant_id,
        name=name,
        version=version,
    )

    try:
        registry = get_registry()
        prompt = await registry.activate_version(tenant_id, name, version)

        if not prompt:
            raise HTTPException(
                status_code=404,
                detail=f"Prompt version {name} v{version} not found",
            )

        return PromptVersionResponse(**prompt.to_dict())
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to activate version", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/{name}/render", response_model=RenderPromptResponse)
async def render_prompt(
    name: str,
    request: RenderPromptRequest,
) -> RenderPromptResponse:
    """Render a prompt template with variables."""
    logger.info(
        "Rendering prompt",
        tenant_id=request.tenant_id,
        name=name,
        variable_count=len(request.variables),
    )

    try:
        registry = get_registry()
        rendered, system_prompt, model_config = await registry.render_prompt(
            tenant_id=request.tenant_id,
            name=name,
            variables=request.variables,
        )

        return RenderPromptResponse(
            rendered_template=rendered,
            system_prompt=system_prompt,
            model_config=model_config,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.error("Failed to render prompt", error=str(e))
        raise HTTPException(status_code=500, detail=str(e))

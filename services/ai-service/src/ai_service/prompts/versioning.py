"""Version-controlled prompt template management."""

import json
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime
from string import Template
from typing import Optional

import structlog

logger = structlog.get_logger()


@dataclass
class PromptVersion:
    id: str
    tenant_id: str
    prompt_name: str
    version: int
    template: str
    variables: list[str]  # Expected variable names
    system_prompt: Optional[str]
    model_config: dict  # temperature, max_tokens, etc.
    is_active: bool
    created_by: Optional[str]
    created_at: datetime
    metadata: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        d = asdict(self)
        d["created_at"] = self.created_at.isoformat() if self.created_at else None
        return d

    @classmethod
    def from_dict(cls, data: dict) -> "PromptVersion":
        if data.get("created_at") and isinstance(data["created_at"], str):
            data["created_at"] = datetime.fromisoformat(data["created_at"])
        return cls(**data)


class PromptRegistry:
    """Version-controlled prompt template management."""

    def __init__(self, db_pool):
        self.db_pool = db_pool
        self._cache: dict[str, PromptVersion] = {}

    def _cache_key(self, tenant_id: str, name: str) -> str:
        return f"{tenant_id}:{name}"

    async def create_prompt(
        self,
        tenant_id: str,
        name: str,
        template: str,
        variables: list[str],
        system_prompt: Optional[str],
        model_config: dict,
        created_by: Optional[str] = None,
        metadata: dict = None,
    ) -> PromptVersion:
        """Create a new prompt (version 1) or new version of existing."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            # Determine the next version number
            result = await session.execute(
                text(
                    """
                    SELECT COALESCE(MAX(version), 0) as max_version
                    FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND prompt_name = :name
                    """
                ),
                {"tenant_id": tenant_id, "name": name},
            )
            row = result.fetchone()
            next_version = (row[0] if row else 0) + 1

            # Deactivate all previous versions of this prompt
            await session.execute(
                text(
                    """
                    UPDATE ai_prompt_versions
                    SET is_active = false
                    WHERE tenant_id = :tenant_id AND prompt_name = :name
                    """
                ),
                {"tenant_id": tenant_id, "name": name},
            )

            prompt_id = str(uuid.uuid4())
            now = datetime.utcnow()

            prompt_version = PromptVersion(
                id=prompt_id,
                tenant_id=tenant_id,
                prompt_name=name,
                version=next_version,
                template=template,
                variables=variables,
                system_prompt=system_prompt,
                model_config=model_config,
                is_active=True,
                created_by=created_by,
                created_at=now,
                metadata=metadata or {},
            )

            await session.execute(
                text(
                    """
                    INSERT INTO ai_prompt_versions
                    (id, tenant_id, prompt_name, version, template, variables,
                     system_prompt, model_config, is_active, created_by, created_at, metadata)
                    VALUES (:id, :tenant_id, :prompt_name, :version, :template, :variables,
                            :system_prompt, :model_config, :is_active, :created_by, :created_at, :metadata)
                    """
                ),
                {
                    "id": prompt_id,
                    "tenant_id": tenant_id,
                    "prompt_name": name,
                    "version": next_version,
                    "template": template,
                    "variables": json.dumps(variables),
                    "system_prompt": system_prompt,
                    "model_config": json.dumps(model_config),
                    "is_active": True,
                    "created_by": created_by,
                    "created_at": now,
                    "metadata": json.dumps(metadata or {}),
                },
            )
            await session.commit()

            # Update cache
            cache_key = self._cache_key(tenant_id, name)
            self._cache[cache_key] = prompt_version

            logger.info(
                "Prompt version created",
                tenant_id=tenant_id,
                prompt_name=name,
                version=next_version,
            )

            return prompt_version

    async def get_active_prompt(
        self, tenant_id: str, name: str
    ) -> Optional[PromptVersion]:
        """Get the currently active version of a prompt."""
        # Check cache first
        cache_key = self._cache_key(tenant_id, name)
        if cache_key in self._cache:
            return self._cache[cache_key]

        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text(
                    """
                    SELECT id, tenant_id, prompt_name, version, template, variables,
                           system_prompt, model_config, is_active, created_by, created_at, metadata
                    FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND prompt_name = :name AND is_active = true
                    ORDER BY version DESC
                    LIMIT 1
                    """
                ),
                {"tenant_id": tenant_id, "name": name},
            )
            row = result.fetchone()
            if not row:
                return None

            prompt = _row_to_prompt_version(row)
            self._cache[cache_key] = prompt
            return prompt

    async def get_prompt_version(
        self, tenant_id: str, name: str, version: int
    ) -> Optional[PromptVersion]:
        """Get a specific version."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text(
                    """
                    SELECT id, tenant_id, prompt_name, version, template, variables,
                           system_prompt, model_config, is_active, created_by, created_at, metadata
                    FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND prompt_name = :name AND version = :version
                    """
                ),
                {"tenant_id": tenant_id, "name": name, "version": version},
            )
            row = result.fetchone()
            if not row:
                return None

            return _row_to_prompt_version(row)

    async def list_prompts(self, tenant_id: str) -> list[PromptVersion]:
        """List all prompts (latest active versions)."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text(
                    """
                    SELECT DISTINCT ON (prompt_name)
                           id, tenant_id, prompt_name, version, template, variables,
                           system_prompt, model_config, is_active, created_by, created_at, metadata
                    FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND is_active = true
                    ORDER BY prompt_name, version DESC
                    """
                ),
                {"tenant_id": tenant_id},
            )
            rows = result.fetchall()
            return [_row_to_prompt_version(row) for row in rows]

    async def list_versions(
        self, tenant_id: str, name: str
    ) -> list[PromptVersion]:
        """List all versions of a prompt."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text(
                    """
                    SELECT id, tenant_id, prompt_name, version, template, variables,
                           system_prompt, model_config, is_active, created_by, created_at, metadata
                    FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND prompt_name = :name
                    ORDER BY version DESC
                    """
                ),
                {"tenant_id": tenant_id, "name": name},
            )
            rows = result.fetchall()
            return [_row_to_prompt_version(row) for row in rows]

    async def activate_version(
        self, tenant_id: str, name: str, version: int
    ) -> PromptVersion:
        """Set a specific version as active (rollback support)."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            # Verify the target version exists
            result = await session.execute(
                text(
                    """
                    SELECT id FROM ai_prompt_versions
                    WHERE tenant_id = :tenant_id AND prompt_name = :name AND version = :version
                    """
                ),
                {"tenant_id": tenant_id, "name": name, "version": version},
            )
            if not result.fetchone():
                raise ValueError(
                    f"Prompt version {name} v{version} not found for tenant {tenant_id}"
                )

            # Deactivate all versions
            await session.execute(
                text(
                    """
                    UPDATE ai_prompt_versions
                    SET is_active = false
                    WHERE tenant_id = :tenant_id AND prompt_name = :name
                    """
                ),
                {"tenant_id": tenant_id, "name": name},
            )

            # Activate the target version
            await session.execute(
                text(
                    """
                    UPDATE ai_prompt_versions
                    SET is_active = true
                    WHERE tenant_id = :tenant_id AND prompt_name = :name AND version = :version
                    """
                ),
                {"tenant_id": tenant_id, "name": name, "version": version},
            )
            await session.commit()

            # Invalidate cache
            cache_key = self._cache_key(tenant_id, name)
            self._cache.pop(cache_key, None)

            logger.info(
                "Prompt version activated",
                tenant_id=tenant_id,
                prompt_name=name,
                version=version,
            )

            # Return the activated version
            prompt = await self.get_prompt_version(tenant_id, name, version)
            if prompt:
                prompt.is_active = True
                self._cache[cache_key] = prompt
            return prompt

    async def render_prompt(
        self, tenant_id: str, name: str, variables: dict
    ) -> tuple[str, Optional[str], dict]:
        """Render a prompt template with variables.

        Returns (rendered_template, system_prompt, model_config).
        Uses Python string.Template for variable substitution.
        """
        prompt = await self.get_active_prompt(tenant_id, name)
        if not prompt:
            raise ValueError(
                f"No active prompt found for '{name}' in tenant {tenant_id}"
            )

        # Validate that all expected variables are provided
        missing = [v for v in prompt.variables if v not in variables]
        if missing:
            raise ValueError(
                f"Missing required variables for prompt '{name}': {missing}"
            )

        # Render with string.Template (uses $variable syntax)
        tmpl = Template(prompt.template)
        rendered = tmpl.safe_substitute(variables)

        # Also render system prompt if it has variables
        rendered_system = None
        if prompt.system_prompt:
            sys_tmpl = Template(prompt.system_prompt)
            rendered_system = sys_tmpl.safe_substitute(variables)

        return rendered, rendered_system, prompt.model_config


def _row_to_prompt_version(row) -> PromptVersion:
    """Convert a database row to a PromptVersion object."""
    variables = row[5]
    if isinstance(variables, str):
        variables = json.loads(variables)

    model_config = row[7]
    if isinstance(model_config, str):
        model_config = json.loads(model_config)

    metadata = row[11]
    if isinstance(metadata, str):
        metadata = json.loads(metadata)

    created_at = row[10]
    if isinstance(created_at, str):
        created_at = datetime.fromisoformat(created_at)

    return PromptVersion(
        id=row[0],
        tenant_id=row[1],
        prompt_name=row[2],
        version=row[3],
        template=row[4],
        variables=variables,
        system_prompt=row[6],
        model_config=model_config,
        is_active=row[8],
        created_by=row[9],
        created_at=created_at,
        metadata=metadata or {},
    )

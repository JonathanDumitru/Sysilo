"""Business glossary for canonical term definitions.

Provides a shared vocabulary that drift detection uses to match fields
against approved business terms, reducing false positives and aligning
definitions across connected systems.
"""

import json
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime
from typing import Optional

import structlog

logger = structlog.get_logger()


@dataclass
class GlossaryTerm:
    """A canonical business term with its approved definition."""

    id: str
    tenant_id: str
    term: str  # e.g. "revenue", "customer"
    canonical_definition: str
    synonyms: list[str]
    domain: str  # e.g. "finance", "sales", "operations"
    owner: Optional[str]
    approved: bool
    created_at: datetime
    updated_at: datetime

    def to_dict(self) -> dict:
        d = asdict(self)
        d["created_at"] = self.created_at.isoformat() if self.created_at else None
        d["updated_at"] = self.updated_at.isoformat() if self.updated_at else None
        return d

    @classmethod
    def from_dict(cls, data: dict) -> "GlossaryTerm":
        for ts_field in ("created_at", "updated_at"):
            if data.get(ts_field) and isinstance(data[ts_field], str):
                data[ts_field] = datetime.fromisoformat(data[ts_field])
        if isinstance(data.get("synonyms"), str):
            data["synonyms"] = json.loads(data["synonyms"])
        return cls(**data)


class BusinessGlossary:
    """Manages the tenant-scoped business glossary.

    Persists to the ``ai_glossary_terms`` PostgreSQL table.
    """

    def __init__(self, db_pool, llm_client=None):
        self.db_pool = db_pool
        self.llm = llm_client

    # ------------------------------------------------------------------
    # Schema
    # ------------------------------------------------------------------

    async def initialize(self) -> None:
        """Create the glossary table if it does not exist."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(text("""
                CREATE TABLE IF NOT EXISTS ai_glossary_terms (
                    id                   TEXT PRIMARY KEY,
                    tenant_id            TEXT NOT NULL,
                    term                 TEXT NOT NULL,
                    canonical_definition TEXT NOT NULL DEFAULT '',
                    synonyms             JSONB NOT NULL DEFAULT '[]'::jsonb,
                    domain               TEXT NOT NULL DEFAULT '',
                    owner                TEXT,
                    approved             BOOLEAN NOT NULL DEFAULT false,
                    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    UNIQUE (tenant_id, term)
                )
            """))

            await session.execute(text("""
                CREATE INDEX IF NOT EXISTS idx_glossary_tenant
                    ON ai_glossary_terms (tenant_id)
            """))
            await session.execute(text("""
                CREATE INDEX IF NOT EXISTS idx_glossary_domain
                    ON ai_glossary_terms (tenant_id, domain)
            """))

            await session.commit()
            logger.info("Glossary table initialised")

    # ------------------------------------------------------------------
    # CRUD
    # ------------------------------------------------------------------

    async def create_term(self, tenant_id: str, term_data: dict) -> GlossaryTerm:
        """Create a new glossary term."""
        term_id = str(uuid.uuid4())
        now = datetime.utcnow()

        term = GlossaryTerm(
            id=term_id,
            tenant_id=tenant_id,
            term=term_data.get("term", ""),
            canonical_definition=term_data.get("canonical_definition", ""),
            synonyms=term_data.get("synonyms", []),
            domain=term_data.get("domain", ""),
            owner=term_data.get("owner"),
            approved=term_data.get("approved", False),
            created_at=now,
            updated_at=now,
        )

        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(
                text("""
                    INSERT INTO ai_glossary_terms
                        (id, tenant_id, term, canonical_definition, synonyms,
                         domain, owner, approved, created_at, updated_at)
                    VALUES
                        (:id, :tenant_id, :term, :canonical_definition, :synonyms,
                         :domain, :owner, :approved, :created_at, :updated_at)
                """),
                {
                    "id": term.id,
                    "tenant_id": term.tenant_id,
                    "term": term.term,
                    "canonical_definition": term.canonical_definition,
                    "synonyms": json.dumps(term.synonyms),
                    "domain": term.domain,
                    "owner": term.owner,
                    "approved": term.approved,
                    "created_at": term.created_at,
                    "updated_at": term.updated_at,
                },
            )
            await session.commit()

        logger.info(
            "Glossary term created",
            term_id=term_id,
            tenant_id=tenant_id,
            term=term.term,
        )
        return term

    async def get_term(self, term_id: str) -> Optional[GlossaryTerm]:
        """Get a single glossary term by ID."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text("""
                    SELECT id, tenant_id, term, canonical_definition, synonyms,
                           domain, owner, approved, created_at, updated_at
                    FROM ai_glossary_terms
                    WHERE id = :term_id
                """),
                {"term_id": term_id},
            )
            row = result.fetchone()
            if not row:
                return None
            return _row_to_glossary_term(row)

    async def list_terms(
        self, tenant_id: str, domain: Optional[str] = None
    ) -> list[GlossaryTerm]:
        """List glossary terms for a tenant, optionally filtered by domain."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            if domain:
                result = await session.execute(
                    text("""
                        SELECT id, tenant_id, term, canonical_definition, synonyms,
                               domain, owner, approved, created_at, updated_at
                        FROM ai_glossary_terms
                        WHERE tenant_id = :tenant_id AND domain = :domain
                        ORDER BY term ASC
                    """),
                    {"tenant_id": tenant_id, "domain": domain},
                )
            else:
                result = await session.execute(
                    text("""
                        SELECT id, tenant_id, term, canonical_definition, synonyms,
                               domain, owner, approved, created_at, updated_at
                        FROM ai_glossary_terms
                        WHERE tenant_id = :tenant_id
                        ORDER BY term ASC
                    """),
                    {"tenant_id": tenant_id},
                )

            rows = result.fetchall()
            return [_row_to_glossary_term(row) for row in rows]

    async def update_term(self, term_id: str, updates: dict) -> GlossaryTerm:
        """Update an existing glossary term.  Only provided fields are changed."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            # Fetch existing term to merge
            existing = await self.get_term(term_id)
            if not existing:
                raise ValueError(f"Glossary term {term_id} not found")

            # Apply updates
            new_term = updates.get("term", existing.term)
            new_definition = updates.get("canonical_definition", existing.canonical_definition)
            new_synonyms = updates.get("synonyms", existing.synonyms)
            new_domain = updates.get("domain", existing.domain)
            new_owner = updates.get("owner", existing.owner)
            new_approved = updates.get("approved", existing.approved)
            now = datetime.utcnow()

            await session.execute(
                text("""
                    UPDATE ai_glossary_terms
                    SET term = :term,
                        canonical_definition = :canonical_definition,
                        synonyms = :synonyms,
                        domain = :domain,
                        owner = :owner,
                        approved = :approved,
                        updated_at = :updated_at
                    WHERE id = :term_id
                """),
                {
                    "term_id": term_id,
                    "term": new_term,
                    "canonical_definition": new_definition,
                    "synonyms": json.dumps(new_synonyms),
                    "domain": new_domain,
                    "owner": new_owner,
                    "approved": new_approved,
                    "updated_at": now,
                },
            )
            await session.commit()

        logger.info("Glossary term updated", term_id=term_id)

        updated = await self.get_term(term_id)
        if not updated:
            raise ValueError(f"Glossary term {term_id} not found after update")
        return updated

    async def delete_term(self, term_id: str) -> None:
        """Delete a glossary term."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(
                text("DELETE FROM ai_glossary_terms WHERE id = :term_id"),
                {"term_id": term_id},
            )
            await session.commit()

        logger.info("Glossary term deleted", term_id=term_id)

    # ------------------------------------------------------------------
    # Search
    # ------------------------------------------------------------------

    async def search_terms(
        self, tenant_id: str, query: str
    ) -> list[GlossaryTerm]:
        """Search glossary terms by name, definition, or synonyms."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            search_pattern = f"%{query.lower()}%"

            result = await session.execute(
                text("""
                    SELECT id, tenant_id, term, canonical_definition, synonyms,
                           domain, owner, approved, created_at, updated_at
                    FROM ai_glossary_terms
                    WHERE tenant_id = :tenant_id
                      AND (
                          LOWER(term) LIKE :pattern
                          OR LOWER(canonical_definition) LIKE :pattern
                          OR synonyms::text ILIKE :pattern
                      )
                    ORDER BY term ASC
                """),
                {"tenant_id": tenant_id, "pattern": search_pattern},
            )
            rows = result.fetchall()
            return [_row_to_glossary_term(row) for row in rows]

    async def find_matching_term(
        self,
        tenant_id: str,
        field_name: str,
        description: Optional[str] = None,
    ) -> Optional[GlossaryTerm]:
        """Find the glossary term that best matches a field name.

        First tries an exact/synonym match.  If that fails and an LLM client
        is available, uses it to semantically match.
        """
        # Direct match
        async with self.db_pool() as session:
            from sqlalchemy import text

            normalised = field_name.lower().strip()

            result = await session.execute(
                text("""
                    SELECT id, tenant_id, term, canonical_definition, synonyms,
                           domain, owner, approved, created_at, updated_at
                    FROM ai_glossary_terms
                    WHERE tenant_id = :tenant_id
                      AND (
                          LOWER(term) = :name
                          OR synonyms @> :name_json::jsonb
                      )
                    LIMIT 1
                """),
                {
                    "tenant_id": tenant_id,
                    "name": normalised,
                    "name_json": json.dumps(normalised),
                },
            )
            row = result.fetchone()
            if row:
                return _row_to_glossary_term(row)

        # LLM-based matching if available
        if self.llm:
            return await self._llm_match_term(tenant_id, field_name, description)

        return None

    async def _llm_match_term(
        self,
        tenant_id: str,
        field_name: str,
        description: Optional[str],
    ) -> Optional[GlossaryTerm]:
        """Use the LLM to find the best matching glossary term."""
        all_terms = await self.list_terms(tenant_id)
        if not all_terms:
            return None

        terms_text = "\n".join(
            f"- {t.term}: {t.canonical_definition} (synonyms: {', '.join(t.synonyms)})"
            for t in all_terms
        )

        field_desc = f"Field: {field_name}"
        if description:
            field_desc += f"\nDescription: {description}"

        messages = [
            {
                "role": "system",
                "content": (
                    "You are a data governance expert. Match a field to the most "
                    "appropriate glossary term. Respond with ONLY the exact glossary "
                    "term name, or 'NONE' if no match is appropriate."
                ),
            },
            {
                "role": "user",
                "content": (
                    f"{field_desc}\n\n"
                    f"Available glossary terms:\n{terms_text}\n\n"
                    "Which glossary term best matches this field? Respond with "
                    "the exact term name or 'NONE'."
                ),
            },
        ]

        try:
            response = await self.llm.generate(
                messages=messages,
                temperature=0.0,
                max_tokens=100,
            )
            matched_name = response.strip().strip('"').strip("'").lower()

            if matched_name == "none":
                return None

            # Find the term by name
            for t in all_terms:
                if t.term.lower() == matched_name:
                    return t

        except Exception as exc:
            logger.error("LLM glossary matching failed", error=str(exc))

        return None


# ---------------------------------------------------------------------------
# Row mapper
# ---------------------------------------------------------------------------

def _row_to_glossary_term(row) -> GlossaryTerm:
    """Convert a database row to a :class:`GlossaryTerm`."""
    synonyms = row[4]
    if isinstance(synonyms, str):
        synonyms = json.loads(synonyms)
    elif synonyms is None:
        synonyms = []

    created_at = row[8]
    if isinstance(created_at, str):
        created_at = datetime.fromisoformat(created_at)

    updated_at = row[9]
    if isinstance(updated_at, str):
        updated_at = datetime.fromisoformat(updated_at)

    return GlossaryTerm(
        id=row[0],
        tenant_id=row[1],
        term=row[2],
        canonical_definition=row[3],
        synonyms=synonyms,
        domain=row[5],
        owner=row[6],
        approved=row[7],
        created_at=created_at,
        updated_at=updated_at,
    )

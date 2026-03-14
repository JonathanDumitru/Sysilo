"""Semantic drift detection engine.

Detects when the same field or concept means different things across connected
systems, scores divergence, and surfaces conflicts for resolution.
"""

import json
import re
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime, timedelta
from typing import Optional, Tuple

import structlog

logger = structlog.get_logger()

# ---------------------------------------------------------------------------
# Common irregular plurals used for singularization
# ---------------------------------------------------------------------------
_IRREGULAR_PLURALS = {
    "addresses": "address",
    "statuses": "status",
    "indices": "index",
    "indexes": "index",
    "matrices": "matrix",
    "vertices": "vertex",
    "analyses": "analysis",
    "currencies": "currency",
    "categories": "category",
    "companies": "company",
    "entities": "entity",
    "policies": "policy",
    "quantities": "quantity",
}


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------

@dataclass
class FieldDefinition:
    """How a field is defined in a specific system/dataset."""

    source_system: str
    dataset: str
    field_name: str
    data_type: str
    description: Optional[str]
    sample_values: list[str]
    business_glossary_term: Optional[str]
    last_updated: datetime

    def to_dict(self) -> dict:
        d = asdict(self)
        d["last_updated"] = self.last_updated.isoformat() if self.last_updated else None
        return d

    @classmethod
    def from_dict(cls, data: dict) -> "FieldDefinition":
        if data.get("last_updated") and isinstance(data["last_updated"], str):
            data["last_updated"] = datetime.fromisoformat(data["last_updated"])
        return cls(**data)


@dataclass
class SemanticConflict:
    """A detected semantic drift between two definitions of the same concept."""

    id: str
    concept_name: str
    definitions: list[FieldDefinition]
    drift_score: float  # 0.0 (identical) to 1.0 (completely different meaning)
    drift_type: str  # definition_mismatch | type_divergence | value_range_drift | temporal_drift
    explanation: str
    detected_at: datetime
    lineage_path: Optional[str]
    severity: str  # critical | high | medium | low
    status: str  # open | acknowledged | resolved | false_positive

    def to_dict(self) -> dict:
        d = asdict(self)
        d["definitions"] = [defn.to_dict() if isinstance(defn, FieldDefinition) else defn
                            for defn in self.definitions]
        d["detected_at"] = self.detected_at.isoformat() if self.detected_at else None
        return d

    @classmethod
    def from_dict(cls, data: dict) -> "SemanticConflict":
        if data.get("detected_at") and isinstance(data["detected_at"], str):
            data["detected_at"] = datetime.fromisoformat(data["detected_at"])
        defs = data.get("definitions", [])
        data["definitions"] = [
            FieldDefinition.from_dict(d) if isinstance(d, dict) else d for d in defs
        ]
        return cls(**data)


@dataclass
class DriftScanResult:
    """Result of a drift detection scan."""

    scan_id: str
    tenant_id: str
    scanned_at: datetime
    total_fields_scanned: int
    conflicts_found: int
    conflicts: list[SemanticConflict]
    overall_drift_score: float  # Average across all conflicts

    def to_dict(self) -> dict:
        d = asdict(self)
        d["scanned_at"] = self.scanned_at.isoformat() if self.scanned_at else None
        d["conflicts"] = [c.to_dict() if isinstance(c, SemanticConflict) else c
                          for c in self.conflicts]
        return d


# ---------------------------------------------------------------------------
# Severity thresholds
# ---------------------------------------------------------------------------

_SEVERITY_THRESHOLDS = [
    (0.8, "critical"),
    (0.6, "high"),
    (0.4, "medium"),
    (0.0, "low"),
]


# ---------------------------------------------------------------------------
# Detector
# ---------------------------------------------------------------------------

class SemanticDriftDetector:
    """Detects semantic drift across field definitions from connected systems.

    Persists results to PostgreSQL tables ``ai_drift_scans`` and
    ``ai_drift_conflicts``.  Uses an :class:`LLMClient` to compare field
    definitions semantically.
    """

    def __init__(self, db_pool, llm_client):
        self.db_pool = db_pool
        self.llm = llm_client

    # ------------------------------------------------------------------
    # Schema initialisation
    # ------------------------------------------------------------------

    async def initialize(self) -> None:
        """Create tables if they do not exist."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(text("""
                CREATE TABLE IF NOT EXISTS ai_drift_scans (
                    scan_id      TEXT PRIMARY KEY,
                    tenant_id    TEXT NOT NULL,
                    scanned_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    total_fields INTEGER NOT NULL DEFAULT 0,
                    conflicts_found INTEGER NOT NULL DEFAULT 0,
                    overall_drift_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                    scan_data    JSONB NOT NULL DEFAULT '{}'::jsonb
                )
            """))

            await session.execute(text("""
                CREATE TABLE IF NOT EXISTS ai_drift_conflicts (
                    id              TEXT PRIMARY KEY,
                    scan_id         TEXT NOT NULL,
                    tenant_id       TEXT NOT NULL,
                    concept_name    TEXT NOT NULL,
                    drift_score     DOUBLE PRECISION NOT NULL,
                    drift_type      TEXT NOT NULL,
                    explanation     TEXT NOT NULL DEFAULT '',
                    severity        TEXT NOT NULL DEFAULT 'low',
                    status          TEXT NOT NULL DEFAULT 'open',
                    lineage_path    TEXT,
                    definitions     JSONB NOT NULL DEFAULT '[]'::jsonb,
                    detected_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    resolution_note TEXT,
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
                )
            """))

            await session.execute(text("""
                CREATE INDEX IF NOT EXISTS idx_drift_conflicts_tenant
                    ON ai_drift_conflicts (tenant_id)
            """))
            await session.execute(text("""
                CREATE INDEX IF NOT EXISTS idx_drift_conflicts_status
                    ON ai_drift_conflicts (status)
            """))
            await session.execute(text("""
                CREATE INDEX IF NOT EXISTS idx_drift_scans_tenant
                    ON ai_drift_scans (tenant_id)
            """))

            await session.commit()
            logger.info("Drift detection tables initialised")

    # ------------------------------------------------------------------
    # Full scan
    # ------------------------------------------------------------------

    async def scan_for_drift(
        self, tenant_id: str, scope: Optional[str] = None
    ) -> DriftScanResult:
        """Run a full drift detection scan.

        1. Fetch all field definitions from connected systems (catalog).
        2. Group fields by normalised name.
        3. For each group with 2+ definitions, compute drift score via LLM.
        4. Persist and return all conflicts.
        """
        scan_id = str(uuid.uuid4())
        now = datetime.utcnow()

        logger.info(
            "Starting drift scan",
            scan_id=scan_id,
            tenant_id=tenant_id,
            scope=scope,
        )

        # Step 1 -- fetch field definitions
        definitions = await self._fetch_field_definitions(tenant_id, scope)
        total_fields = len(definitions)

        # Step 2 -- group by normalised name
        groups: dict[str, list[FieldDefinition]] = {}
        for defn in definitions:
            key = await self._normalize_field_name(defn.field_name)
            groups.setdefault(key, []).append(defn)

        # Step 3 -- compute drift for groups with 2+ definitions
        conflicts: list[SemanticConflict] = []
        for concept_name, defs in groups.items():
            if len(defs) < 2:
                continue

            try:
                drift_score, drift_type, explanation = await self.compute_drift_score(defs)
            except Exception as exc:
                logger.error(
                    "Failed to compute drift score",
                    concept_name=concept_name,
                    error=str(exc),
                )
                drift_score, drift_type, explanation = (
                    0.5,
                    "definition_mismatch",
                    f"Unable to compute drift score: {exc}",
                )

            # Only surface conflicts with meaningful drift
            if drift_score < 0.1:
                continue

            severity = await self._classify_severity(drift_score, len(defs))

            conflict = SemanticConflict(
                id=str(uuid.uuid4()),
                concept_name=concept_name,
                definitions=defs,
                drift_score=round(drift_score, 4),
                drift_type=drift_type,
                explanation=explanation,
                detected_at=now,
                lineage_path=None,
                severity=severity,
                status="open",
            )
            conflicts.append(conflict)

        # Compute overall drift score
        overall = (
            round(sum(c.drift_score for c in conflicts) / len(conflicts), 4)
            if conflicts
            else 0.0
        )

        result = DriftScanResult(
            scan_id=scan_id,
            tenant_id=tenant_id,
            scanned_at=now,
            total_fields_scanned=total_fields,
            conflicts_found=len(conflicts),
            conflicts=conflicts,
            overall_drift_score=overall,
        )

        # Step 4 -- persist
        await self._persist_scan(result)

        logger.info(
            "Drift scan completed",
            scan_id=scan_id,
            tenant_id=tenant_id,
            total_fields=total_fields,
            conflicts_found=len(conflicts),
            overall_drift_score=overall,
        )

        return result

    # ------------------------------------------------------------------
    # LLM drift scoring
    # ------------------------------------------------------------------

    async def compute_drift_score(
        self, definitions: list[FieldDefinition]
    ) -> Tuple[float, str, str]:
        """Use the LLM to compare field definitions semantically.

        Returns ``(drift_score, drift_type, explanation)``.
        """
        # Build the comparison prompt
        field_descriptions = []
        for i, defn in enumerate(definitions, 1):
            desc_parts = [
                f"Definition {i}:",
                f"  Source System: {defn.source_system}",
                f"  Dataset: {defn.dataset}",
                f"  Field Name: {defn.field_name}",
                f"  Data Type: {defn.data_type}",
            ]
            if defn.description:
                desc_parts.append(f"  Description: {defn.description}")
            if defn.sample_values:
                sample_str = ", ".join(defn.sample_values[:10])
                desc_parts.append(f"  Sample Values: {sample_str}")
            if defn.business_glossary_term:
                desc_parts.append(f"  Glossary Term: {defn.business_glossary_term}")
            field_descriptions.append("\n".join(desc_parts))

        definitions_text = "\n\n".join(field_descriptions)

        messages = [
            {
                "role": "system",
                "content": (
                    "You are a data governance expert that analyses field definitions "
                    "across enterprise systems to detect semantic drift — cases where "
                    "the same concept means different things in different systems.\n\n"
                    "You MUST respond with a valid JSON object and nothing else."
                ),
            },
            {
                "role": "user",
                "content": (
                    "Compare the following field definitions that share the same "
                    "normalised name and determine if they refer to the same business "
                    "concept.\n\n"
                    f"{definitions_text}\n\n"
                    "Respond with a JSON object containing:\n"
                    '- "drift_score": float between 0.0 (identical semantics) and '
                    "1.0 (completely different meanings)\n"
                    '- "drift_type": one of "definition_mismatch", "type_divergence", '
                    '"value_range_drift", "temporal_drift"\n'
                    '- "explanation": a concise explanation of the semantic difference '
                    "in business terms (2-3 sentences)\n\n"
                    "Guidelines:\n"
                    "- Score 0.0-0.2: fields are semantically identical or trivially "
                    "different (formatting, casing)\n"
                    "- Score 0.2-0.4: minor differences in scope or precision\n"
                    "- Score 0.4-0.6: moderate divergence — same general domain but "
                    "different business interpretation\n"
                    "- Score 0.6-0.8: significant divergence — could lead to data "
                    "quality issues if joined\n"
                    "- Score 0.8-1.0: completely different business concepts despite "
                    "sharing a name"
                ),
            },
        ]

        try:
            response_text = await self.llm.generate(
                messages=messages,
                temperature=0.1,
                max_tokens=500,
            )

            parsed = self._parse_llm_drift_response(response_text)
            return (
                parsed["drift_score"],
                parsed["drift_type"],
                parsed["explanation"],
            )
        except Exception as exc:
            logger.error("LLM drift scoring failed", error=str(exc))
            # Fallback: heuristic comparison
            return self._heuristic_drift_score(definitions)

    # ------------------------------------------------------------------
    # Conflict queries
    # ------------------------------------------------------------------

    async def get_conflicts(
        self,
        tenant_id: str,
        status: Optional[str] = None,
        min_severity: Optional[str] = None,
    ) -> list[SemanticConflict]:
        """List all detected conflicts with optional filters."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            query = """
                SELECT id, concept_name, definitions, drift_score, drift_type,
                       explanation, detected_at, lineage_path, severity, status
                FROM ai_drift_conflicts
                WHERE tenant_id = :tenant_id
            """
            params: dict = {"tenant_id": tenant_id}

            if status:
                query += " AND status = :status"
                params["status"] = status

            if min_severity:
                severity_order = {"critical": 0, "high": 1, "medium": 2, "low": 3}
                threshold = severity_order.get(min_severity, 3)
                allowed = [s for s, v in severity_order.items() if v <= threshold]
                placeholders = ", ".join(f":sev_{i}" for i in range(len(allowed)))
                query += f" AND severity IN ({placeholders})"
                for i, s in enumerate(allowed):
                    params[f"sev_{i}"] = s

            query += " ORDER BY drift_score DESC, detected_at DESC"

            result = await session.execute(text(query), params)
            rows = result.fetchall()
            return [self._row_to_conflict(row) for row in rows]

    async def get_conflict(self, conflict_id: str) -> Optional[SemanticConflict]:
        """Get a single conflict by ID."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            result = await session.execute(
                text("""
                    SELECT id, concept_name, definitions, drift_score, drift_type,
                           explanation, detected_at, lineage_path, severity, status
                    FROM ai_drift_conflicts
                    WHERE id = :conflict_id
                """),
                {"conflict_id": conflict_id},
            )
            row = result.fetchone()
            if not row:
                return None
            return self._row_to_conflict(row)

    async def update_conflict_status(
        self,
        conflict_id: str,
        status: str,
        resolution_note: Optional[str] = None,
    ) -> None:
        """Update conflict status (acknowledge, resolve, mark false positive)."""
        valid_statuses = {"open", "acknowledged", "resolved", "false_positive"}
        if status not in valid_statuses:
            raise ValueError(f"Invalid status '{status}'. Must be one of {valid_statuses}")

        async with self.db_pool() as session:
            from sqlalchemy import text

            await session.execute(
                text("""
                    UPDATE ai_drift_conflicts
                    SET status = :status,
                        resolution_note = COALESCE(:resolution_note, resolution_note),
                        updated_at = :updated_at
                    WHERE id = :conflict_id
                """),
                {
                    "conflict_id": conflict_id,
                    "status": status,
                    "resolution_note": resolution_note,
                    "updated_at": datetime.utcnow(),
                },
            )
            await session.commit()

        logger.info(
            "Conflict status updated",
            conflict_id=conflict_id,
            status=status,
        )

    # ------------------------------------------------------------------
    # Trends
    # ------------------------------------------------------------------

    async def get_drift_trends(self, tenant_id: str, days: int = 30) -> dict:
        """Get drift score trends over time.

        Returns ``{dates: [...], scores: [...], conflict_counts: [...]}``.
        """
        async with self.db_pool() as session:
            from sqlalchemy import text

            cutoff = datetime.utcnow() - timedelta(days=days)

            result = await session.execute(
                text("""
                    SELECT
                        DATE(scanned_at) AS scan_date,
                        AVG(overall_drift_score) AS avg_score,
                        SUM(conflicts_found) AS total_conflicts
                    FROM ai_drift_scans
                    WHERE tenant_id = :tenant_id AND scanned_at >= :cutoff
                    GROUP BY DATE(scanned_at)
                    ORDER BY scan_date ASC
                """),
                {"tenant_id": tenant_id, "cutoff": cutoff},
            )
            rows = result.fetchall()

            dates = []
            scores = []
            conflict_counts = []
            for row in rows:
                dates.append(str(row[0]))
                scores.append(round(float(row[1]), 4))
                conflict_counts.append(int(row[2]))

            return {
                "dates": dates,
                "scores": scores,
                "conflict_counts": conflict_counts,
            }

    # ------------------------------------------------------------------
    # Helpers — normalisation
    # ------------------------------------------------------------------

    async def _normalize_field_name(self, name: str) -> str:
        """Normalise a field name for grouping: lowercase, strip separators,
        singularise simple cases.
        """
        # Lowercase
        normalised = name.lower().strip()

        # Replace common separators with nothing
        normalised = re.sub(r"[-_ ]+", "", normalised)

        # Simple singularisation
        if normalised in _IRREGULAR_PLURALS:
            normalised = _IRREGULAR_PLURALS[normalised]
        elif normalised.endswith("ies") and len(normalised) > 4:
            normalised = normalised[:-3] + "y"
        elif normalised.endswith("ses") and len(normalised) > 4:
            normalised = normalised[:-2]
        elif normalised.endswith("s") and not normalised.endswith("ss"):
            normalised = normalised[:-1]

        return normalised

    async def _classify_severity(self, drift_score: float, field_count: int) -> str:
        """Classify severity from drift score and the number of conflicting
        definitions.  More conflicting sources amplifies severity.
        """
        # Bump effective score when many sources disagree
        effective = drift_score
        if field_count >= 4:
            effective = min(1.0, drift_score + 0.15)
        elif field_count >= 3:
            effective = min(1.0, drift_score + 0.1)

        for threshold, label in _SEVERITY_THRESHOLDS:
            if effective >= threshold:
                return label
        return "low"

    # ------------------------------------------------------------------
    # Helpers — persistence
    # ------------------------------------------------------------------

    async def _persist_scan(self, result: DriftScanResult) -> None:
        """Write a scan result and its conflicts to PostgreSQL."""
        async with self.db_pool() as session:
            from sqlalchemy import text

            # Scan record
            await session.execute(
                text("""
                    INSERT INTO ai_drift_scans
                        (scan_id, tenant_id, scanned_at, total_fields,
                         conflicts_found, overall_drift_score, scan_data)
                    VALUES
                        (:scan_id, :tenant_id, :scanned_at, :total_fields,
                         :conflicts_found, :overall_drift_score, :scan_data)
                """),
                {
                    "scan_id": result.scan_id,
                    "tenant_id": result.tenant_id,
                    "scanned_at": result.scanned_at,
                    "total_fields": result.total_fields_scanned,
                    "conflicts_found": result.conflicts_found,
                    "overall_drift_score": result.overall_drift_score,
                    "scan_data": json.dumps(result.to_dict()),
                },
            )

            # Individual conflict records
            for conflict in result.conflicts:
                await session.execute(
                    text("""
                        INSERT INTO ai_drift_conflicts
                            (id, scan_id, tenant_id, concept_name, drift_score,
                             drift_type, explanation, severity, status,
                             lineage_path, definitions, detected_at, updated_at)
                        VALUES
                            (:id, :scan_id, :tenant_id, :concept_name, :drift_score,
                             :drift_type, :explanation, :severity, :status,
                             :lineage_path, :definitions, :detected_at, :updated_at)
                    """),
                    {
                        "id": conflict.id,
                        "scan_id": result.scan_id,
                        "tenant_id": result.tenant_id,
                        "concept_name": conflict.concept_name,
                        "drift_score": conflict.drift_score,
                        "drift_type": conflict.drift_type,
                        "explanation": conflict.explanation,
                        "severity": conflict.severity,
                        "status": conflict.status,
                        "lineage_path": conflict.lineage_path,
                        "definitions": json.dumps(
                            [d.to_dict() for d in conflict.definitions]
                        ),
                        "detected_at": conflict.detected_at,
                        "updated_at": conflict.detected_at,
                    },
                )

            await session.commit()

    # ------------------------------------------------------------------
    # Helpers — fetch field definitions
    # ------------------------------------------------------------------

    async def _fetch_field_definitions(
        self, tenant_id: str, scope: Optional[str] = None
    ) -> list[FieldDefinition]:
        """Fetch field definitions from the catalog / connected systems.

        In production this calls the data-service catalog API.  Here we pull
        from a local ``ai_field_definitions`` table that is populated by the
        catalog sync process.
        """
        async with self.db_pool() as session:
            from sqlalchemy import text

            query = """
                SELECT source_system, dataset, field_name, data_type,
                       description, sample_values, business_glossary_term,
                       last_updated
                FROM ai_field_definitions
                WHERE tenant_id = :tenant_id
            """
            params: dict = {"tenant_id": tenant_id}

            if scope:
                query += " AND (source_system = :scope OR dataset = :scope)"
                params["scope"] = scope

            result = await session.execute(text(query), params)
            rows = result.fetchall()

            definitions = []
            for row in rows:
                sample_values = row[5]
                if isinstance(sample_values, str):
                    try:
                        sample_values = json.loads(sample_values)
                    except json.JSONDecodeError:
                        sample_values = [sample_values]
                elif sample_values is None:
                    sample_values = []

                last_updated = row[7]
                if isinstance(last_updated, str):
                    last_updated = datetime.fromisoformat(last_updated)

                definitions.append(
                    FieldDefinition(
                        source_system=row[0],
                        dataset=row[1],
                        field_name=row[2],
                        data_type=row[3],
                        description=row[4],
                        sample_values=sample_values,
                        business_glossary_term=row[6],
                        last_updated=last_updated or datetime.utcnow(),
                    )
                )

            return definitions

    # ------------------------------------------------------------------
    # Helpers — LLM response parsing
    # ------------------------------------------------------------------

    def _parse_llm_drift_response(self, response_text: str) -> dict:
        """Parse the JSON response from the LLM, handling common formatting
        issues like markdown code fences.
        """
        text = response_text.strip()

        # Strip markdown code fences
        if text.startswith("```"):
            lines = text.split("\n")
            # Remove first and last lines if they are fences
            if lines[0].startswith("```"):
                lines = lines[1:]
            if lines and lines[-1].strip() == "```":
                lines = lines[:-1]
            text = "\n".join(lines)

        try:
            parsed = json.loads(text)
        except json.JSONDecodeError:
            logger.warning("Failed to parse LLM drift response", response=text[:200])
            return {
                "drift_score": 0.5,
                "drift_type": "definition_mismatch",
                "explanation": text[:500] if text else "Unable to parse LLM response.",
            }

        # Validate and clamp values
        drift_score = float(parsed.get("drift_score", 0.5))
        drift_score = max(0.0, min(1.0, drift_score))

        valid_types = {
            "definition_mismatch",
            "type_divergence",
            "value_range_drift",
            "temporal_drift",
        }
        drift_type = parsed.get("drift_type", "definition_mismatch")
        if drift_type not in valid_types:
            drift_type = "definition_mismatch"

        explanation = parsed.get("explanation", "No explanation provided.")

        return {
            "drift_score": drift_score,
            "drift_type": drift_type,
            "explanation": explanation,
        }

    # ------------------------------------------------------------------
    # Helpers — heuristic fallback
    # ------------------------------------------------------------------

    def _heuristic_drift_score(
        self, definitions: list[FieldDefinition]
    ) -> Tuple[float, str, str]:
        """Compute a rough drift score without the LLM by comparing data types,
        descriptions, and sample values.
        """
        score = 0.0
        reasons: list[str] = []

        # Check data type divergence
        data_types = {d.data_type.lower() for d in definitions if d.data_type}
        if len(data_types) > 1:
            score += 0.4
            reasons.append(f"Data types differ: {', '.join(sorted(data_types))}")

        # Check description similarity (very basic)
        descriptions = [d.description for d in definitions if d.description]
        if len(descriptions) >= 2:
            # Simple word-overlap similarity between first two
            words_a = set(descriptions[0].lower().split())
            words_b = set(descriptions[1].lower().split())
            if words_a and words_b:
                overlap = len(words_a & words_b) / max(len(words_a | words_b), 1)
                if overlap < 0.3:
                    score += 0.3
                    reasons.append("Descriptions have low word overlap")

        # Check sample value overlap
        all_samples = [set(d.sample_values) for d in definitions if d.sample_values]
        if len(all_samples) >= 2:
            overlap = len(all_samples[0] & all_samples[1]) / max(
                len(all_samples[0] | all_samples[1]), 1
            )
            if overlap < 0.1:
                score += 0.2
                reasons.append("Sample values have little overlap")

        # Check temporal drift — large gap in last_updated
        timestamps = [d.last_updated for d in definitions if d.last_updated]
        if len(timestamps) >= 2:
            sorted_ts = sorted(timestamps)
            gap = (sorted_ts[-1] - sorted_ts[0]).days
            if gap > 365:
                score += 0.1
                reasons.append(f"Definitions updated {gap} days apart")

        score = min(1.0, score)

        drift_type = "definition_mismatch"
        if len(data_types) > 1:
            drift_type = "type_divergence"
        elif timestamps and len(timestamps) >= 2:
            sorted_ts = sorted(timestamps)
            if (sorted_ts[-1] - sorted_ts[0]).days > 365:
                drift_type = "temporal_drift"

        explanation = "; ".join(reasons) if reasons else "Heuristic comparison found minor differences."

        return score, drift_type, explanation

    # ------------------------------------------------------------------
    # Helpers — row mapping
    # ------------------------------------------------------------------

    def _row_to_conflict(self, row) -> SemanticConflict:
        """Map a database row to a :class:`SemanticConflict`."""
        definitions_raw = row[2]
        if isinstance(definitions_raw, str):
            definitions_raw = json.loads(definitions_raw)

        definitions = [
            FieldDefinition.from_dict(d) if isinstance(d, dict) else d
            for d in (definitions_raw or [])
        ]

        detected_at = row[6]
        if isinstance(detected_at, str):
            detected_at = datetime.fromisoformat(detected_at)

        return SemanticConflict(
            id=row[0],
            concept_name=row[1],
            definitions=definitions,
            drift_score=float(row[3]),
            drift_type=row[4],
            explanation=row[5],
            detected_at=detected_at,
            lineage_path=row[7],
            severity=row[8],
            status=row[9],
        )

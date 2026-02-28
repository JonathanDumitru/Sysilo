# Project Research Summary

**Project:** Sysilo
**Domain:** Enterprise integration intelligence platform with AI operations copilot
**Researched:** 2026-02-28
**Confidence:** MEDIUM

## Executive Summary

Sysilo is best built as an enterprise-first integration intelligence control plane, not as a broad automation builder. The research converges on a relational core (inventory, ownership, lineage, governance), event-driven health intelligence, and an AI copilot constrained by tool mediation, policy gates, and evidence-linked recommendations. Experts in this category sequence delivery from trustworthy metadata and telemetry foundations toward guided action, then toward selective automation.

The recommended delivery approach is a domain-centric modular monolith on Node.js/TypeScript with Next.js UI, PostgreSQL as system-of-record, Temporal for durable workflows, and a Kafka-compatible backbone for ingestion and projections. This gives fast v1 delivery without early distributed-systems overhead while preserving clear extraction boundaries for connector runtime and health processing once scale demands it.

The highest risks are false trust signals: "green but broken" health models, weak ownership/lineage metadata, and copilot recommendations that are plausible but unsafe. Mitigation is to enforce governance completeness in core schemas, define health as transport plus semantic/freshness/SLO signals, and require evidence and approval gates for all actionable AI outputs.

## Key Findings

### Recommended Stack

The stack is strongly opinionated around reliability and governance for enterprise operations: TypeScript + Node.js for shared contracts and backend velocity, Next.js for operator workflows, PostgreSQL for canonical metadata and joins, Temporal for durable workflow execution, and Kafka-compatible messaging for event decoupling and replay. Observability and validation are treated as first-class with OpenTelemetry, Zod, and structured logging.

Critical version constraints are straightforward and stable for 2026 planning: Node 22 LTS, TypeScript 5.8.x, React 19 + Next.js 16, PostgreSQL 17, and Temporal 1.27.x. This combination supports monorepo development now and later service extraction with minimal churn.

**Core technologies:**
- TypeScript 5.8 + Node.js 22 LTS: end-to-end typed contracts and stable backend runtime for connectors, APIs, and workers.
- Next.js 16 + React 19: enterprise console for inventory, health operations, and copilot workflows.
- PostgreSQL 17: canonical relational source for tenants, integrations, lineage, ownership, incidents, and governance state.
- Temporal 1.27: durable orchestration for retries, backfills, and auditable long-running integration workflows.
- Kafka-compatible bus: decoupled ingestion, replay, and near-real-time health/event processing.
- OpenAI Responses API + tool calling: constrained AI reasoning with explicit tool boundaries and controllable actions.

### Expected Features

Feature research is clear that v1 success is inventory + health + governance-ready operations, with AI as an accelerator layered on trusted data. Differentiation should come from evidence-based diagnosis, mapping assistance, and safe remediation guidance, not connector-count marketing or autonomous action.

**Must have (table stakes):**
- Auto-discovered integration inventory with ownership metadata.
- Health monitoring and alerting with actionable operational context.
- Dependency/data lineage graph for blast-radius understanding.
- Audit trail, governance controls, and enterprise auth (SSO/SCIM/RBAC).

**Should have (competitive):**
- AI incident diagnostician with cited evidence.
- AI mapping/transformation copilot for schema and field mapping work.
- Governance-aware remediation recommendations with approval workflows.
- Proactive risk scoring and cross-team incident memory (after telemetry quality is proven).

**Defer (v2+):**
- Universal Zapier-style automation builder.
- Long-tail connector breadth before quality depth.
- Fully autonomous remediation with no human approval.
- Real-time-by-default sync for all integrations.

### Architecture Approach

Architecture should start as a domain-centric modular monolith with explicit boundaries: inventory, health, governance, copilot, and connector runtime. Ingestion is event-driven, health is projection-based, and copilot access is tool-mediated with policy checks and auditability. This matches the required build order: foundation (identity/contracts/observability) -> inventory slice -> health intelligence -> governance overlay -> AI copilot.

**Major components:**
1. Experience layer (Ops UI/Admin/API): operational workflows, dashboards, and controlled action entry points.
2. Application services (inventory, health, governance, copilot, connector runtime): core business behavior and orchestration.
3. Data/platform layer (Postgres, metrics store, queue/bus, object storage): persistence, projections, event transport, and evidence storage.

### Critical Pitfalls

1. **Inventory without ownership/lineage completeness** - enforce mandatory owner, lineage, and criticality fields (with explicit unknown states) and block trust rollups on missing governance metadata.
2. **Health disconnected from business impact** - model health as transport + semantic correctness + freshness + contract adherence tied to explicit SLOs.
3. **Unsafe or hallucinated copilot guidance** - require evidence citations, confidence scoring, and approval-gated actions for every recommendation.
4. **Identity/environment ambiguity across systems** - enforce canonical tenant/env/system/integration identity and strict context propagation.
5. **Governance and audit added too late** - make auditability, policy enforcement, and least-privilege access platform primitives before broad copilot rollout.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation and Canonical Integration Graph
**Rationale:** All higher-order intelligence depends on reliable identity, ownership, lineage, and contracts.
**Delivers:** Tenant/auth model, core schema, connector SDK baseline, first connector vertical slice, canonical inventory graph.
**Addresses:** Auto-discovery, ownership mapping, lineage table stakes.
**Avoids:** Discovery-without-governance and identity confusion pitfalls.

### Phase 2: Multi-Layer Health Intelligence
**Rationale:** AI and remediation are low-value without trustworthy operational signals.
**Delivers:** Event ingestion pipeline, transport/freshness/semantic health model, SLO-aligned alerting, health dashboards.
**Uses:** Kafka-compatible bus, Temporal jobs, metrics store, OpenTelemetry.
**Implements:** Health Intelligence service and projection architecture.

### Phase 3: Contract Intelligence and Drift Control
**Rationale:** Schema drift is a major enterprise failure mode and must be prevented before scaling actions.
**Delivers:** Schema diffing, contract compatibility checks, drift alerts, versioned mapping controls.
**Addresses:** AI mapping copilot prerequisites and governance readiness.
**Avoids:** Late-detected mapping breakage and downstream corruption.

### Phase 4: Evidence-Grounded AI Copilot
**Rationale:** Copilot must earn trust via explainability before deeper automation.
**Delivers:** Tool-mediated diagnosis and mapping suggestions with citations, confidence scores, and reversible recommendation paths.
**Uses:** OpenAI Responses API, retrieval context, strict tool schemas.
**Implements:** Copilot orchestration service with policy hooks.

### Phase 5: Incident Correlation and Operator Workflow Intelligence
**Rationale:** MTTR reduction requires correlated context, not raw alert volume.
**Delivers:** Dependency-aware correlation, deduplication, impact scoring, incident briefs with next-best actions.
**Addresses:** Alert-fatigue and triage-efficiency gaps.
**Avoids:** Low-context alerting and repeated manual triage loops.

### Phase 6: Governance, Audit, and Enterprise Control Plane Hardening
**Rationale:** Procurement and production rollout depend on verifiable compliance controls.
**Delivers:** Immutable audit trail for AI-assisted actions, approval workflows, RBAC/ABAC expansion, redaction and secrets controls.
**Addresses:** Enterprise auth/access and policy enforcement requirements.
**Avoids:** Late security-review blockers and non-repudiation gaps.

### Phase 7: Quality-Gated Connector Expansion and Portfolio Optimization
**Rationale:** Breadth should follow proven reliability outcomes, not precede them.
**Delivers:** Controlled connector expansion, portfolio risk/cost insights, scale optimizations in connector runtime and health processing.
**Addresses:** Competitive breadth and optimization differentiators.
**Avoids:** Premature long-tail connector sprawl.

### Phase Ordering Rationale

- Dependencies are strict: inventory/identity -> health telemetry -> contract controls -> copilot -> advanced workflows -> scale-out.
- Architecture grouping aligns with domain boundaries, enabling parallel work without coupling UI, connectors, and governance logic.
- This order directly mitigates the highest-risk pitfalls before introducing high-risk capabilities like AI-guided action and connector breadth expansion.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** Connector priority and provider API constraints need targeted research per initial connector set.
- **Phase 4:** Copilot evaluation harness (grounding, hallucination rate, safety gates) needs method-specific design research.
- **Phase 6:** Enterprise governance controls may require deployment-target-specific compliance mapping.
- **Phase 7:** Scaling thresholds and service extraction points need workload-specific validation.

Phases with standard patterns (skip research-phase):
- **Phase 2:** Event-driven health ingestion/projections and SLO modeling are well-documented patterns.
- **Phase 3:** Schema versioning, contract testing, and drift detection follow established implementation approaches.
- **Phase 5:** Incident correlation and dedup pipelines are mature operational patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Recommendations are consistent and align with stable enterprise SaaS patterns and concrete versioning guidance. |
| Features | HIGH | Table stakes, differentiators, and anti-features are coherent and dependency-mapped for roadmap use. |
| Architecture | MEDIUM | Strong pattern guidance, but some scale decisions (metrics store and extraction timing) are inference-based until load profiles are known. |
| Pitfalls | HIGH | Failure modes are concrete, operationally realistic, and mapped to prevention phases with measurable warning signs. |

**Overall confidence:** MEDIUM

### Gaps to Address

- Initial connector set and sequencing: finalize top enterprise systems for v1 based on target customer profile and API feasibility.
- Health semantics by domain: define per-integration SLO/freshness/reconciliation rules to avoid generic "green" metrics.
- Copilot quality bar: establish acceptance metrics for evidence coverage, recommendation precision, and unsafe-action prevention.
- Governance target depth: clarify required compliance frameworks and audit retention requirements before Phase 6 implementation detail.
- Cost model at scale: validate ingestion, storage, and copilot token-cost assumptions with projected tenant and event growth.

## Sources

### Primary (HIGH confidence)
- `.planning/research/STACK.md` - recommended technologies, versions, and anti-stack guidance.
- `.planning/research/FEATURES.md` - table stakes, differentiators, anti-features, and dependency map.
- `.planning/research/PITFALLS.md` - critical failure modes, warning signs, and pitfall-to-phase mapping.
- `.planning/PROJECT.md` - product goals, scope constraints, and v1 priorities.

### Secondary (MEDIUM confidence)
- `.planning/research/ARCHITECTURE.md` - architecture patterns and build-order guidance (noted as inference-heavy in parts).

### Tertiary (LOW confidence)
- None.

---
*Research completed: 2026-02-28*
*Ready for roadmap: yes*

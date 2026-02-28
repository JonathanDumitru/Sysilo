# Pitfalls Research

**Domain:** Enterprise integration intelligence platform with AI operations copilot
**Researched:** 2026-02-28
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: "Discovery" without trustworthy ownership and lineage

**What goes wrong:**
The inventory lists integrations, but does not reliably identify owners, downstream dependencies, business criticality, and data lineage.

**Why it happens:**
Teams optimize for connector count and UI completeness instead of governance completeness.

**How to avoid:**
Make ownership, lineage edges, and criticality mandatory schema fields with explicit "unknown" states; block "healthy" rollups when governance metadata is missing.

**Warning signs:**
- Many integrations show "owner: unknown" after onboarding
- Incident responders still ask in chat "who owns this flow?"
- Health dashboards cannot show blast radius for a failing integration

**Phase to address:**
Phase 1: Integration graph and metadata model foundation

---

### Pitfall 2: Health metrics disconnected from business impact

**What goes wrong:**
The platform reports API uptime and job success rates, but misses data freshness, reconciliation drift, and SLA impact, so "green" systems still break reporting and operations.

**Why it happens:**
Implementation tracks only transport-level telemetry because it is easier to collect.

**How to avoid:**
Define health as layered signals: transport, semantic correctness, freshness windows, and contract adherence; tie each integration to explicit SLOs.

**Warning signs:**
- Ops reports recurring "all green but broken" incidents
- No metric for freshness lag or reconciliation mismatch
- Alert volume is high but postmortems cite missing signal quality

**Phase to address:**
Phase 2: Multi-layer health intelligence and SLO definitions

---

### Pitfall 3: AI copilot hallucinating confident but unsafe guidance

**What goes wrong:**
Copilot provides plausible root-cause claims or mapping suggestions without sufficient evidence, leading to harmful operator actions.

**Why it happens:**
RAG and tool outputs are not grounded in verifiable runtime evidence and provenance.

**How to avoid:**
Require evidence-backed answers: every recommendation must include source artifacts (logs, traces, schema diffs, run IDs), confidence scoring, and safe-action gating.

**Warning signs:**
- Suggestions lack citations to concrete system evidence
- Users report "sounds right" guidance that fails in execution
- Same incident gets contradictory copilot diagnoses

**Phase to address:**
Phase 4: Evidence-grounded copilot and action-safety framework

---

### Pitfall 4: Cross-system identity and environment confusion

**What goes wrong:**
The system confuses tenant, environment, or account context, causing incorrect diagnostics or remediation recommendations against the wrong system.

**Why it happens:**
Identity normalization is deferred, and naming conventions are assumed consistent across tools.

**How to avoid:**
Implement canonical entity identity (tenant, env, system, integration instance) with strict context propagation and hard guardrails on cross-environment actions.

**Warning signs:**
- Duplicate entities representing the same integration endpoint
- Incidents attributed to prod while evidence comes from staging
- Manual corrections needed after automated recommendations

**Phase to address:**
Phase 1: Canonical identity and context model

---

### Pitfall 5: Schema drift treated as noise instead of first-class risk

**What goes wrong:**
Mapping breakages surface late after downstream corruption, because schema and contract drift are only detected after failures propagate.

**Why it happens:**
Drift detection and contract testing are considered "nice to have" and postponed.

**How to avoid:**
Continuously diff schemas and payload contracts per integration pair; enforce compatibility policies and proactive drift alerts before deployment windows.

**Warning signs:**
- Frequent hotfixes for mapping or transformation errors
- Downstream teams detect mismatches before the platform does
- Rising manual reconciliation effort each release cycle

**Phase to address:**
Phase 3: Contract intelligence and schema-drift controls

---

### Pitfall 6: Alert fatigue from low-context incident surfaces

**What goes wrong:**
Operators receive many alerts but little actionable context, increasing MTTR and missed critical incidents.

**Why it happens:**
Alerting is threshold-based without dependency-aware correlation, severity scoring, or suggested first steps.

**How to avoid:**
Introduce event correlation by dependency graph, impact scoring, deduplication windows, and incident briefs that include likely cause + next best action.

**Warning signs:**
- High alert volume with low acknowledge quality
- Repeated manual triage queries for the same issue
- MTTR increases despite better monitoring coverage

**Phase to address:**
Phase 5: Incident correlation and operator workflow intelligence

---

### Pitfall 7: Governance and auditability bolted on after AI features

**What goes wrong:**
Enterprise buyers reject adoption because recommendation history, data access logs, and policy controls are incomplete.

**Why it happens:**
Teams prioritize visible copilot UX before audit and compliance requirements.

**How to avoid:**
Design policy enforcement, immutable audit trails, and role/attribute-based access controls as platform primitives before broad copilot rollout.

**Warning signs:**
- Security reviews block production rollout late in cycle
- No immutable record of who accepted/rejected AI actions
- Sensitive integration metadata appears in broad user contexts

**Phase to address:**
Phase 6: Governance, RBAC/ABAC, and enterprise audit controls

---

### Pitfall 8: Long-tail connector expansion before core intelligence quality

**What goes wrong:**
The roadmap chases connector breadth and marketplace optics, but core inventory accuracy and health diagnostics remain weak.

**Why it happens:**
Feature pressure favors visible connector count over reliability outcomes.

**How to avoid:**
Gate connector expansion on quality SLAs for core systems (coverage depth, detection precision, MTTR reduction).

**Warning signs:**
- Connector count grows while incident quality metrics stagnate
- Enterprise pilots request fewer systems but deeper reliability features
- Backlog dominated by adapter requests over intelligence improvements

**Phase to address:**
Phase 7: Scale-out only after core reliability benchmarks are met

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Free-form integration metadata | Faster onboarding | Broken ownership/search/correlation | Only during discovery spikes with strict timebox |
| Single "status" health metric | Simple dashboard | False confidence and missed data-quality failures | Never for production reporting |
| Unversioned transformation/mapping rules | Rapid iteration | Impossible incident replay and blame analysis | Never |
| Copilot output without evidence links | Faster UX delivery | Unsafe recommendations, low trust | Never |
| Per-connector one-off auth handling | Quicker connector shipping | Security inconsistency and maintenance burden | Only for internal prototype connectors |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Salesforce/CRM | Assume API success equals business success | Validate object-level semantic outcomes and reconciliation |
| ERP systems (NetSuite/SAP) | Ignore batch window and backfill behavior | Model freshness expectations and maintenance windows explicitly |
| HRIS (Workday/SuccessFactors) | Treat schema as stable | Continuously monitor contract drift and field deprecations |
| Data warehouses (Snowflake/BigQuery) | Check load completion only | Verify downstream model readiness and row-level quality checks |
| ITSM (ServiceNow/Jira) | Alert by ticket count thresholds only | Correlate with dependency impact and incident lifecycle states |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Recompute full integration graph on each event | UI lag, delayed alerts | Incremental graph updates + materialized views | ~10k+ integration edges or bursty event streams |
| Synchronous fan-out health checks | Timeouts and cascading failures | Queue-based async probes with budgets and circuit breakers | During third-party rate-limit or outage periods |
| Unbounded log retrieval for copilot context | High latency, high cost, token overflow | Windowed retrieval, relevance ranking, evidence caps | Multi-day incidents or high-cardinality services |
| Per-request policy evaluation without caching | Slow dashboards and actions | Cache policy decisions with short TTL and invalidation hooks | Large enterprise org trees (1k+ roles) |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Shared integration credentials across environments | Cross-env blast radius, data leakage | Per-env scoped secrets, rotation, and just-in-time access |
| Copilot exposing sensitive payload fragments in answers | Compliance and privacy incidents | Redaction pipeline, field-level sensitivity tags, response filters |
| Weak audit event model for AI-assisted actions | Failed compliance reviews and non-repudiation gaps | Immutable action logs with actor, context, evidence, and approval chain |
| Over-privileged service accounts for discovery | Lateral movement risk | Least privilege templates and connector-specific permission baselines |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Single global health score with no explainability | Users distrust score and ignore it | Layered health breakdown with causal drill-down |
| Copilot suggestions mixed with system facts | Ambiguous confidence, unsafe actioning | Separate "observed facts" from "recommended actions" sections |
| Incident views missing dependency blast radius | Slow triage and repeated escalations | Graph-first incident view with upstream/downstream impact |
| No explicit "unknown" state in metadata | Hidden data quality gaps | Show unknowns prominently and make them operational tasks |

## "Looks Done But Isn't" Checklist

- [ ] **Integration Inventory:** Includes owner, system-of-record, data domains, and dependency edges, not just connector names.
- [ ] **Health Intelligence:** Covers freshness and semantic correctness, not only transport uptime.
- [ ] **AI Copilot:** Every recommendation includes evidence links, confidence, and reversible action path.
- [ ] **Governance:** Audit logs capture user intent, approvals, and execution context for AI-assisted changes.
- [ ] **Enterprise Readiness:** RBAC/ABAC, secrets management, and data redaction verified in multi-tenant scenarios.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Inaccurate ownership/lineage graph | HIGH | Freeze automation suggestions, run metadata reconciliation campaign, backfill ownership via enforced workflow |
| Misleading health signals | HIGH | Re-baseline SLO model, introduce semantic/freshness checks, reclassify alert severities |
| Unsafe copilot recommendations | HIGH | Disable auto-action paths, enforce evidence gating, replay incident recommendations and retrain prompts/policies |
| Schema drift incidents | MEDIUM | Roll back mapping versions, apply contract tests in CI/CD, create proactive drift monitors |
| Auditability gaps discovered in security review | HIGH | Implement immutable audit pipeline, retro-log available history, restrict privileged features until compliant |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Discovery without ownership/lineage | Phase 1: Integration graph foundation | >=95% of active integrations have owner + dependency metadata populated |
| Health disconnected from business impact | Phase 2: Multi-layer health model | Post-incident review shows "green but broken" class reduced quarter-over-quarter |
| Schema drift treated as noise | Phase 3: Contract intelligence | Drift alerts detected before downstream breakage in staging/prod validation windows |
| Hallucinated or unsafe copilot advice | Phase 4: Evidence-grounded copilot | 100% actionable recommendations carry evidence refs + confidence + approval gate |
| Alert fatigue from low-context incidents | Phase 5: Incident correlation | Alert volume-to-action ratio improves and MTTR trends downward |
| Governance bolted on late | Phase 6: Governance and audit controls | Security/compliance review passes without critical findings |
| Premature connector sprawl | Phase 7: Quality-gated expansion | Core reliability KPIs hit target before adding new long-tail connectors |

## Sources

- Enterprise integration operating model experience and common incident/postmortem patterns
- Observed failure modes from SaaS/ETL/IPAAS operations in multi-system environments
- Project constraints and scope in `.planning/PROJECT.md`

---
*Pitfalls research for: Sysilo (enterprise integration intelligence + AI copilot)*
*Researched: 2026-02-28*

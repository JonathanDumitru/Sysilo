# Architecture Research

**Domain:** Enterprise integration intelligence platform (SaaS discovery, health monitoring, AI copilot)
**Researched:** 2026-02-28
**Confidence:** MEDIUM

## Standard Architecture

### System Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                           Experience Layer                                │
├────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌───────────────────────────┐  │
│  │ Web App (Ops)   │  │ Admin Console   │  │ API Clients (future)      │  │
│  └────────┬────────┘  └────────┬────────┘  └──────────────┬────────────┘  │
│           │                    │                          │               │
├───────────┴────────────────────┴──────────────────────────┴───────────────┤
│                           Application Layer                                │
├────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ API Gateway + Domain Services:                                      │  │
│  │ - Integration Inventory Service                                     │  │
│  │ - Health Intelligence Service                                       │  │
│  │ - Ownership & Governance Service                                    │  │
│  │ - AI Copilot Orchestration Service                                 │  │
│  │ - Connector Runtime + Job Scheduler                                │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────────────────┤
│                          Data & Platform Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Postgres     │  │ Time-series  │  │ Message Bus  │  │ Object Store │  │
│  │ (metadata)   │  │ metrics store│  │ / queue      │  │ logs/schemas │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │
└────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Web App (Ops/IT) | Surface inventory, health, ownership, and recommended actions | React/Next.js dashboard with server-side auth and API calls |
| API Gateway | Stable contract, authn/authz, tenant routing, rate limiting | REST/GraphQL gateway with OpenAPI and middleware |
| Integration Inventory Service | Canonical model of systems, connectors, and data flow edges | Service module with Postgres persistence and connector ingestion |
| Health Intelligence Service | Compute sync status, incident hotspots, SLIs, anomaly flags | Event-driven processors plus scheduled aggregation jobs |
| AI Copilot Orchestrator | Build context, call LLM tools, produce explainable recommendations | Tool-calling pipeline with guardrails and audit trail |
| Connector Runtime | Pull metadata/health from third-party systems | Isolated workers per connector with retries/backoff |
| Postgres | Source of truth for tenants, assets, ownership, topology | Relational schema with migrations |
| Metrics Store | Fast time-window queries for health timelines | ClickHouse/Timescale/managed TSDB |
| Queue/Bus | Decouple ingestion, scoring, and notifications | SQS/Kafka/Rabbit-style queue |

## Recommended Project Structure

```
src/
├── app/                     # Entrypoints, routing, API surface
│   ├── api/                 # HTTP handlers and transport adapters
│   └── web/                 # UI routes, pages, and shells
├── domains/                 # Core bounded contexts
│   ├── inventory/           # Integration graph, assets, ownership edges
│   ├── health/              # Status scoring, incidents, trend logic
│   ├── governance/          # Policies, approvals, access constraints
│   └── copilot/             # Prompt/tool orchestration and response shaping
├── connectors/              # Third-party integrations and mapping adapters
│   ├── runtime/             # Worker execution engine and retry policies
│   └── providers/           # Per-provider connector implementations
├── platform/                # Cross-cutting infrastructure
│   ├── db/                  # ORM/query layer, migrations, repositories
│   ├── queue/               # Producer/consumer abstractions
│   ├── metrics/             # Telemetry ingestion and aggregation plumbing
│   └── auth/                # Tenant/session/role enforcement
├── jobs/                    # Scheduled/background jobs
├── contracts/               # API/event schemas, DTOs, validation
└── tests/                   # Unit, integration, and workflow tests
```

### Structure Rationale

- **`domains/`:** Enforces clear business boundaries so inventory, health, governance, and copilot can evolve independently.
- **`connectors/`:** Isolates unstable third-party APIs from core domain logic and limits blast radius.
- **`platform/`:** Centralizes shared infra concerns to avoid duplication and hidden coupling.
- **`contracts/`:** Keeps API and event schemas explicit, versioned, and testable across services/workers.
- **`jobs/`:** Makes asynchronous computation (health scoring, drift checks) first-class and observable.

## Architectural Patterns

### Pattern 1: Domain-Centric Modular Monolith (v1)

**What:** One deployable backend with strict domain module boundaries and explicit contracts.
**When to use:** Early stage product with tight iteration loops and limited team size.
**Trade-offs:** Faster delivery and simpler ops; must enforce boundaries to avoid a tangled monolith.

**Example:**
```typescript
// app/api/integrations/getIntegration.ts
export async function getIntegration(id: string, tenantId: string) {
  return inventoryService.getById({ id, tenantId });
}
```

### Pattern 2: Event-Driven Ingestion + Derived Health Projections

**What:** Connector workers emit raw events; health service builds derived read models.
**When to use:** High fan-in from many connectors and asynchronous external APIs.
**Trade-offs:** Better resilience and decoupling; eventual consistency and replay complexity.

**Example:**
```typescript
// connectors/runtime/publishSyncEvent.ts
queue.publish("integration.sync.completed", {
  tenantId,
  integrationId,
  provider,
  durationMs,
  errorCount,
  completedAt,
});
```

### Pattern 3: AI Copilot with Tool Mediation and Policy Gates

**What:** LLM never queries arbitrary data directly; it uses approved tools with policy checks.
**When to use:** Enterprise environments requiring traceability and controlled recommendations.
**Trade-offs:** Safer and auditable outputs; extra orchestration latency and implementation effort.

## Data Flow

### Request Flow

```
[Ops Lead opens health dashboard]
    ↓
[Web App] → [API Gateway] → [Health Service] → [Read Models / Metrics Store]
    ↓             ↓               ↓                     ↓
[UI cards] ← [DTO shaping] ← [Scoring + joins] ← [Aggregated telemetry]
```

### State Management

```
[Server state cache/query client]
    ↓ (subscribe / invalidate)
[Dashboard Components] ←→ [User Actions] → [API Mutations] → [Backend State]
```

### Key Data Flows

1. **Inventory Discovery Flow:** Connector runtime polls provider metadata, normalizes entities, writes canonical integration graph into Postgres.
2. **Health Scoring Flow:** Sync events and failure logs land on queue, health workers compute SLIs/hotspots, projections stored in metrics/read models.
3. **Copilot Recommendation Flow:** User asks question, orchestrator fetches scoped context (inventory + recent incidents + ownership), calls LLM tools, returns ranked actions with evidence.
4. **Governance Escalation Flow:** High-risk recommendation or failing integration triggers policy checks and routes approval tasks/alerts to owners.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k integrations | Single modular backend + managed Postgres + queue workers; prioritize schema clarity and observability |
| 1k-100k integrations | Split connector runtime and health workers into independently scaled services; add dedicated metrics store and partitioning |
| 100k+ integrations | Separate high-write ingestion plane from query plane, introduce stream processing and domain service extraction where hotspots justify |

### Scaling Priorities

1. **First bottleneck:** Connector ingestion throughput and retries; fix with worker autoscaling, backpressure, and provider-specific concurrency limits.
2. **Second bottleneck:** Health timeline/query latency; fix with pre-aggregated projections, retention tiers, and read-optimized metrics store.

## Anti-Patterns

### Anti-Pattern 1: Connector Logic Embedded in Domain Services

**What people do:** Place provider-specific API behavior directly in inventory/health domain code.
**Why it's wrong:** Creates hard coupling, slows onboarding of new connectors, and increases regression risk.
**Do this instead:** Keep provider adapters in `connectors/providers` with normalized contracts consumed by domain services.

### Anti-Pattern 2: Direct LLM Access to Raw Production Data

**What people do:** Send broad data dumps to LLM prompts without constrained tools or policy checks.
**Why it's wrong:** Breaks governance posture, weakens explainability, and increases data leakage risk.
**Do this instead:** Use tool-mediated access with tenant scoping, audit logs, and deterministic evidence in responses.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Enterprise SaaS APIs (CRM/ERP/ITSM/Data) | Connector adapters + polling/webhooks | Expect API drift, per-provider rate limits, and auth token lifecycle complexity |
| Identity Provider (Okta/Azure AD) | OIDC/SAML SSO + SCIM (later) | Needed early for enterprise access and role mapping |
| LLM Provider | Tool-calling API behind orchestrator | Enforce prompt/tool versioning, redaction, and response auditing |
| Alerting/Comms (Slack/Email/PagerDuty) | Event-driven notifications | Keep idempotent delivery and escalation policy mapping |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `app/api` ↔ `domains/*` | In-process service interfaces | Transport concerns remain in API layer only |
| `connectors/runtime` ↔ `domains/inventory` | Queue events + normalized DTOs | Avoid synchronous dependency on external providers |
| `domains/health` ↔ `platform/metrics` | Repository interfaces + append/event reads | Keep scoring logic portable from storage tech |
| `domains/copilot` ↔ all domains | Read-only tool contracts | Copilot should not mutate core state directly |

## Build-Order Implications

1. **Foundation first:** Establish tenant/auth model, core Postgres schema, contract validation, and observability scaffolding before connector work.
2. **Inventory vertical slice:** Deliver one provider connector end-to-end into canonical inventory graph to validate domain boundaries early.
3. **Health visibility next:** Add event pipeline, scoring jobs, and timeline dashboard on top of inventory entities.
4. **Ownership/governance overlay:** Introduce owner mapping, policy rules, and escalation workflows once core telemetry is stable.
5. **AI copilot last in v1:** Add tool-mediated recommendations only after inventory + health data quality is reliable.
6. **Service extraction only by evidence:** Keep modular monolith until measured bottlenecks justify splitting connector runtime or health processing.

## Sources

- `/Users/dev/Documents/Software/Web/Sysilo/.planning/PROJECT.md`
- `/Users/dev/.codex/get-shit-done/templates/research-project/ARCHITECTURE.md`
- Inference from enterprise integration platform architecture conventions

---
*Architecture research for: Sysilo enterprise integration intelligence platform*
*Researched: 2026-02-28*

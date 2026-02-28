# Stack Research

**Domain:** Enterprise integration intelligence platform + AI integration copilot (SaaS)
**Researched:** 2026-02-28
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| TypeScript | 5.8.x | End-to-end type safety across UI, API, connectors, and shared contracts | Reduces integration-domain defects (schema mismatch, nullability drift, contract regressions) and improves maintainability for multi-team enterprise codebases | HIGH |
| Node.js | 22 LTS | Runtime for API, workers, connector orchestration, and AI tool-execution services | Strong ecosystem for integration tooling, JSON-heavy workloads, and rapid iteration; stable LTS base for enterprise workloads | HIGH |
| React + Next.js | React 19.x + Next.js 16.x | Enterprise console for inventory, health views, governance workflows, and copilot UX | Mature app-router + server components + strong DX; high velocity for dashboard-style products with auth, RBAC, and data-heavy UI | HIGH |
| PostgreSQL | 17.x | Source of truth for tenants, integrations, lineage metadata, ownership, incidents, and governance state | ACID reliability, mature indexing, strong JSON support, and predictable operations for enterprise SaaS | HIGH |
| Temporal | 1.27.x | Durable orchestration for connector sync, retries, backfills, and remediation workflows | Enterprise integration domains need stateful long-running workflows, deterministic retries, and auditability; Temporal is purpose-built for this | HIGH |
| Kafka-compatible event bus (Redpanda Cloud or Confluent Cloud) | 2026 managed release | Event backbone for health signals, connector state changes, and AI-observed anomalies | Decouples ingestion/processing, supports replay, enables near-real-time analytics and resilient pipeline evolution | HIGH |
| OpenAI Responses API + tool calling | 2026 model family | AI copilot reasoning for diagnosis, mapping suggestions, and guided remediation | Best fit for structured tool use, enterprise-grade reasoning loops, and integrating operational context with explicit actions | HIGH |
| OpenTelemetry | 1.30.x SDK ecosystem | Unified observability across API, workers, workflows, and LLM calls | Required for enterprise-grade traceability, SLOs, and debugging integration failures across distributed subsystems | HIGH |

### Supporting Libraries

| Library | Version | Purpose | When to Use | Confidence |
|---------|---------|---------|-------------|------------|
| Prisma | 6.x | Type-safe ORM + migrations for PostgreSQL | Core application persistence where developer speed and schema evolution consistency matter | HIGH |
| Zod | 3.24.x | Runtime validation for connector payloads and API inputs | All external boundaries: webhooks, connector adapters, and AI tool I/O validation | HIGH |
| TanStack Query | 5.x | Client-side data fetching/caching for operations dashboard | Use in all UI screens with frequent refresh and optimistic updates (health, incidents, ownership) | HIGH |
| Auth.js or WorkOS SDK | Latest stable | SSO/SAML/OIDC and enterprise auth flows | Multi-tenant enterprise onboarding and identity federation | HIGH |
| pgvector | 0.8.x | Vector similarity in PostgreSQL for lightweight retrieval contexts | Use only for scoped semantic recall in copilot (runbooks, prior incidents, mapping examples) | MEDIUM |
| Pino | 9.x | Structured logging for high-volume services | Mandatory in API/workers for machine-parseable logs and SIEM export | HIGH |
| BullMQ (optional edge queue) | 5.x | Short-lived background jobs local to service boundaries | Use for simple fire-and-forget jobs that do not require Temporal-level durability | MEDIUM |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| pnpm | Fast, deterministic monorepo package management | Use workspaces + lockfile enforcement in CI |
| Turborepo | Task orchestration and incremental builds/tests | Define strict task pipelines per package boundary |
| Vitest + Playwright | Unit/integration and E2E verification | Vitest for service logic; Playwright for critical enterprise workflows |
| ESLint + Prettier + TypeScript strict mode | Quality gate and consistency | Keep `strict: true`, `noImplicitAny: true`, and CI lint/test/type gates |
| Terraform | Infrastructure as code for cloud services | Treat data plane and control plane infra as versioned modules |
| Snyk or Trivy + Dependabot/Renovate | Vulnerability and dependency hygiene | Enforce SLAs on critical CVEs for enterprise posture |

## Installation

```bash
# Core
pnpm add typescript@^5.8 next@^16 react@^19 react-dom@^19 node@22 prisma @prisma/client zod pino @opentelemetry/api @opentelemetry/sdk-node

# Supporting
pnpm add @tanstack/react-query pg pgvector temporalio bullmq

# Dev dependencies
pnpm add -D vitest playwright eslint prettier turbo terraform
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Temporal | AWS Step Functions | If team is fully AWS-native, accepts vendor lock-in, and workflow complexity is moderate |
| PostgreSQL | CockroachDB | If you require multi-region active-active SQL by default and can absorb higher operational/cost complexity |
| Kafka-compatible bus | AWS EventBridge + SQS/SNS | If throughput is moderate and team prioritizes simpler AWS-managed primitives over replay-heavy streaming |
| Next.js | Remix | If you want simpler server-rendered patterns and smaller platform surface area |
| Prisma | Drizzle ORM | If you want lower abstraction and SQL-first patterns with tighter control |
| OpenAI Responses API | Hybrid with Anthropic/Google model router | If procurement or compliance requires multi-model portability from day one |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| MongoDB as system-of-record for core integration governance | Weak fit for relational lineage, ownership joins, and transactional consistency across incident/governance state | PostgreSQL 17 with normalized relational model + selective JSON columns |
| Ad-hoc cron + retry scripts for critical workflows | Poor durability, limited observability, and brittle failure recovery for enterprise integrations | Temporal workflows with explicit retries, compensation, and audit trails |
| Pure prompt-only copilot without tool boundaries | Hallucination and unsafe action risk in operational enterprise context | Tool-called copilot with schema-validated inputs/outputs and approval checkpoints |
| Single giant monolith package with no domain boundaries | Slows team velocity and raises blast radius for integration changes | Monorepo with clear package/service boundaries and contract enforcement |
| DIY auth/session implementation | High security/compliance risk and avoidable maintenance burden | Enterprise auth platform (Auth.js + provider stack or WorkOS) |
| Unbounded vector-only architecture for all intelligence | Costly, noisy retrieval and weak deterministic governance reporting | Relational-first model + targeted vector retrieval for copilot context |

## Stack Patterns by Variant

**If v1 is control-plane-first (inventory + health dashboards, limited automated actions):**
- Use Next.js + Node API + PostgreSQL + Temporal + OpenAI Responses API with read-heavy copilot tools
- Because this maximizes delivery speed for visibility/governance while keeping remediation actions gated

**If v1.5 adds active remediation orchestration:**
- Expand Temporal workflows, introduce stricter policy engine checks, and add event-driven remediation pipelines on Kafka
- Because enterprise trust requires deterministic execution, policy compliance, and full auditability before autonomous changes

**If large-enterprise procurement requires strict tenant isolation:**
- Use dedicated database-per-tenant or logical isolation + per-tenant encryption keys and stronger runtime isolation for connectors
- Because security/legal requirements may outweigh operational simplicity at larger ACV tiers

**If AI copilot scope expands to deep RCA and proactive recommendations:**
- Add curated incident memory, retrieval pipelines, and evaluation harnesses with human-in-the-loop scoring
- Because copilot quality in ops contexts depends on measured reliability, not just model capability

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| Next.js 16.x | React 19.x | Use app router defaults; validate third-party component compatibility |
| Prisma 6.x | PostgreSQL 17.x | Stable primary pairing for typed schema + migrations |
| Temporal 1.27.x (server/cloud) | Node SDK latest stable | Keep SDK/server compatibility matrix pinned in CI |
| OpenTelemetry SDK 1.30.x | Node.js 22 LTS | Prefer OTLP exporter and consistent resource attributes across services |
| pgvector 0.8.x | PostgreSQL 17.x | Validate extension availability in managed PostgreSQL provider |
| TanStack Query 5.x | React 19.x | Align suspense and caching strategy with Next.js server/client boundaries |

## Sources

- Internal project context (`/Users/dev/Documents/Software/Web/Sysilo/.planning/PROJECT.md`) — product direction, constraints, and v1 scope alignment
- GSD stack template (`/Users/dev/.codex/get-shit-done/templates/research-project/STACK.md`) — required structure and output format
- Industry/practitioner inference (2026 architecture patterns) — confidence based on stable enterprise SaaS and integration-platform practices

---
*Stack research for: Enterprise integration intelligence + AI integration copilot*
*Researched: 2026-02-28*

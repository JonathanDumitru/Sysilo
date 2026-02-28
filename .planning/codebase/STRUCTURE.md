# Codebase Structure

**Analysis Date:** 2026-02-28

## Directory Layout

```
sysilo/
├── agent/                          # On-premise agent binary (Go)
│   ├── cmd/agent/                  # Entry point (main.go)
│   ├── internal/
│   │   ├── adapters/               # Pluggable task handlers
│   │   │   ├── discovery/          # Asset discovery adapter
│   │   │   ├── playbook/           # Playbook step handler
│   │   │   └── postgresql/         # PostgreSQL query adapter
│   │   ├── config/                 # Config loading
│   │   ├── executor/               # Core task executor + TaskHandler interface
│   │   ├── health/                 # Health check server
│   │   └── tunnel/                 # gRPC client for agent-gateway
│   └── pkg/
│       ├── logging/                # Logger factory
│       └── version/                # Build version info
├── connectors/                     # Connector definitions (empty stubs)
│   ├── databases/
│   ├── protocols/
│   └── saas/
├── docs/                           # Project documentation
│   ├── api/                        # API reference docs
│   ├── architecture/               # Architecture diagrams and ADRs
│   ├── decisions/                  # Architectural decision records
│   ├── development/                # Onboarding and dev guides
│   ├── integration/                # Integration Studio + Connectors SDK docs
│   └── diagrams/                   # PNG diagrams
├── infra/
│   ├── docker/                     # docker-compose.yml for local dev stack
│   ├── kubernetes/                 # K8s manifests
│   └── terraform/                  # Infrastructure as code
├── packages/
│   ├── frontend/
│   │   ├── design-system/          # Shared design tokens (stub)
│   │   ├── ui-components/          # Shared component library (stub)
│   │   └── web-app/                # React SPA
│   │       └── src/
│   │           ├── components/     # UI components by domain
│   │           │   ├── agents/
│   │           │   ├── ai/         # AI chat panel, recommendation cards
│   │           │   ├── billing/    # Plan badge, upgrade modal, usage meter
│   │           │   ├── common/     # Shared/generic components
│   │           │   ├── graph/      # Asset graph visualization
│   │           │   ├── integrations/
│   │           │   ├── layout/     # AppLayout, Header, Sidebar
│   │           │   ├── playbooks/  # Playbook editor nodes
│   │           │   └── studio/     # Integration Studio nodes and panels
│   │           ├── hooks/          # Data-fetching React hooks
│   │           ├── pages/          # Route-level page components
│   │           ├── services/       # API client modules
│   │           ├── store/          # (empty — no global state manager)
│   │           └── types/          # Shared TypeScript types
│   └── sdk/
│       ├── python/                 # Python connector SDK (stub)
│       └── typescript/
│           └── src/
│               ├── connector.ts    # BaseConnector, ConnectorRegistry
│               ├── types.ts        # Shared type definitions
│               ├── testing.ts      # Test helpers for connectors
│               └── index.ts        # Public SDK exports
├── proto/
│   └── agent/v1/
│       └── agent.proto             # gRPC contract: AgentService, Task, TaskResult, etc.
├── schemas/
│   ├── postgres/                   # SQL migration files (001–009)
│   ├── kafka/                      # Kafka topic schemas
│   └── neo4j/                      # Neo4j constraint scripts
├── scripts/                        # Utility shell scripts
├── services/
│   ├── agent-gateway/              # gRPC server for agent connections (Go)
│   │   ├── cmd/agent-gateway/      # Entry point
│   │   └── internal/
│   │       ├── auth/
│   │       ├── config/
│   │       ├── kafka/              # Kafka producer/consumer
│   │       ├── registry/           # In-memory agent registry
│   │       └── tunnel/             # gRPC bidirectional stream server
│   ├── ai-service/                 # AI/LLM service (Python/FastAPI)
│   │   └── src/ai_service/
│   │       ├── api/                # Route handlers (chat, embeddings, insights, recommendations)
│   │       └── llm/                # LLM client abstraction + prompts
│   ├── api-gateway/                # External REST API (Go/chi)
│   │   ├── cmd/api-gateway/        # Entry point
│   │   └── internal/
│   │       ├── auth/
│   │       ├── config/
│   │       ├── db/                 # Repository pattern: AgentRepo, ConnectionRepo, etc.
│   │       ├── handlers/           # HTTP handler functions
│   │       └── middleware/         # Auth, CORS, tenant, plan gate, rate limit
│   ├── asset-service/              # Asset registry + graph (Rust/axum)
│   │   └── src/
│   │       ├── api/                # HTTP handlers
│   │       ├── assets/             # Asset CRUD (PostgreSQL)
│   │       ├── graph/              # Graph queries (Neo4j)
│   │       ├── impact/             # Impact analysis
│   │       └── relationships/      # Relationship CRUD (Neo4j)
│   ├── data-service/               # Data Hub: catalog, lineage, quality (Rust/axum)
│   │   └── src/
│   │       ├── api/
│   │       ├── catalog/
│   │       ├── ingestion/
│   │       ├── lineage/
│   │       └── quality/
│   ├── governance-service/         # Governance: policies, standards, approvals, audit (Rust/axum)
│   │   └── src/
│   │       ├── api/
│   │       ├── approvals/
│   │       ├── audit/
│   │       ├── compliance/
│   │       ├── kafka/
│   │       ├── policies/
│   │       └── standards/
│   ├── integration-service/        # Integration + playbook orchestration (Rust/axum)
│   │   ├── migrations/             # sqlx migration files
│   │   └── src/
│   │       ├── api/                # HTTP handlers for integrations/runs/discovery
│   │       ├── config/
│   │       ├── connections/        # Connection management
│   │       ├── consumer/           # Kafka result consumer
│   │       ├── engine/             # Task dispatch engine
│   │       ├── kafka/              # Kafka producer
│   │       ├── middleware/         # Tenant context middleware
│   │       ├── playbooks/          # Playbook CRUD, executor, result handler
│   │       └── storage/            # PostgreSQL queries
│   ├── ops-service/                # Operations: metrics, alerts, incidents (Rust/axum)
│   │   └── src/
│   │       ├── alerts/
│   │       ├── api/
│   │       ├── incidents/
│   │       ├── metrics/
│   │       └── notifications/
│   └── rationalization-service/    # App rationalization: scoring, scenarios (Rust/axum)
│       └── src/
│           ├── api/
│           ├── playbooks/
│           ├── recommendations/
│           ├── scenarios/
│           └── scoring/
├── website/                        # Marketing website (separate from app)
├── Makefile                        # Build, test, run, and dev-env targets
└── README.md
```

## Directory Purposes

**`agent/`:**
- Purpose: Customer-deployed binary that runs inside private networks
- Contains: gRPC tunnel client, pluggable executor, data-source adapters
- Key files: `agent/cmd/agent/main.go`, `agent/internal/executor/executor.go`, `agent/internal/adapters/postgresql/`

**`services/`:**
- Purpose: All cloud-side microservices
- Contains: 9 independent services, each with its own language, binary, and `go.mod`/`Cargo.toml`
- Key service ports: api-gateway (configured), agent-gateway (gRPC), integration-service `8082`, data-service `8083`, asset-service `8084`, ops-service `8085`, governance-service `8086`, rationalization-service `8087`, ai-service (FastAPI default)

**`packages/frontend/web-app/`:**
- Purpose: The primary user-facing React application
- Contains: All UI code; page-per-route pattern
- Key files: `src/App.tsx` (routing), `src/main.tsx` (entry), `src/services/api.ts` (base API client)

**`packages/sdk/typescript/`:**
- Purpose: Public SDK for building custom connectors
- Contains: `BaseConnector` abstract class, `ConnectorRegistry`, Zod-validated config, test helpers
- Key files: `src/connector.ts`, `src/types.ts`, `src/index.ts`

**`proto/`:**
- Purpose: Protocol Buffer definitions shared between agent and agent-gateway
- Contains: `agent.proto` defining `AgentService` gRPC contract
- Key files: `proto/agent/v1/agent.proto`

**`schemas/`:**
- Purpose: Database schema definitions (not ORM migrations — raw SQL/Cypher)
- Contains: PostgreSQL SQL files (`001`–`009`), Neo4j Cypher constraints, Kafka topic schemas
- Key files: `schemas/postgres/001_initial_schema.sql` (primary schema)

**`infra/docker/`:**
- Purpose: Local development infrastructure
- Contains: `docker-compose.yml` spinning up PostgreSQL, Neo4j, Redis, Kafka, MinIO, Kafka UI

## Key File Locations

**Entry Points:**
- `agent/cmd/agent/main.go`: Agent binary start
- `services/agent-gateway/cmd/agent-gateway/main.go`: Agent gateway start
- `services/api-gateway/cmd/api-gateway/main.go`: REST API start
- `services/integration-service/src/main.rs`: Integration service start
- `services/asset-service/src/main.rs`: Asset service start
- `services/governance-service/src/main.rs`: Governance service start
- `services/ops-service/src/main.rs`: Ops service start
- `services/rationalization-service/src/main.rs`: Rationalization service start
- `services/data-service/src/main.rs`: Data hub service start
- `services/ai-service/src/ai_service/main.py`: AI service start
- `packages/frontend/web-app/src/main.tsx`: Frontend React entry

**Configuration:**
- `Makefile`: Top-level build, test, run, and dev-env orchestration
- `infra/docker/docker-compose.yml`: Local dev stack (all infrastructure)
- `schemas/postgres/001_initial_schema.sql`: Primary PostgreSQL schema
- `proto/agent/v1/agent.proto`: Agent/gateway gRPC contract

**Core Logic:**
- `agent/internal/executor/executor.go`: Task execution engine and `TaskHandler` interface
- `services/agent-gateway/internal/tunnel/server.go`: gRPC stream management and Kafka forwarding
- `services/api-gateway/internal/handlers/handlers.go`: All REST handler functions
- `services/api-gateway/internal/middleware/middleware.go`: Auth, tenant, plan enforcement chain
- `services/api-gateway/internal/middleware/plan_gate.go`: Feature gating middleware
- `services/integration-service/src/playbooks/executor.rs`: Playbook DAG executor
- `services/integration-service/src/playbooks/result_handler.rs`: Step result routing and chaining
- `services/integration-service/src/consumer/mod.rs`: Kafka result consumer

**Frontend:**
- `packages/frontend/web-app/src/App.tsx`: Route definitions
- `packages/frontend/web-app/src/services/api.ts`: Base `apiFetch()` helper
- `packages/frontend/web-app/src/services/playbooks.ts`: Playbook API client
- `packages/frontend/web-app/src/hooks/`: Domain-specific data hooks

## Naming Conventions

**Files:**
- Go: `snake_case.go` (e.g., `plan_gate.go`, `billing_handlers.go`)
- Rust: `mod.rs` inside a directory module (e.g., `playbooks/mod.rs`, `consumer/mod.rs`)
- TypeScript/TSX: `PascalCase.tsx` for components (e.g., `AppLayout.tsx`), `camelCase.ts` for services/hooks (e.g., `usePlaybooks.ts`, `api.ts`)
- Python: `snake_case.py` (e.g., `main.py`, `clients.py`)
- Proto: `snake_case.proto` matching package name

**Directories:**
- Services: `kebab-case` (e.g., `agent-gateway`, `integration-service`)
- Go internal packages: `lowercase` single word (e.g., `handlers`, `middleware`, `tunnel`)
- Rust modules: `lowercase` single word (e.g., `playbooks`, `consumer`, `storage`)
- Frontend components: `PascalCase` by domain (e.g., `components/billing/`, `components/playbooks/`)
- Frontend pages: `PascalCasePage.tsx` convention

## Where to Add New Code

**New Agent Adapter (new data source type):**
- Implementation: `agent/internal/adapters/{adapter-name}/adapter.go`
- Register in: `agent/cmd/agent/main.go` via `exec.RegisterHandler(newadapter.NewAdapter(logger))`
- Implement: `Type() string` + `Execute(ctx, *Task) (*TaskResult, error)`

**New API Route (api-gateway):**
- Handler: `services/api-gateway/internal/handlers/handlers.go` or a new file like `services/api-gateway/internal/handlers/{domain}_handlers.go`
- Route registration: `services/api-gateway/cmd/api-gateway/main.go` inside the `r.Route("/api/v1", ...)` block

**New Integration Service Endpoint:**
- Handler: `services/integration-service/src/api/mod.rs` or domain sub-module
- Route registration: `services/integration-service/src/main.rs` in `protected_routes`

**New Domain Service:**
- Create a new directory under `services/{service-name}/`
- Follow Rust/axum pattern: `src/main.rs` with `AppState`, `src/api/mod.rs` for handlers, `src/{domain}/mod.rs` for logic
- Add a SQL migration file to `schemas/postgres/` following the numbered sequence

**New Frontend Page:**
- Page component: `packages/frontend/web-app/src/pages/{DomainName}Page.tsx`
- Route: Add to `packages/frontend/web-app/src/App.tsx` inside `<Route path="/" element={<AppLayout />}>`
- Data hook: `packages/frontend/web-app/src/hooks/use{Domain}.ts`
- API service: `packages/frontend/web-app/src/services/{domain}.ts`

**New Frontend Component:**
- Component: `packages/frontend/web-app/src/components/{domain}/{ComponentName}.tsx`
- Re-export from: `packages/frontend/web-app/src/components/{domain}/index.ts` if barrel exists

**New Playbook Step Type:**
- Backend step type enum: `services/integration-service/src/playbooks/mod.rs`
- Agent handler: `agent/internal/adapters/playbook/` step routing logic
- Frontend node: `packages/frontend/web-app/src/components/playbooks/nodes/{TypeName}StepNode.tsx`

**New Proto Message/RPC:**
- Edit: `proto/agent/v1/agent.proto`
- Regenerate: `make proto`
- Update agent tunnel client: `agent/internal/tunnel/`
- Update gateway tunnel server: `services/agent-gateway/internal/tunnel/server.go`

## Special Directories

**`proto/gen/` (generated, not committed):**
- Purpose: Generated Go gRPC bindings from `agent.proto`
- Generated: Yes (`make proto`)
- Committed: No (in .gitignore)

**`services/*/target/` (Rust build artifacts):**
- Purpose: Cargo build output
- Generated: Yes
- Committed: No

**`packages/frontend/web-app/dist/`:**
- Purpose: Vite production build output
- Generated: Yes
- Committed: No (present in repo currently — likely unintentional)

**`bin/` (build output):**
- Purpose: Compiled Go and Rust binaries
- Generated: Yes (`make build`)
- Committed: No

**`.planning/codebase/`:**
- Purpose: GSD codebase analysis documents consumed by planning/execution agents
- Generated: Yes (by map-codebase commands)
- Committed: Yes

---

*Structure analysis: 2026-02-28*

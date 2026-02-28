# Architecture

**Analysis Date:** 2026-02-28

## Pattern Overview

**Overall:** Event-driven polyglot microservices platform

**Key Characteristics:**
- Multi-language microservices: Go for gateways/control-plane, Rust for domain services, Python for AI, TypeScript for frontend
- Agents deployed on-premise connect to the cloud control plane via a persistent bidirectional gRPC stream (reverse tunnel)
- Kafka is the async backbone for task dispatch, result aggregation, and inter-service event propagation
- All domain services share a single PostgreSQL instance with schema-separated tables; asset-service also uses Neo4j for graph relationships
- Multi-tenant: every data operation is scoped by `tenant_id` extracted from JWT claims

## Layers

**Frontend (React SPA):**
- Purpose: Single-page web application serving the full platform UI
- Location: `packages/frontend/web-app/src/`
- Contains: Pages, components, hooks, service clients
- Depends on: Integration-service HTTP API at `VITE_API_URL` (default `http://localhost:8082`)
- Used by: End users via browser

**API Gateway (Go/chi):**
- Purpose: External HTTP REST gateway; JWT auth, tenant context, feature gating, billing enforcement
- Location: `services/api-gateway/`
- Contains: Middleware chain (Auth → TenantContext → LoadTenantPlan → PlanGate → RateLimit), handlers, DB repository layer
- Depends on: PostgreSQL (via `services/api-gateway/internal/db/`)
- Used by: Frontend, third-party API consumers

**Integration Service (Rust/axum):**
- Purpose: Integration and playbook orchestration; manages connections, integration definitions, playbook runs, dispatches tasks to agents via Kafka
- Location: `services/integration-service/src/`
- Contains: REST API, execution engine (`engine/mod.rs`), Kafka task producer, result consumer, playbook executor
- Depends on: PostgreSQL, Kafka
- Used by: Frontend (directly), API Gateway (proxied routes at `/integrations/*`)

**Agent Gateway (Go/gRPC):**
- Purpose: Control-plane endpoint for on-premise agents; maintains persistent bidirectional gRPC streams; forwards task results to Kafka
- Location: `services/agent-gateway/`
- Contains: gRPC tunnel server, agent registry, Kafka producer
- Depends on: Kafka
- Used by: Agents connecting from customer environments

**Agent (Go):**
- Purpose: Runs inside customer networks; receives tasks from agent-gateway and executes them against local data sources
- Location: `agent/`
- Contains: Executor with pluggable handler interface, adapters (PostgreSQL, discovery, playbook), tunnel gRPC client
- Depends on: Agent-gateway (outbound gRPC connection initiated by agent)
- Used by: Agent-gateway (pushes tasks via stream)

**Domain Services (Rust/axum):**
- Purpose: Bounded-context services for each platform capability domain
- Location: `services/{asset-service,data-service,governance-service,ops-service,rationalization-service}/`
- Contains: Domain logic modules, REST API handlers, service state
- Depends on: PostgreSQL; asset-service also depends on Neo4j
- Used by: Frontend, integration-service result consumer (asset-service)

**AI Service (Python/FastAPI):**
- Purpose: Conversational AI, embeddings, recommendations, insights
- Location: `services/ai-service/src/ai_service/`
- Contains: Chat, embeddings, recommendations, insights routers; LLM client abstraction
- Depends on: PostgreSQL, LLM provider (configured via env)
- Used by: Frontend (AI assistant panel), rationalization-service

## Data Flow

**Task Execution (Integration Run):**

1. Frontend calls `POST /integrations/{id}/run` → integration-service
2. Integration-service creates a run record in PostgreSQL, dispatches `Task` message to Kafka topic `tasks`
3. Agent-gateway (Kafka consumer) receives the task and routes it to the appropriate connected agent via its in-memory gRPC stream map
4. Agent executor finds the registered handler (e.g., `postgresql`) and executes the task against the target system
5. Agent sends `TaskResult` back over the gRPC stream
6. Agent-gateway publishes the result to Kafka topic `results`
7. Integration-service result consumer receives the result, updates run status in PostgreSQL, and (for discovery tasks) calls asset-service HTTP API to persist discovered assets

**Playbook Execution:**

1. Frontend calls `POST /integrations/playbooks/{id}/run` → integration-service
2. `PlaybookExecutor::start_run` identifies starting steps (no incoming edges) and dispatches each as a `playbook_step` task to Kafka
3. Each step result flows back via the same Kafka results pipeline
4. `PlaybookResultHandler` in integration-service processes each step result, updates step state in PostgreSQL, dispatches successor steps (`on_success`/`on_failure` edges), or marks run complete
5. Approval steps pause execution; frontend polls or receives status, then calls `/playbook-runs/{id}/approve` or `/reject`

**Agent Connection (Reverse Tunnel):**

1. Agent process starts, calls `tunnel.NewClient` and connects outbound to agent-gateway gRPC address
2. Agent sends `AgentRegistration` message (first message on stream)
3. Agent-gateway registers agent in in-memory registry, sends `RegistrationAck`
4. Persistent bidirectional stream remains open; agent sends periodic heartbeats; gateway pushes tasks

**State Management (Frontend):**
- No global state manager detected (no Redux/Zustand store files found)
- State managed via React hooks (`usePlaybooks.ts`, `useConnections.ts`, `useBilling.ts`, etc.) in `packages/frontend/web-app/src/hooks/`
- API calls made through service modules in `packages/frontend/web-app/src/services/`

## Key Abstractions

**TaskHandler (Agent):**
- Purpose: Pluggable interface for executing a specific type of task on the agent
- Examples: `agent/internal/adapters/postgresql/`, `agent/internal/adapters/discovery/`, `agent/internal/adapters/playbook/`
- Pattern: Implement `Type() string` and `Execute(ctx, *Task) (*TaskResult, error)`; register via `executor.RegisterHandler()`

**PlaybookExecutor (Integration-service):**
- Purpose: DAG-based step dispatcher; finds root steps (no incoming edges), dispatches via Kafka, handles step results and chains next steps
- Examples: `services/integration-service/src/playbooks/executor.rs`, `services/integration-service/src/playbooks/result_handler.rs`
- Pattern: Stateless struct with static methods; receives producer and storage references

**DB Repository (API Gateway):**
- Purpose: Repository pattern wrapping raw SQL; each entity type has its own struct (`AgentRepository`, `ConnectionRepository`, etc.)
- Examples: `services/api-gateway/internal/db/db.go`, `services/api-gateway/internal/db/agents.go`
- Pattern: `DB` struct aggregates all repositories; handlers access via `h.DB.Agents`, `h.DB.Connections`, etc.

**AppState (Rust Services):**
- Purpose: Axum shared application state passed to all handlers via `with_state()`
- Examples: `services/integration-service/src/main.rs`, `services/asset-service/src/main.rs`, `services/governance-service/src/main.rs`
- Pattern: `pub struct AppState { pub service_a: ServiceA, pub service_b: ServiceB, ... }` wrapped in `Arc<AppState>`

**Connector SDK:**
- Purpose: TypeScript base classes for building third-party connectors
- Examples: `packages/sdk/typescript/src/connector.ts`
- Pattern: Implement `BaseConnector` abstract class, override `onInitialize()` and `healthCheck()`; register with `ConnectorRegistry`

## Entry Points

**API Gateway:**
- Location: `services/api-gateway/cmd/api-gateway/main.go`
- Triggers: Process start; binds HTTP on `cfg.Server.Address`
- Responsibilities: Route registration, middleware chain setup, database init, graceful shutdown

**Agent Gateway:**
- Location: `services/agent-gateway/cmd/agent-gateway/main.go`
- Triggers: Process start; binds gRPC server
- Responsibilities: gRPC server setup, Kafka producer init, agent registry init, tunnel server registration

**Integration Service:**
- Location: `services/integration-service/src/main.rs`
- Triggers: Process start (`tokio::main`)
- Responsibilities: Storage init, execution engine init, result consumer spawn (background tokio task), Axum server bind

**Agent:**
- Location: `agent/cmd/agent/main.go`
- Triggers: Process start; connects outbound to agent-gateway
- Responsibilities: Executor init, handler registration (postgresql, discovery, playbook), tunnel client connect, signal handling

**AI Service:**
- Location: `services/ai-service/src/ai_service/main.py`
- Triggers: `uvicorn` process start
- Responsibilities: FastAPI app creation, LLM client init, DB init, router registration

**Frontend:**
- Location: `packages/frontend/web-app/src/main.tsx`
- Triggers: Browser load
- Responsibilities: React root mount, React Router setup

## Error Handling

**Strategy:** Inline error propagation; no shared error middleware framework across services

**Patterns:**
- Go (api-gateway): Return `nil` + typed error from DB layer; handlers check `err == sql.ErrNoRows` for 404, other errors for 500; respond via `respondError(w, status, message)`
- Rust (domain services): `anyhow::Result` for startup; `thiserror::Error` for domain-specific errors (e.g., `ExecutorError`); Axum handlers return `impl IntoResponse`
- Agent executor: Explicit result types `TaskStatus{Timeout, Cancelled, Failed, Completed}` with `Retryable` flag on `TaskError`
- Frontend: `ApiError` class extends `Error` with `status` field; thrown from `apiFetch()` in `packages/frontend/web-app/src/services/api.ts`

## Cross-Cutting Concerns

**Logging:**
- Go services: `go.uber.org/zap` structured JSON logging
- Rust services: `tracing` crate with `tracing_subscriber` JSON format
- Python AI service: `structlog` with JSON renderer
- All services log JSON to stdout

**Validation:**
- API Gateway: Inline field checks in handlers (e.g., `req.Name == ""`)
- Integration Service: Rust type system + `serde` deserialization
- Connector SDK: Zod schema validation via `metadata.configSchema.safeParse()`

**Authentication:**
- API Gateway: JWT Bearer token validation (HMAC) via `middleware.Auth()`; claims carry `sub` (user ID), `tenant_id`, `roles`
- Agent Gateway: Agent identifies via `AgentRegistration` message; mTLS noted as TODO for production
- Integration Service: Optional tenant context middleware (`optional_tenant_context_middleware`) — strict enforcement is a TODO

**Feature Gating:**
- API Gateway: `middleware.PlanGate` reads `PlanInfo` from request context (loaded by `middleware.LoadTenantPlan`) to enforce tier-based feature access
- Plan data stored in `plans` and `tenant_plans` PostgreSQL tables; `services/api-gateway/internal/db/plans.go`

---

*Architecture analysis: 2026-02-28*

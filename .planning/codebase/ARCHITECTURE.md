# Architecture

**Analysis Date:** 2026-02-28

## System Style
- Sysilo is a polyglot microservices platform split across Go, Rust, Python, and TypeScript.
- Control-plane ingress is split between HTTP (`services/api-gateway/cmd/api-gateway/main.go`) and gRPC (`services/agent-gateway/cmd/agent-gateway/main.go`).
- Domain services use Rust + Axum entrypoints such as `services/integration-service/src/main.rs` and `services/asset-service/src/main.rs`.
- End-user UI is a React SPA rooted at `packages/frontend/web-app/src/main.tsx` and route tree in `packages/frontend/web-app/src/App.tsx`.
- Agent-side execution runs on customer infrastructure via `agent/cmd/agent/main.go`.

## Runtime Boundaries
- **Frontend boundary:** Browser app calls HTTP APIs through `packages/frontend/web-app/src/services/api.ts` (default `VITE_API_URL` is `http://localhost:8082`).
- **Gateway boundary:** `services/api-gateway/` handles JWT auth, tenant context, plan loading, and feature gating using middleware in `services/api-gateway/internal/middleware/middleware.go` and `services/api-gateway/internal/middleware/plan_gate.go`.
- **Orchestration boundary:** `services/integration-service/` owns integration runs and playbook DAG orchestration (`services/integration-service/src/playbooks/executor.rs`, `services/integration-service/src/playbooks/result_handler.rs`).
- **Agent tunnel boundary:** `services/agent-gateway/internal/tunnel/server.go` keeps long-lived agent streams and forwards results/logs to Kafka.
- **Execution boundary:** `agent/internal/executor/executor.go` routes incoming tasks to adapters under `agent/internal/adapters/`.

## Primary Data Flow
1. UI triggers integration/playbook actions from pages under `packages/frontend/web-app/src/pages/`.
2. Requests hit integration endpoints declared in `services/integration-service/src/main.rs`.
3. Integration service emits work to Kafka via its producer modules in `services/integration-service/src/kafka/`.
4. Agent gateway consumes/distributes tasks via stream handling in `services/agent-gateway/internal/tunnel/server.go`.
5. Agent executes task handlers registered in `agent/cmd/agent/main.go` (`postgresql`, `discovery`, `playbook`).
6. Agent returns `TaskResult` over gRPC contract defined in `proto/agent/v1/agent.proto`.
7. Result consumer in `services/integration-service/src/consumer/mod.rs` updates run state and may call asset-service.

## Service Responsibilities
- `services/api-gateway/`: REST edge, authn/authz, tenant scoping, billing/plan enforcement, repository-backed DB access (`services/api-gateway/internal/db/`).
- `services/integration-service/`: connections, integrations, runs, discovery dispatch, playbook CRUD + execution (`services/integration-service/src/connections/`, `services/integration-service/src/playbooks/`, `services/integration-service/src/storage/`).
- `services/agent-gateway/`: agent registration, stream lifecycle, heartbeat tracking, Kafka forwarding (`services/agent-gateway/internal/registry/`, `services/agent-gateway/internal/kafka/`).
- `services/asset-service/`: asset registry + graph relationships (`services/asset-service/src/assets/`, `services/asset-service/src/relationships/`, `services/asset-service/src/graph/`).
- `services/data-service/`, `services/governance-service/`, `services/ops-service/`, `services/rationalization-service/`: bounded contexts for data, governance, operations, and rationalization domains.
- `services/ai-service/src/ai_service/main.py`: FastAPI app for chat, embeddings, recommendations, and insights.

## Storage and Messaging
- PostgreSQL schema scripts live in `schemas/postgres/`.
- Neo4j constraints live in `schemas/neo4j/001_constraints.cypher`.
- Kafka is core async transport; local stack config is in `infra/docker/docker-compose.yml`.
- Local infra includes PostgreSQL, Neo4j, Redis, Kafka, MinIO, Kafka UI in `infra/docker/docker-compose.yml`.

## Shared Contracts and SDKs
- Agent/gateway protocol is versioned in `proto/agent/v1/agent.proto`.
- TypeScript connector SDK primitives are in `packages/sdk/typescript/src/connector.ts` and `packages/sdk/typescript/src/types.ts`.

## Cross-Cutting Concerns
- Logging: Go uses Zap (`services/api-gateway/cmd/api-gateway/main.go`, `services/agent-gateway/cmd/agent-gateway/main.go`), Rust uses `tracing` (`services/integration-service/src/main.rs`), Python uses `structlog` (`services/ai-service/src/ai_service/main.py`).
- Multi-tenancy: tenant context enforced in gateway middleware (`services/api-gateway/internal/middleware/middleware.go`) and carried through integration records/storage.
- Feature gates: route-prefix feature checks live in `services/api-gateway/internal/middleware/plan_gate.go`.

## Practical Entry Points
- Frontend boot: `packages/frontend/web-app/src/main.tsx`
- API Gateway boot: `services/api-gateway/cmd/api-gateway/main.go`
- Agent Gateway boot: `services/agent-gateway/cmd/agent-gateway/main.go`
- Integration boot: `services/integration-service/src/main.rs`
- Agent boot: `agent/cmd/agent/main.go`
- AI Service boot: `services/ai-service/src/ai_service/main.py`

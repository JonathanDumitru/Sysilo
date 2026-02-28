# Sysilo Technology Stack

Analysis scope: repository state at `/Users/dev/Documents/Software/Web/Sysilo`.

## 1) Languages and service distribution
- Go 1.22 for control-plane edge services and agent runtime (`services/api-gateway/go.mod`, `services/agent-gateway/go.mod`, `agent/go.mod`).
- Rust (edition 2021) for domain services (`services/asset-service/Cargo.toml`, `services/data-service/Cargo.toml`, `services/integration-service/Cargo.toml`, `services/ops-service/Cargo.toml`, `services/governance-service/Cargo.toml`, `services/rationalization-service/Cargo.toml`).
- Python 3.11+ for AI engine (`services/ai-service/pyproject.toml`).
- TypeScript for frontend and connector SDK (`packages/frontend/web-app/package.json`, `packages/sdk/typescript/package.json`).
- SQL/Cypher schema assets (`schemas/postgres/*.sql`, `schemas/neo4j/001_constraints.cypher`).
- Protobuf for agent protocol contracts (`proto/agent/v1/agent.proto`).

## 2) Backend frameworks and runtime libraries
- Go HTTP routing/middleware: Chi (`services/api-gateway/go.mod`).
- Go infra libraries: JWT, Redis client, Zap (`services/api-gateway/go.mod`).
- Rust web stack: Axum + Tower + Tokio in all Rust services (`services/*/Cargo.toml`).
- Rust DB layer: SQLx/PostgreSQL broadly; Neo4rs specifically in asset-service (`services/asset-service/Cargo.toml`).
- Rust eventing: rdkafka used by integration/data/governance services (`services/integration-service/Cargo.toml`, `services/data-service/Cargo.toml`, `services/governance-service/Cargo.toml`).
- Python API stack: FastAPI + Uvicorn + Pydantic settings (`services/ai-service/pyproject.toml`, `services/ai-service/src/ai_service/config.py`).

## 3) Frontend stack
- React 18 app bootstrapped with Vite (`packages/frontend/web-app/package.json`, `packages/frontend/web-app/vite.config.ts`, `packages/frontend/web-app/src/main.tsx`).
- Routing/forms/data fetching: `react-router-dom`, `react-hook-form`, `@tanstack/react-query` (`packages/frontend/web-app/package.json`).
- Graph/node editors: `@xyflow/react` and related components (`packages/frontend/web-app/src/components/studio/*`, `packages/frontend/web-app/src/components/playbooks/*`).
- Styling toolchain: Tailwind + PostCSS (`packages/frontend/web-app/tailwind.config.js`, `packages/frontend/web-app/postcss.config.js`, `packages/frontend/web-app/src/index.css`).

## 4) Data and platform infrastructure
- PostgreSQL is primary relational store (`infra/docker/docker-compose.yml`, `schemas/postgres/001_initial_schema.sql`).
- Neo4j is graph store for asset relationships (`infra/docker/docker-compose.yml`, `services/asset-service/src/graph/mod.rs`).
- Redis is cache/rate-limit backing (`infra/docker/docker-compose.yml`, `services/api-gateway/go.mod`, `services/ai-service/src/ai_service/config.py`).
- Kafka is event/task bus (`infra/docker/docker-compose.yml`, `services/agent-gateway/internal/kafka/consumer.go`, `services/integration-service/src/kafka/mod.rs`).
- MinIO is provisioned as S3-compatible object storage for local infra (`infra/docker/docker-compose.yml`).

## 5) Interface and protocol stack
- External client entrypoint is REST over API Gateway handlers (`services/api-gateway/internal/handlers/handlers.go`).
- Agent connectivity is gRPC bi-directional streaming (`proto/agent/v1/agent.proto`, `services/agent-gateway/internal/tunnel/server.go`).
- Async inter-service transport is Kafka topic messaging (`services/agent-gateway/internal/kafka/producer.go`, `services/integration-service/src/playbooks/result_handler.rs`).

## 6) Build/test/tooling
- Root automation uses GNU Make (`Makefile`).
- Go modules for dependency management (`agent/go.mod`, `services/api-gateway/go.mod`, `services/agent-gateway/go.mod`).
- Cargo for Rust services (`services/*/Cargo.toml`).
- Pyproject/Hatch for AI service packaging (`services/ai-service/pyproject.toml`).
- npm for web app and TypeScript SDK (`packages/frontend/web-app/package-lock.json`, `packages/sdk/typescript/package.json`).
- Protobuf code generation wired in make target `proto` (`Makefile`, `proto/agent/v1/agent.proto`).

## 7) Practical stack notes
- This is a polyglot monorepo with clear service boundaries by language under `services/` and `agent/`.
- `connectors/` currently contains no checked-in connector implementation files; connector API surface exists via SDK (`packages/sdk/typescript/src/*`) and integration service APIs (`services/integration-service/src/connections/api.rs`).
- Local development baseline is containerized dependencies plus host-run binaries (`Makefile`, `infra/docker/docker-compose.yml`).

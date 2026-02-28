# Codebase Structure

**Analysis Date:** 2026-02-28

## Top-Level Layout
- `agent/`: On-prem executable and adapters for local task execution.
- `services/`: Cloud-side microservices (gateway, orchestration, domain services, AI service).
- `packages/frontend/web-app/`: Main React application used by platform users.
- `packages/sdk/typescript/`: Connector SDK for external integration authors.
- `proto/`: Shared protobuf contracts, especially `proto/agent/v1/agent.proto`.
- `schemas/`: SQL/Cypher/Kafka schemas used by platform persistence and messaging.
- `infra/docker/`: Local development infra stack in `infra/docker/docker-compose.yml`.
- `docs/`: Product, architecture, deployment, and implementation notes.
- `.planning/codebase/`: Planning docs including this file and `ARCHITECTURE.md`.

## Service Tree (Practical)
- `services/api-gateway/`
- `services/agent-gateway/`
- `services/integration-service/`
- `services/asset-service/`
- `services/data-service/`
- `services/governance-service/`
- `services/ops-service/`
- `services/rationalization-service/`
- `services/ai-service/`

## High-Value Entry Files
- `agent/cmd/agent/main.go`
- `services/agent-gateway/cmd/agent-gateway/main.go`
- `services/api-gateway/cmd/api-gateway/main.go`
- `services/integration-service/src/main.rs`
- `services/asset-service/src/main.rs`
- `services/data-service/src/main.rs`
- `services/governance-service/src/main.rs`
- `services/ops-service/src/main.rs`
- `services/rationalization-service/src/main.rs`
- `services/ai-service/src/ai_service/main.py`
- `packages/frontend/web-app/src/main.tsx`
- `packages/frontend/web-app/src/App.tsx`

## Internal Organization Patterns
- Go services follow `cmd/` + `internal/` layout (example: `services/api-gateway/internal/handlers/`, `services/agent-gateway/internal/tunnel/`).
- Rust services group domain code in `src/<domain>/mod.rs` plus API surface in `src/api/mod.rs`.
- Frontend uses domain folders under `packages/frontend/web-app/src/components/`, route pages under `packages/frontend/web-app/src/pages/`, data access in `packages/frontend/web-app/src/services/`, and query hooks in `packages/frontend/web-app/src/hooks/`.
- Agent adapters are plugin-like units under `agent/internal/adapters/` registered at startup from `agent/cmd/agent/main.go`.

## Data and Contract Locations
- PostgreSQL DDL: `schemas/postgres/001_initial_schema.sql` through `schemas/postgres/009_billing_schema.sql`.
- Integration-service SQL migrations: `services/integration-service/migrations/`.
- Neo4j constraints: `schemas/neo4j/001_constraints.cypher`.
- gRPC protocol contract: `proto/agent/v1/agent.proto`.
- Generated protobuf outputs: `proto/gen/go/`.

## Frontend Route Surface
- Route composition lives in `packages/frontend/web-app/src/App.tsx`.
- Layout shell components live in `packages/frontend/web-app/src/components/layout/`.
- Major route pages include `packages/frontend/web-app/src/pages/ConnectionsPage.tsx`, `packages/frontend/web-app/src/pages/IntegrationStudioPage.tsx`, `packages/frontend/web-app/src/pages/DataHubPage.tsx`, and `packages/frontend/web-app/src/pages/PricingPage.tsx`.

## Build and Operations Anchors
- Build/test orchestration is centralized in `Makefile`.
- Local stack bootstrapping is `make dev-up` backed by `infra/docker/docker-compose.yml`.
- Repository overview and onboarding pointer are in `README.md` and `docs/development/onboarding.md`.

## Where To Add New Code
- New API gateway endpoint: add handler in `services/api-gateway/internal/handlers/` and wire route in `services/api-gateway/cmd/api-gateway/main.go`.
- New integration endpoint/flow: extend `services/integration-service/src/api/mod.rs` and supporting modules in `services/integration-service/src/storage/` or `services/integration-service/src/playbooks/`.
- New agent capability: implement handler in `agent/internal/adapters/<name>/` and register it in `agent/cmd/agent/main.go`.
- New frontend screen: create page in `packages/frontend/web-app/src/pages/` and route in `packages/frontend/web-app/src/App.tsx`.

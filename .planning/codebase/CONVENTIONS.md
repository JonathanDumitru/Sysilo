# Coding Conventions

Analysis date: 2026-02-28.
Scope: observed conventions in `agent/`, `services/`, `packages/`, and shared build files.

## Monorepo shape and boundaries
- The repository is polyglot: Go (`agent/`, `services/api-gateway/`, `services/agent-gateway/`), Rust (`services/*-service`), Python (`services/ai-service/`), and TypeScript (`packages/frontend/web-app/`, `packages/sdk/typescript/`).
- Service startup entrypoints are explicit: `agent/cmd/agent/main.go`, `services/api-gateway/cmd/api-gateway/main.go`, `services/agent-gateway/cmd/agent-gateway/main.go`, `services/integration-service/src/main.rs`, `services/ai-service/src/ai_service/main.py`.
- Build/test/lint/fmt orchestration is centralized in `Makefile` and then delegated per language.

## Shared coding style signals
- Prefer structured logging rather than plain prints: Go uses Zap in `agent/pkg/logging/logger.go` and gateway mains; Rust uses `tracing` JSON in `services/integration-service/src/main.rs`; Python uses `structlog` in `services/ai-service/src/ai_service/main.py`.
- Keep configuration in typed structs/models and load from env/files: Go in `services/api-gateway/internal/config/config.go`, Python settings in `services/ai-service/src/ai_service/config.py`, Rust config module wiring in `services/integration-service/src/main.rs` + `services/integration-service/src/config/mod.rs`.
- HTTP layers separate routing, middleware, and handlers (`services/api-gateway/internal/handlers/`, `services/api-gateway/internal/middleware/`, `services/integration-service/src/api/`, `services/integration-service/src/middleware/`).

## Go conventions
- Standard Go formatting and naming are assumed (`go fmt` targets in `Makefile`, snake_case directories, CamelCase exported identifiers).
- Handler dependencies are injected via structs (see `type Handler struct` in `services/api-gateway/internal/handlers/handlers.go`).
- HTTP helpers are centralized per package (`respondJSON`, `respondError` in `services/api-gateway/internal/handlers/handlers.go`).
- Middleware composition is explicit and ordered in router setup (`services/api-gateway/cmd/api-gateway/main.go`).
- Config structs use YAML tags and env override methods (`services/api-gateway/internal/config/config.go`).

## Rust conventions
- Rust edition is 2021 (`services/integration-service/Cargo.toml`).
- Modules use `mod.rs` plus submodules (`services/integration-service/src/playbooks/mod.rs`, `services/data-service/src/ingestion/mod.rs`).
- Types derive serde and debug traits heavily (`#[derive(Debug, Clone, Serialize, Deserialize)]` patterns in `services/integration-service/src/playbooks/mod.rs`).
- Serialized enums usually apply `#[serde(rename_all = "snake_case")]` (`services/integration-service/src/playbooks/mod.rs`, `services/data-service/src/ingestion/mod.rs`).
- Async service code uses Tokio + Axum and shared app state (`services/integration-service/src/main.rs`).
- Error modeling uses typed enums with `thiserror` for domain failures (`ExecutorError` in `services/integration-service/src/playbooks/executor.rs`).

## Python conventions
- Python baseline is 3.11+ (`services/ai-service/pyproject.toml`).
- Lint/type/format settings are tool-driven: Black, Ruff, and strict MyPy are configured in `services/ai-service/pyproject.toml`.
- FastAPI app structure keeps routers under `services/ai-service/src/ai_service/api/` and startup lifecycle in `services/ai-service/src/ai_service/main.py`.
- Configuration is centralized in a cached Pydantic settings object (`get_settings()` in `services/ai-service/src/ai_service/config.py`).

## TypeScript conventions
- Frontend uses Vite + React + TypeScript (`packages/frontend/web-app/vite.config.ts`, `packages/frontend/web-app/package.json`).
- Data access is isolated to service modules (`packages/frontend/web-app/src/services/assets.ts`, `packages/frontend/web-app/src/services/api.ts`).
- Query logic is wrapped in React Query hooks (`packages/frontend/web-app/src/hooks/useAssets.ts`).
- SDK package exports typed abstractions and helper harnesses (`packages/sdk/typescript/src/connector.ts`, `packages/sdk/typescript/src/testing.ts`).
- Path alias `@` is configured for `src` in `packages/frontend/web-app/vite.config.ts`.

## Practical guardrails for new code
- Add code in the language-local pattern first, then wire to root `Makefile` targets if needed.
- Preserve handler/service separation in gateways (`services/api-gateway/internal/handlers/` vs `services/api-gateway/internal/db/`).
- Keep Rust business logic testable as pure functions when possible, mirroring `services/integration-service/src/playbooks/executor.rs`.
- Keep frontend network concerns in `src/services/*` and avoid direct `fetch` calls in UI components; use `apiFetch` from `packages/frontend/web-app/src/services/api.ts`.
- Keep environment defaults development-safe but explicit (examples in `services/api-gateway/internal/config/config.go` and `services/ai-service/src/ai_service/config.py`).

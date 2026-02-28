# Testing Patterns

Analysis date: 2026-02-28.
Scope: current test frameworks, existing tests, and operational commands in this repo.

## Current test orchestration
- Root test orchestration is in `Makefile` (`test`, `test-agent`, `test-agent-gateway`, `test-api-gateway`, `test-integration-service`).
- Root `make test` currently runs Go tests for three Go services and Rust tests for `services/integration-service/`.
- Frontend, SDK, and Python tests are configured at package level but are not part of root `make test`.

## Rust testing (most implemented)
- Framework: built-in Rust unit tests via `cargo test` (`Makefile` target `test-integration-service`).
- Test placement is inline with source under `#[cfg(test)] mod tests`.
- Concrete examples:
  - `services/integration-service/src/playbooks/executor.rs`
  - `services/integration-service/src/playbooks/mod.rs`
  - `services/integration-service/src/connections/mod.rs`
  - `services/data-service/src/ingestion/mod.rs`
- Test naming follows `test_<unit>_<scenario>` (for example `test_find_starting_steps_simple_chain` in `services/integration-service/src/playbooks/executor.rs`).
- Common assertion style uses `assert_eq!`, `assert!`, and `matches!` for enum checks.

## Go testing (configured, little/no implementation)
- Framework: standard `go test`.
- Commands are defined and expected to run recursively:
  - `cd agent && go test -v ./...`
  - `cd services/agent-gateway && go test -v ./...`
  - `cd services/api-gateway && go test -v ./...`
- Pattern expectation is co-located `_test.go` files near production code (`services/api-gateway/internal/handlers/`, `agent/internal/`, `services/agent-gateway/internal/`).
- Current repository signal: test execution paths exist, but test-file footprint is minimal compared with Rust inline tests.

## Python testing (configured, expected under tests/)
- Framework: `pytest` with async support and coverage dependencies in `services/ai-service/pyproject.toml`.
- Pytest configuration exists in `[tool.pytest.ini_options]` with `testpaths = ["tests"]`.
- Expected location is `services/ai-service/tests/`.
- Practical run commands:
  - `cd services/ai-service && pytest`
  - `cd services/ai-service && pytest --cov=ai_service --cov-report=term-missing`

## TypeScript testing (configured via Vitest)
- Frontend test runner: Vitest in `packages/frontend/web-app/package.json` (`"test": "vitest"`).
- SDK test runner: Vitest in `packages/sdk/typescript/package.json` (`"test": "vitest"`).
- Typical commands:
  - `cd packages/frontend/web-app && npm run test`
  - `cd packages/sdk/typescript && npm run test`
- There are testing utilities in SDK (`packages/sdk/typescript/src/testing.ts`) that can be exercised by unit tests.

## Practical patterns to follow when adding tests
- Rust: keep pure logic extractable and test inline near module code, following `services/integration-service/src/playbooks/executor.rs`.
- Go: create package-local `_test.go` files and target handler/helper functions first (for example around `services/api-gateway/internal/handlers/handlers.go`).
- Python: place tests in `services/ai-service/tests/` to match `pyproject.toml` testpaths and use `pytest.mark.asyncio` for async code.
- Frontend/SDK: use Vitest and co-locate `*.test.ts`/`*.test.tsx` files near hooks/services (`packages/frontend/web-app/src/hooks/`, `packages/sdk/typescript/src/`).

## High-value coverage gaps
- API handler behavior in `services/api-gateway/internal/handlers/` (status codes, validation, error mapping).
- Integration service API/storage/kafka paths in `services/integration-service/src/api/`, `services/integration-service/src/storage/`, `services/integration-service/src/kafka/`.
- AI service routers in `services/ai-service/src/ai_service/api/` with FastAPI client tests.
- Frontend service-layer edge cases in `packages/frontend/web-app/src/services/` and query hooks in `packages/frontend/web-app/src/hooks/`.

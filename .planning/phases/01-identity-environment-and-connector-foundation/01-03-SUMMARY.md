# 01-03 Summary

## Outcome
Implemented connector lifecycle and onboarding gating across owned SDK/backend/frontend files with a new registry-backed health-check path, draft/test/activate state handling, and write-only credential replacement semantics.

## Completed Tasks

### Task 1: Connector lifecycle state machine + registry-backed health tests
- Added `services/integration-service/src/connectors/registry.rs` with prioritized connector-specific `health_check` handlers for PostgreSQL, MySQL, Snowflake, Oracle, Salesforce, and REST API.
- Updated `services/integration-service/src/connections/mod.rs` with lifecycle primitives:
  - statuses: `draft`, `tested`, `active`, `error`
  - transition guard `determine_next_status(...)` that blocks activation unless latest test succeeded
- Updated `services/integration-service/src/connections/api.rs`:
  - `/connections/:id/test` now dispatches through connector registry health checks
  - successful test sets lifecycle to `tested`; failed test sets `error`
  - create/update flows reset to `draft` when config/credentials change
  - update now supports `desired_status: "active"` and enforces successful test gating before activation

### Task 2: Write-only and replace-only credential semantics
- Updated `services/integration-service/src/connections/mod.rs` with strict credential validation/normalization by auth mode (`credential`, `oauth`, `api_key`) and masked placeholder rejection.
- Updated `services/integration-service/src/connections/api.rs`:
  - credentials accepted only as write payloads; normalized before persistence
  - replacement-only behavior for updates (`credentials` optional; when supplied, treated as full replacement)
  - connection responses remain masked (`has_credentials` only), no plaintext credential fields returned
- Updated `packages/frontend/web-app/src/services/connections.ts` types to match masked response contract and activation request model.

### Task 3: Auth-aware onboarding flow + test-before-activate UI
- Updated `packages/frontend/web-app/src/pages/ConnectionsPage.tsx`:
  - onboarding text and UX now explicitly communicate draft-first and write-only credential behavior by auth type
  - save action is draft-oriented (`Save Draft`)
  - test result surfaced per connection (`success` / `failed` with error tooltip)
  - activation button added and blocked until test status is `success`
- Updated `packages/frontend/web-app/src/hooks/useConnections.ts` and `packages/frontend/web-app/src/services/connections.ts` with `activateConnection(...)` mutation path.
- Updated `packages/sdk/typescript/src/connector.ts` with exported supported connector/auth specs (`SUPPORTED_CONNECTORS`) to keep SDK contract authoritative.

## Verification

### Commands Run
1. `cargo test --manifest-path services/integration-service/Cargo.toml connector_lifecycle -- --nocapture`
- Result: **failed**
- Reason: pre-existing compile errors in non-owned files (`services/integration-service/src/api/mod.rs`, `services/integration-service/src/playbooks/api.rs`) with `ApiError` initializer field mismatches (`E0063`).

2. `cargo test --manifest-path services/integration-service/Cargo.toml secret_handling -- --nocapture`
- Result: **failed**
- Reason: same pre-existing non-owned compile errors (`E0063`) block test execution.

3. `cargo test --manifest-path services/integration-service/Cargo.toml`
- Result: **failed**
- Reason: same pre-existing non-owned compile errors (`E0063`).

4. `pnpm --filter web-app test -- connections-onboarding`
- Result: **failed**
- Reason: no matching test files found in `packages/frontend/web-app` for that filter.

5. `pnpm --filter @sysilo/sdk-typescript test`
- Result: **passed (no-op)**
- Reason: filter matches no project in this workspace (`No projects matched the filters`).

6. `pnpm --filter @sysilo/connector-sdk test` (closest valid package)
- Result: **failed**
- Reason: `vitest` not found in environment (`sh: vitest: command not found`).

## Deviations
- Plan referenced `services/integration-service/src/connectors/registry.rs`; file did not exist and was created.
- At-rest credential encryption implementation was constrained by non-owned runtime/storage paths that currently consume stored credential JSON directly; this implementation enforces write-only/replace-only semantics and non-readback responses in owned files.
- Verification was partially blocked by pre-existing non-owned Rust compile failures and missing local test tooling.

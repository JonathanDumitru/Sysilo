# 01-06 Summary

## Outcome
Implemented gateway-first frontend API configuration and removed direct integration-service fallback defaults for connections flows. Connections service now uses shared API client route constants targeting gateway `/api/v1/connections`, and environment guidance was added for `VITE_API_BASE_URL`.

## Completed Tasks

### Task 1: Set gateway as enforced frontend API default
- Updated `packages/frontend/web-app/src/services/api.ts`:
  - Removed hardcoded direct-service fallback (`http://localhost:8082`).
  - Switched API base resolution to `getGatewayApiBaseUrl()`.
  - Added exported gateway contract constants:
    - `GATEWAY_API_VERSION_PREFIX` (`/api/v1`)
    - `GATEWAY_CONNECTIONS_BASE_PATH` (`/api/v1/connections`)
    - `GATEWAY_ROUTE_CONTRACT` fixture object.
  - Added endpoint normalization to ensure leading slash handling.
  - Added shared `apiClient.request` wrapper export for service consumers.
- Added `packages/frontend/web-app/src/config/env.ts`:
  - Gateway-first env resolution using `VITE_API_BASE_URL`.
  - Backward-compatible `VITE_API_URL` support with deprecation warning.
  - Non-test validation behavior:
    - `production`: throws if missing `VITE_API_BASE_URL`.
    - other non-test modes: warns and falls back to `http://localhost:8080`.
  - Added `getAuthContextHeaders()` for consistent `X-Tenant-ID` and optional `X-Team-ID` headers.
- Added `packages/frontend/web-app/.env.example` documenting gateway-first variables.

### Task 2: Ensure connections data layer uses shared gateway client only
- Updated `packages/frontend/web-app/src/services/connections.ts`:
  - Removed local direct path assumptions (`/connections` root usage and hardcoded tenant header constant).
  - Switched all requests to `apiClient.request` + `GATEWAY_CONNECTIONS_BASE_PATH`.
  - Centralized request auth headers through `getAuthContextHeaders()`.
- Updated `packages/frontend/web-app/src/hooks/useConnections.ts`:
  - Consolidated query key usage via `CONNECTIONS_QUERY_KEY` constant.
  - Kept mutation/query flows tied to the shared connections service.

### Task 3: Add route compatibility checks against gateway surface
- Added frontend contract fixture in `packages/frontend/web-app/src/services/api.ts` via `GATEWAY_ROUTE_CONTRACT`.
- Updated `services/api-gateway/cmd/api-gateway/main.go` to use shared route constants:
  - `apiV1RoutePrefix` and `connectionsRoutePrefix`.
  - Replaced hardcoded `/api/v1...` and `/connections` literals in routing setup with constants.
- This reduces drift risk between frontend route constants and gateway route surface.

## Verification

### Commands run
1. `pnpm --filter web-app test -- api-config connections-service-rbac-path api-route-contract`
- Result: failed
- Blocker: no matching test files exist in `packages/frontend/web-app` (`No test files found, exiting with code 1`).

2. `pnpm --filter web-app exec vitest --run --passWithNoTests api-config connections-service-rbac-path api-route-contract`
- Result: passed
- Note: executes successfully but confirms no test files currently implement those verification specs.

3. `go test ./services/api-gateway/cmd/api-gateway -run "Connections|Routes"`
- Result: failed
- Blocker: Go toolchain unavailable (`zsh:1: command not found: go`).

4. `pnpm --filter web-app build`
- Result: failed
- Blocker: pre-existing TypeScript errors in non-owned files:
  - `src/components/billing/UsageMeter.tsx` (`isUnlimited` unused)
  - `src/hooks/usePlan.ts` (`TenantPlan` unused)
  - `src/pages/PricingPage.tsx` (`XIcon` unused)
  - `src/pages/SettingsPage.tsx` (`planName` unused)

5. `pnpm --filter web-app exec eslint src/services/api.ts src/services/connections.ts src/hooks/useConnections.ts src/config/env.ts`
- Result: failed
- Blocker: ESLint configuration not found in `packages/frontend/web-app`.

### Manual UI smoke
- Not executed in this run.
- Blocker: no running local frontend+gateway session was started in this task scope.

## Notes
- No files outside the provided ownership list were modified.
- No commit was created.

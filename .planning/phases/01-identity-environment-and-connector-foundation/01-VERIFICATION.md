status: gaps_found
phase: 01-identity-environment-and-connector-foundation
verified_on: 2026-03-01
verifier: codex

# Phase 1 Verification Report

## Goal Under Verification
Enterprise users can securely access Sysilo, operate in bounded environments, and connect prioritized systems through a stable connector foundation.

Sources reviewed:
- `.planning/phases/01-identity-environment-and-connector-foundation/01-01-PLAN.md`
- `.planning/phases/01-identity-environment-and-connector-foundation/01-02-PLAN.md`
- `.planning/phases/01-identity-environment-and-connector-foundation/01-03-PLAN.md`
- `.planning/phases/01-identity-environment-and-connector-foundation/01-01-SUMMARY.md`
- `.planning/phases/01-identity-environment-and-connector-foundation/01-02-SUMMARY.md`
- `.planning/phases/01-identity-environment-and-connector-foundation/01-03-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`

## Top-Line Verdict
Phase 1 is **not yet fully achieved**. Core pieces are implemented, but critical integration/activation gaps prevent end-to-end goal completion.

## Major Gaps

1. Identity flows are implemented but not routed/exposed in the API gateway.
- Handlers exist (`auth_sso.go`, `auth_breakglass.go`, `auth_session.go`, `scim_users.go`), but router wiring in `services/api-gateway/cmd/api-gateway/main.go` does not register SSO/SCIM/session-refresh/break-glass endpoints.
- Impact: roadmap success criterion "User can sign in through enterprise SSO and user lifecycle changes from SCIM provisioning are reflected in access" is not verifiably deliverable end-to-end.

2. Environment/team boundary requirement is only partially met.
- Environment policy exists (`authorization/environment_policy.go`) and connection scoping exists in integration-service storage.
- But integration-service runs with `optional_tenant_context_middleware` in `services/integration-service/src/main.rs`, which defaults missing environment to `dev`.
- No concrete team-scope enforcement model was found in gateway/integration-service authorization path (GOV-02 requires team/environment RBAC).

3. Frontend path bypasses gateway RBAC controls.
- Web app API base defaults to integration-service directly (`packages/frontend/web-app/src/services/api.ts` -> `http://localhost:8082`), while gateway RBAC is implemented separately.
- Impact: gateway environment-role checks are not guaranteed on frontend-driven operations.

4. Connector metadata authority is split, not SDK-authoritative in UI.
- SDK exports `SUPPORTED_CONNECTORS`, but frontend still uses local `CONNECTOR_TYPES` in `packages/frontend/web-app/src/services/connections.ts`.
- Impact: PLAT-02 "stable connector SDK-backed framework" is partially implemented but not consistently consumed.

5. Verification evidence is incomplete due test/tooling blockers.
- `go` toolchain unavailable (`go: command not found`), so gateway Go tests could not run.
- `cargo test` fails due pre-existing non-owned compile errors in `src/api/mod.rs` and `src/playbooks/api.rs` (`ApiError` field mismatch).
- Frontend targeted test selectors return no matching tests (`vitest -- connections-onboarding`).

## Requirement Coverage (Phase 1 scope)

| Requirement | Result | Evidence |
|---|---|---|
| GOV-03 (SSO + SCIM) | Partial | OIDC/SCIM/session logic implemented in handlers and DB methods; missing router exposure blocks end-to-end readiness |
| GOV-02 (RBAC by team/environment) | Partial | Environment RBAC implemented; no concrete team-scope enforcement found; bypass path exists |
| PLAT-01 (env separation dev/staging/prod) | Partial | Connection APIs scoped by environment; optional tenant context can default to `dev` when missing |
| INVT-01 (connect enterprise system securely) | Partial | Draft/test/activate lifecycle and connector health checks implemented; secure auth flow present but full security posture/testing incomplete |
| PLAT-02 (stable SDK-backed connector management) | Partial | Connector registry and SDK specs exist; frontend still duplicates connector metadata instead of consuming SDK constants |

## Must-Have Truths Check

### Plan 01-01 (Identity)
- SSO sign-in + valid session: **Partial** (handler exists; route exposure missing)
- Break-glass controlled/auditable: **Implemented** (eligibility + audit event code present)
- JIT provisioning on first SSO: **Implemented** (upsert on callback)
- Short-lived access + silent refresh rotation: **Implemented** (token manager + refresh rotate)
- SCIM deactivate blocks API access: **Partial** (session-version/status enforcement exists; SCIM route exposure missing)

### Plan 01-02 (Environment Controls)
- Role enforcement differs by environment: **Implemented in policy**
- Visibility constrained to selected environment: **Partial** (connections scoped; broader scope and bypass caveats remain)
- Production mutation requires confirmation/reason: **Implemented** (frontend guard + integration-service header enforcement)

### Plan 01-03 (Connector Foundation)
- Draft -> test -> activate gating: **Implemented**
- Failed tests block activation: **Implemented**
- No plaintext credential readback: **Implemented at API response layer** (`has_credentials` contract)

## Overall Assessment
Phase 1 contains substantial implementation progress but does not yet satisfy roadmap-level success criteria with production confidence. The highest-priority blockers are endpoint routing/exposure for identity flows and consistent enforcement topology (gateway vs direct service access).

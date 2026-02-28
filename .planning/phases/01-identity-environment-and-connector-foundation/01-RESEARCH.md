# Phase 1 Research: Identity, Environment, and Connector Foundation

**Phase**: 01-identity-environment-and-connector-foundation  
**Date**: 2026-02-28  
**Scope Requirements**: INVT-01, GOV-02, GOV-03, PLAT-01, PLAT-02

## 1. Objective Fit

Phase 1 must establish security and platform primitives that all later phases depend on:
- Enterprise identity entry + lifecycle controls (GOV-03)
- Environment-bounded authorization model (GOV-02, PLAT-01)
- Reliable connector onboarding and SDK-backed connector lifecycle (INVT-01, PLAT-02)

This phase should avoid inventory intelligence and incident workflows beyond what is required for secure connector onboarding.

## 2. Current-State Findings

## Identity and Access Baseline
- API gateway currently uses HMAC JWT auth middleware (`services/api-gateway/internal/middleware/middleware.go`) with `tenant_id`, `sub`, and `roles` claim extraction.
- Authorization today is coarse role checks (`RequireRole`) without environment scoping.
- No enterprise SSO/OIDC/SAML handshake implementation found.
- No SCIM endpoints/provisioning/deprovisioning pipeline found.

## Tenant/Service Context
- Integration service already accepts tenant/user/role context via headers (`services/integration-service/src/middleware/mod.rs`).
- This supports extension for environment claims and scoped enforcement, but environment is not yet a first-class field.

## Connector Foundation
- Connection CRUD/test APIs exist (`services/integration-service/src/connections/api.rs`).
- Connector/auth types and config validation are implemented (`services/integration-service/src/connections/mod.rs`).
- Frontend has auth-type-aware connection UI (`packages/frontend/web-app/src/pages/ConnectionsPage.tsx`, `src/services/connections.ts`).
- TypeScript SDK foundation exists (`packages/sdk/typescript/src/connector.ts`, `types.ts`) with registry + base connector abstractions.
- Current runtime "test connection" is schema-level validation only; no real remote connectivity verification yet.

## 3. Requirement-by-Requirement Research

### GOV-03: Enterprise SSO + SCIM Provisioning

## Needed Capability
- SSO-first login for enterprise users
- JIT user provisioning at first SSO login
- SCIM create/update/deactivate user synchronization
- Deprovisioning should invalidate active sessions quickly

## Gaps
- Existing JWT model assumes local token issuance; no external IdP trust or token exchange flow.
- User repository supports user CRUD/status/roles, but no IdP identity linkage (e.g., external subject ID).
- No session revocation mechanism tied to SCIM deactivation events.

## Planning Direction
- Add OIDC-based enterprise auth provider integration in gateway (preferred v1 path over SAML due implementation complexity).
- Introduce identity mapping table: `tenant_id + idp_subject -> user_id`.
- Implement JIT provisioning on first successful IdP token validation with domain/tenant allowlist checks.
- Add SCIM v2 endpoints (Users, optionally Groups minimal) in gateway admin surface.
- Store user status (`active`/`inactive`) and enforce status checks on each authenticated request.
- Add token revocation/versioning strategy (e.g., `session_version` in user record checked in JWT validation path).

## Minimal Acceptable v1
- OIDC SSO login and callback
- JIT user creation and role bootstrap
- SCIM user create/update/deactivate
- Deactivated user blocked immediately (request-time check + short token TTL)

### GOV-02 + PLAT-01: Role-by-Environment Access Boundaries

## Needed Capability
- User role assignments can differ by environment (dev/staging/prod).
- Read/write restrictions must be enforceable by environment.
- Production-changing actions require explicit confirmation and reason capture (from phase context decisions).

## Gaps
- Roles are global arrays in JWT claims and user table; no environment dimension.
- Connection/integration rows are tenant-scoped but not environment-scoped.
- API endpoints do not require environment parameter/context for authorization decisions.

## Planning Direction
- Define canonical environments: `dev`, `staging`, `prod`.
- Introduce `environment` as a required dimension for connector/integration resources.
- Add environment role binding model:
  - `user_environment_roles(tenant_id, user_id, environment, role)`
- Enforce authorization using `(role, environment, action)` policy matrix at gateway and integration-service boundary.
- Pass `x-environment` or signed claim in request context; reject missing/invalid environment for protected routes.
- Update UI app shell with persistent global environment switcher and prominent environment badge.
- Require extra confirmation modal + mandatory reason for production mutating actions; include reason in API payload and audit event stream.

## Minimal Acceptable v1
- Environment selector persisted client-side + server validation
- Environment-scoped list/read/write for connections/integrations
- Distinct permissions by env (at least viewer/operator/admin variants)
- Production mutation guardrail (confirm + reason)

### INVT-01 + PLAT-02: Secure Connector Onboarding + Stable SDK Foundation

## Needed Capability
- Connect prioritized enterprise systems with secure credentials.
- Stable, SDK-backed connector framework for supported connectors.
- Onboarding UX should be auth-type-specific and require connection test before active state.

## Gaps
- Connection test is not performing real connector-level health checks.
- Secrets handling policy is incomplete (write-only/replace-only behavior after save must be explicit).
- SDK exists in TS package but not yet wired as authoritative runtime execution path for connector operations.

## Planning Direction
- Adopt connector lifecycle contract for v1:
  - `draft -> tested -> active` (and `error` on failed verification)
- Implement connector runtime bridge where `/connections/:id/test` calls concrete connector `healthCheck`.
- Keep prioritized connector set narrow for v1 (from context): Postgres, MySQL, Snowflake, Salesforce, REST API (or final shortlist agreed in planning).
- Enforce secret handling rules:
  - Never return raw secrets from API
  - Store encrypted at rest
  - Update path supports replace-only credentials
  - Redacted field markers in response (`has_credentials`)
- Align frontend wizard to auth mode with required fields, draft save, and test-before-activate gating.

## Minimal Acceptable v1
- Reliable test endpoint performing real connectivity check for prioritized connectors
- Onboarding wizard per auth type with draft support
- Secret masking/write-only semantics end-to-end
- SDK contract and registry treated as integration point for supported connectors

## 4. Architecture and Data Model Implications

## New/Changed Core Entities
- `users` enhancement: identity provider subject mapping and session/version invalidation support
- `user_environment_roles`: per-environment RBAC
- `connections`: add `environment`, explicit lifecycle status, last tested metadata
- `audit_events` (or integration with existing governance audit service) for production reasons and access-sensitive actions

## Context Propagation
- Gateway auth should resolve: tenant, user, environment, effective role set.
- Gateway forwards signed/validated context headers to downstream services.
- Integration service re-validates required context presence and applies resource-level checks.

## 5. Delivery Strategy for 01-01 Plan

1. Identity foundation
- Implement OIDC SSO login/callback and JIT provisioning.
- Add user status/session-version enforcement and token refresh strategy.

2. SCIM lifecycle
- Implement SCIM user CRUD/deactivate paths mapped to local user model.
- Ensure deactivate blocks API access immediately.

3. Environment-aware RBAC
- Add environment role bindings and policy checks in gateway.
- Add `environment` column and filters on connection/integration APIs.
- Add frontend environment switcher + clear badges.

4. Connector lifecycle hardening
- Add connection state machine and real connector health checks.
- Enforce test-before-activate.
- Implement secret write-only/update behavior.

5. Guardrails + traceability
- Add production action confirmation + reason capture in UI/API.
- Emit audit events for high-risk actions and permission-sensitive changes.

## 6. Risks and Mitigations

- Risk: Auth rewrite scope expansion (OIDC + SCIM + RBAC) may exceed phase capacity.
  - Mitigation: constrain to one OIDC provider pattern and SCIM Users subset for v1.
- Risk: Environment dimension retrofitting causes broad schema/API churn.
  - Mitigation: start with connection/integration entities only; enforce default migration path.
- Risk: Connector runtime stability for multiple systems in first cut.
  - Mitigation: strict prioritized connector shortlist; common test harness and contract tests.
- Risk: Deprovisioning race conditions with active JWTs.
  - Mitigation: short token TTL + request-time user status/session-version checks.

## 7. Validation Targets (Phase Success Evidence)

- GOV-03 evidence
  - SSO login works for approved tenant domain.
  - JIT user created on first login.
  - SCIM deactivate causes immediate access denial on subsequent API call.

- GOV-02 / PLAT-01 evidence
  - Same user has different permissions across dev/staging/prod and enforcement is correct.
  - Mutating production action cannot proceed without confirmation + reason.

- INVT-01 / PLAT-02 evidence
  - User can create connector in draft, test successfully, then activate.
  - Failed connectivity test blocks activation.
  - API never returns plaintext credentials after save/update.

## 8. Open Decisions for Planning

- Which single IdP integration is first-class for v1 reference implementation.
- Final prioritized connector list for Phase 1 completion criteria.
- Whether environment is represented as claim-only, request header, or both (recommended: both with server-side validation).
- Exact RBAC role matrix per environment (viewer/operator/admin vs expanded roles).

## 9. Recommendation

Proceed with a single integrated Phase 1 plan (`01-01`) that delivers identity and environment boundaries before full connector activation gates, but keeps onboarding UI progress parallelizable. The critical path is GOV-03 -> GOV-02/PLAT-01 enforcement primitives -> INVT-01/PLAT-02 runtime connector verification.

# Codebase Concerns

**Analysis Date:** 2026-02-28

## Highest Risk (Security / Abuse)

1. Missing transport security on agent control plane
- Evidence: gRPC server starts without TLS credentials in `services/agent-gateway/cmd/agent-gateway/main.go`.
- Evidence: explicit TODO for TLS and auth verification gaps in `services/agent-gateway/cmd/agent-gateway/main.go` and `services/agent-gateway/internal/tunnel/server.go`.
- Risk: agent registration/task traffic can be intercepted or spoofed in non-isolated networks.
- Practical fix: require mTLS for `Connect`, reject plaintext in non-dev, and bind agent identity to cert SAN + tenant.

2. Stripe webhook integrity not enforced
- Evidence: webhook payload is parsed but signature verification is not implemented in `services/api-gateway/internal/handlers/billing_handlers.go`.
- Risk: forged billing events can mutate subscription state.
- Practical fix: enforce `Stripe-Signature` validation with `STRIPE_WEBHOOK_SECRET`; reject unsigned/invalid events with `400`.

3. Auth model allows tenant spoofing via header fallback
- Evidence: `TenantContext` accepts `X-Tenant-ID` when JWT tenant is missing in `services/api-gateway/internal/middleware/middleware.go`.
- Risk: service-to-service fallback can become a privilege escalation path if exposed to untrusted callers.
- Practical fix: only allow header fallback on authenticated internal traffic (mTLS/internal network) and disallow on public routes.

4. JWT verification is weakly configured for production
- Evidence: default secret `dev-secret-change-in-production` and HMAC-only parsing in `services/api-gateway/internal/config/config.go` and `services/api-gateway/internal/middleware/middleware.go`.
- Risk: predictable default secret or env misconfiguration can fully bypass auth.
- Practical fix: hard-fail boot when default secret is present outside `development`; prefer asymmetric keys + issuer/audience checks.

## High Risk (Reliability / Product Integrity)

5. Rate limiting middleware is effectively a no-op
- Evidence: placeholder logic and TODOs in `services/api-gateway/internal/middleware/middleware.go`.
- Risk: unauthenticated/authenticated endpoints are vulnerable to burst abuse and noisy-neighbor impact.
- Practical fix: implement Redis-backed token bucket keyed by tenant/user/IP; emit 429 with retry headers.

6. Core task dispatch path is incomplete in API gateway
- Evidence: TODOs for send/cancel dispatch in `services/api-gateway/internal/handlers/handlers.go`.
- Risk: user actions can succeed at API level but fail to trigger agent execution, creating false-positive UX states.
- Practical fix: wire dispatch to integration/agent gateway with idempotent enqueue + persisted dispatch status.

7. Data ingestion execution path is unimplemented
- Evidence: `todo!()` in `services/data-service/src/ingestion/mod.rs` (`execute_ingestion_job`, status, cancel, batch).
- Risk: ingestion APIs can compile but are not operational, creating runtime failures.
- Practical fix: either gate these endpoints behind feature flags or implement minimal end-to-end job lifecycle before exposure.

8. Integration service uses permissive CORS globally
- Evidence: `CorsLayer::permissive()` in `services/integration-service/src/main.rs`.
- Risk: over-broad cross-origin access expands attack surface for browser-based abuse.
- Practical fix: restrict origins/methods/headers via env config per environment.

## Medium Risk (Performance / Maintainability)

9. AI service exposes raw LLM-generated query execution
- Evidence: SQL/Cypher generation and optional execution in `services/ai-service/src/ai_service/api/chat.py` (`generate_query`, `_execute_sql`, `_execute_cypher`).
- Risk: unsafe query execution, data exfiltration, and expensive scans if query guardrails are bypassed.
- Practical fix: enforce read-only allowlist, deny DDL/DML, add row/time limits, and tenant-scoped query templates.

10. Sensitive defaults in runtime configuration
- Evidence: default DB/Neo4j credentials in `services/ai-service/src/ai_service/config.py`, `services/api-gateway/internal/config/config.go`, and `services/integration-service/src/config/mod.rs`.
- Risk: accidental deployment with dev credentials and lateral movement risk.
- Practical fix: fail-fast when default secrets are detected in non-dev environments.

11. Frontend uses static dev tenant header
- Evidence: `DEV_TENANT_ID` and unconditional `X-Tenant-ID` usage in `packages/frontend/web-app/src/services/connections.ts`.
- Risk: encourages incorrect multi-tenant assumptions and can mask auth integration defects.
- Practical fix: derive tenant identity from auth token/session context only.

12. High polyglot surface with weak test safety net
- Evidence: no obvious Go/Python/TS test files found by repository scan; multiple TODO-heavy critical paths.
- Affected areas include `services/api-gateway/`, `services/ai-service/`, `packages/frontend/web-app/`, `services/integration-service/`.
- Risk: regressions likely during cross-service changes; difficult to verify contract compatibility.
- Practical fix: add minimal contract + integration smoke tests for auth, dispatch, billing webhooks, and connection CRUD.

## Quick Wins (1-2 sprints)

- Enforce TLS/mTLS and agent registration verification in `services/agent-gateway/cmd/agent-gateway/main.go` and `services/agent-gateway/internal/tunnel/server.go`.
- Implement Stripe signature verification in `services/api-gateway/internal/handlers/billing_handlers.go`.
- Replace placeholder rate limit with Redis-backed limiter in `services/api-gateway/internal/middleware/middleware.go`.
- Replace permissive CORS in `services/integration-service/src/main.rs` with explicit allowlist config.
- Add failure guards for default secrets in `services/api-gateway/internal/config/config.go` and `services/ai-service/src/ai_service/config.py`.


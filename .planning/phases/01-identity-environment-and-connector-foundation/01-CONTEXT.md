# Phase 1: Identity, Environment, and Connector Foundation - Context

**Gathered:** 2026-02-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver secure enterprise access, environment-bounded permissions, and a stable connector onboarding foundation for prioritized systems. This phase does not expand into broader inventory intelligence, incident workflows, or advanced automation.

</domain>

<decisions>
## Implementation Decisions

### Identity entry and provisioning
- Enterprise login is SSO-first, with a controlled local admin break-glass path for recovery.
- SCIM deprovisioning revokes access immediately and should terminate active sessions quickly.
- First SSO login uses JIT user creation when domain/tenant policy checks pass.
- Sessions use short-lived tokens with silent refresh.

### Access boundaries by environment
- Access is role-per-environment (user role can differ across dev/staging/prod).
- Default posture is least privilege; no production write access unless explicitly granted.
- UI must keep environment context persistent and obvious (global switcher + clear environment badges).
- Production-changing actions require an extra confirmation step plus mandatory reason capture.

### Connector onboarding experience
- v1 prioritizes a high-value enterprise connector set, not broad connector parity.
- Onboarding uses guided, auth-type-specific wizards (credential vs OAuth vs API key).
- Connection testing is required before a connector reaches active/ready state; draft save is allowed.
- Secrets are write-only after save and masked on subsequent edits (replace-only behavior).

### Claude's Discretion
- Exact wording and placement of confirmation/reason prompts for production actions.
- Fine-grained token/session timeout values and refresh cadence.
- Final visual treatment for environment badges and onboarding progress indicators.

</decisions>

<specifics>
## Specific Ideas

- Sysilo should feel enterprise-safe from first login: clear permission boundaries and auditable high-risk behavior.
- Connector setup should be structured and confidence-building, not a generic form dump.
- v1 should optimize depth and reliability on a prioritized connector set.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `packages/frontend/web-app/src/pages/ConnectionsPage.tsx`: Existing connection creation/editing UX with auth-type-aware form sections.
- `packages/frontend/web-app/src/services/connections.ts`: Connector type metadata and auth mode definitions (`credential`, `oauth`, `api_key`).
- `packages/frontend/web-app/src/hooks/useConnections.ts`: Query/mutation hooks for connection CRUD and test actions.
- `services/integration-service/src/connections/api.rs`: Connection API contracts (create/update/test) suitable for onboarding flow extension.
- `services/integration-service/src/storage/mod.rs`: Storage layer already supports connection config/credential persistence paths.

### Established Patterns
- Frontend data flow uses service modules + React Query hooks (`src/services/*`, `src/hooks/*`) rather than direct fetches.
- API gateway and services already carry role/tenant concepts (e.g., role headers and unauthorized checks), enabling environment-scoped RBAC extension.
- Existing connector/auth model already distinguishes auth types, reducing need for a net-new onboarding abstraction.

### Integration Points
- API route surface for connection operations is already wired in `services/api-gateway/cmd/api-gateway/main.go` (`/connections` endpoints).
- Environment and role enforcement can extend middleware and auth context in `services/integration-service/src/middleware/mod.rs`.
- Frontend environment context and guardrails should integrate into existing routed app shell (`packages/frontend/web-app/src/App.tsx` and layout components).

</code_context>

<deferred>
## Deferred Ideas

- Broad long-tail connector parity across many systems (future phase after foundational reliability).
- Fully autonomous remediation behaviors without human approvals (governance/AI phases).
- Universal Zapier-style automation breadth beyond integration foundation (separate roadmap scope).

</deferred>

---

*Phase: 01-identity-environment-and-connector-foundation*
*Context gathered: 2026-02-28*

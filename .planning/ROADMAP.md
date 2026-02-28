# Roadmap: Sysilo

## Overview

Sysilo v1 delivers an enterprise integration intelligence control plane in a dependency-safe sequence: secure enterprise access and connector foundation, canonical integration inventory and lineage, operational health and incident handling, governance hardening, and finally an evidence-grounded AI copilot for faster and safer integration operations.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Identity, Environment, and Connector Foundation** - Establish secure enterprise access, environment boundaries, and connector SDK baseline.
- [ ] **Phase 2: Integration Inventory and Lineage** - Deliver auto-discovery with searchable inventory, ownership, and dependency visibility.
- [ ] **Phase 3: Health Monitoring and Incident Operations** - Deliver health signals, alerting, incident correlation, and reporting exports.
- [ ] **Phase 4: Governance Controls and Audit Assurance** - Enforce policy-gated high-risk actions with immutable auditability.
- [ ] **Phase 5: Evidence-Grounded AI Copilot** - Deliver diagnoser and mapping copilot workflows with citations and policy-safe recommendations.

## Phase Details

### Phase 1: Identity, Environment, and Connector Foundation
**Goal**: Enterprise users can securely access Sysilo, operate in bounded environments, and connect prioritized systems through a stable connector foundation.
**Depends on**: Nothing (first phase)
**Requirements**: INVT-01, GOV-02, GOV-03, PLAT-01, PLAT-02
**Success Criteria** (what must be TRUE):
  1. User can sign in through enterprise SSO and user lifecycle changes from SCIM provisioning are reflected in access.
  2. User can only view and operate integrations for allowed team/environment scopes (dev, staging, prod).
  3. User can connect a prioritized enterprise system with secure credential authentication.
  4. User can manage connectors using the SDK-backed connector framework for supported systems.
**Plans**: TBD

Plans:
- [ ] 01-01: TBD

### Phase 2: Integration Inventory and Lineage
**Goal**: Users can discover and understand their integration landscape with ownership and dependency context in one place.
**Depends on**: Phase 1
**Requirements**: INVT-02, INVT-03, INVT-04, GOV-01
**Success Criteria** (what must be TRUE):
  1. User can trigger or schedule discovery and see integration endpoints, jobs, and sync relationships populated.
  2. User can search and filter a central inventory by owner, system, environment, and status.
  3. User can inspect dependency and lineage relationships between integrations and core data objects.
  4. User can view and maintain owner, escalation contact, and runbook link for each integration.
**Plans**: TBD

Plans:
- [ ] 02-01: TBD

### Phase 3: Health Monitoring and Incident Operations
**Goal**: Users can detect integration degradation early, triage incidents with context, and report operational state.
**Depends on**: Phase 2
**Requirements**: HEAL-01, HEAL-02, HEAL-03, HEAL-04, PLAT-03
**Success Criteria** (what must be TRUE):
  1. User can view health metrics per integration, including success rate, latency, retry count, and backlog.
  2. User receives alerts when configured health thresholds are breached.
  3. User can open a failure view showing correlated logs, impacted dependencies, and suspected blast radius.
  4. User can track incident lifecycle state and resolution notes per incident.
  5. User can export inventory and health summaries for operational reporting.
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Governance Controls and Audit Assurance
**Goal**: Sysilo enforces enterprise governance controls over high-risk integration actions with verifiable audit history.
**Depends on**: Phase 3
**Requirements**: GOV-04, GOV-05
**Success Criteria** (what must be TRUE):
  1. User can review immutable audit history for integration configuration changes and user actions.
  2. User can configure policy gates that require approval before high-risk integration actions execute.
  3. User can see approval decisions and policy outcomes captured in audit history for governed actions.
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

### Phase 5: Evidence-Grounded AI Copilot
**Goal**: Users can safely use AI assistance for diagnosis, mapping, and remediation with transparent evidence and policy alignment.
**Depends on**: Phase 4
**Requirements**: AICO-01, AICO-02, AICO-03, AICO-04, AICO-05
**Success Criteria** (what must be TRUE):
  1. User can ask AI to diagnose a failed integration and receive likely causes with confidence levels.
  2. User can view cited evidence (logs, lineage nodes, prior incidents) attached to AI diagnosis and recommendations.
  3. User can receive AI mapping suggestions during connector onboarding.
  4. User can simulate AI-proposed mapping or transform changes and view validation errors before applying.
  5. User can receive remediation recommendations that are pre-filtered by policy constraints.
**Plans**: TBD

Plans:
- [ ] 05-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Identity, Environment, and Connector Foundation | 0/1 | Not started | - |
| 2. Integration Inventory and Lineage | 0/1 | Not started | - |
| 3. Health Monitoring and Incident Operations | 0/1 | Not started | - |
| 4. Governance Controls and Audit Assurance | 0/1 | Not started | - |
| 5. Evidence-Grounded AI Copilot | 0/1 | Not started | - |

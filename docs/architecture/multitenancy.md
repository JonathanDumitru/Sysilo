# Multi-tenancy

## Intent

Define how tenant isolation is enforced across data, compute, and network layers.

## Isolation model

```mermaid
flowchart TB
  TenantA[Tenant A] --> APIA[API Gateway]
  TenantB[Tenant B] --> APIB[API Gateway]

  APIA --> RLSA[Row-level security]
  APIB --> RLSB[Row-level security]

  APIA --> ComputeA[Isolated Execution]
  APIB --> ComputeB[Isolated Execution]

  RLSA --> SharedDB[(Postgres)]
  RLSB --> SharedDB
```

## Rules

- Tenant ID on all records
- Isolated execution containers per tenant
- Per-tenant rate limits and quotas
- Agent connections scoped to tenant

## Open questions

- Do we require per-tenant encryption keys in V1?
- What is the approach for tenant data export and deletion?

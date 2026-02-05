# Deployment model

## Intent

Describe how the SaaS control plane and customer agents are deployed.

## Deployment topology

```mermaid
flowchart LR
  subgraph Cloud[Sysilo Cloud]
    CP[Control Plane]
    DB[(Primary DB)]
    Bus[Event Bus]
    CP --> DB
    CP --> Bus
  end

  subgraph Customer[Customer Environment]
    Agent[Agent]
    Systems[Systems]
    Warehouse[Data Warehouse]
  end

  Agent --> CP
  Systems --> Agent
  CP --> Warehouse
```

## Rollout plan (draft placeholder dates)

```mermaid
gantt
  title Initial Rollout (Draft)
  dateFormat  YYYY-MM-DD
  section Foundation
  Control Plane MVP :done, 2026-02-10, 30d
  Agent MVP         :active, 2026-02-20, 25d
  section Integrations
  First 5 Connectors : 2026-03-05, 30d
  Data Hub MVP       : 2026-03-20, 30d
  section Ops
  Operations Center MVP : 2026-04-05, 25d
```

## Open questions

- What is the minimum supported agent footprint?
- Which regions are required for V1?

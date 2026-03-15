# System context

## Intent

Show how Sysilo© interacts with customer environments, external systems, and data warehouses.

## Context diagram

```mermaid
flowchart LR
  subgraph Customer[Customer Environment]
    SaaS[SaaS Apps]
    OnPrem[On-prem Systems]
    Legacy[Legacy Systems]
    Data[Data Warehouse]
    Agent[Sysilo© Agent]
  end

  subgraph Sysilo[Sysilo© Control Plane]
    Studio[Integration Studio]
    Hub[Data Hub]
    Registry[Asset Registry]
    Ops[Operations Center]
    Gov[Governance Center]
    AI[AI Engine]
  end

  SaaS --> Agent
  OnPrem --> Agent
  Legacy --> Agent
  Agent --> Studio
  Studio --> Hub
  Hub --> Data
  Studio --> Registry
  Registry --> Gov
  Studio --> Ops
  AI --> Studio
  AI --> Hub
  AI --> Ops
  AI --> Gov
```

## Open questions

- Which system types must be supported for V1 discovery?
- Is the data warehouse always external, or do we support internal storage?

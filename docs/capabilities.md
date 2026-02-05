# Capabilities

## Intent

Provide a capability map that links the platform components to customer outcomes.

## Capability map

```mermaid
flowchart TB
  Outcomes[Customer Outcomes] --> Orchestration[Integration Orchestration]
  Outcomes --> Unification[Data Unification]
  Outcomes --> Visibility[Landscape Visibility]
  Outcomes --> Rationalization[App Rationalization]

  Orchestration --> Studio[Integration Studio]
  Orchestration --> Agents[Agent Architecture]

  Unification --> Hub[Data Hub]
  Unification --> Canonical[Canonical Models]

  Visibility --> Registry[Asset Registry]
  Visibility --> Ops[Operations Center]

  Rationalization --> Engine[Rationalization Engine]
  Rationalization --> Gov[Governance Center]

  AI[AI Engine] --> Studio
  AI --> Hub
  AI --> Registry
  AI --> Ops
  AI --> Gov
  AI --> Engine
```

## Notes

- Capabilities are grouped by outcome to keep the roadmap aligned to business value.
- The AI engine is horizontal and should be modeled as shared infrastructure.

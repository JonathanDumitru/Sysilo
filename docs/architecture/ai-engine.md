# AI engine

## Intent

Describe the shared AI layer that powers assistive and predictive features.

## Architecture

```mermaid
flowchart TB
  KG[Knowledge Graph] --> Gen[Generative]
  KG --> Pred[Predictive]
  KG --> Analytic[Analytical]

  Gen --> Studio[Integration Studio]
  Gen --> Hub[Data Hub]
  Gen --> Ops[Operations Center]
  Gen --> Gov[Governance Center]

  Pred --> Ops
  Pred --> Engine[Rationalization Engine]

  Analytic --> Registry[Asset Registry]
  Analytic --> Hub
```

## Example capabilities

- Mapping suggestions and code generation
- Anomaly detection and predictive alerts
- Impact modeling and rationalization scoring

## Open questions

- Where does feature storage live for the knowledge graph?
- Which models are hosted vs vendor-provided?

# Control plane architecture

## Intent

Describe the core services and how they communicate.

## Logical view

```mermaid
flowchart TB
  Gateway[API Gateway] --> Web[Web App]
  Gateway --> Services[API Services]
  Gateway --> AgentGW[Agent Gateway]

  Services --> Integrations[Integration Service]
  Services --> Data[Data Service]
  Services --> Assets[Asset Service]
  Services --> AI[AI Service]
  Services --> Ops[Ops Service]

  Integrations --> Bus[Event Bus]
  Data --> Bus
  Assets --> Bus
  Ops --> Bus

  Bus --> PrimaryDB[(Postgres)]
  Bus --> GraphDB[(Graph DB)]
  Bus --> Blob[(Object Storage)]
```

## Service boundaries

- API Gateway: authentication, routing, rate limiting
- API Services: user-facing APIs and orchestration
- Core services: integration, data, asset, ops, AI
- Event bus: durable eventing and workflow decoupling

## Open questions

- Which services are synchronous vs event-driven for V1?
- Do we need a separate job scheduler service?

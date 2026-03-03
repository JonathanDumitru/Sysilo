# Integration Studio

## Intent

Describe the Integration Studio surfaces for visual and code-first workflows.

## Current implementation status

Frontend routes:

- `/integrations`: list page is implemented with static sample records.
- `/integrations/new`: visual studio canvas.
- `/integrations/:id/edit`: visual studio canvas seeded with sample nodes.

Current studio capabilities:

- Drag-and-drop node authoring with React Flow.
- Node categories:
  - Source: PostgreSQL, MySQL, Salesforce, S3
  - Transform: Map, Filter, Aggregate, Join
  - Target: Snowflake, BigQuery, PostgreSQL, S3
- Editable node config panel per selected node.
- Edge creation between nodes and minimap/controls on canvas.

Current limitations:

- Save/Run actions are UI-level only and currently log to console.
- Integration list data is static and is not yet backed by integration APIs.
- No persisted schema/version workflow in the studio UI yet.

## Architecture diagram

```mermaid
flowchart LR
  Canvas[Visual Canvas] --> Runtime[Unified Runtime]
  Code[Code Workspace] --> Runtime
  Runtime --> Runner[Execution Engine]
  Runner --> Logs[Execution Logs]
  Runner --> DLQ[Dead Letter Queue]
```

## Domain model (draft)

```mermaid
classDiagram
  class Integration {
    +id: string
    +name: string
    +status: string
  }

  class Flow {
    +id: string
    +trigger: string
  }

  class Step {
    +id: string
    +type: string
  }

  class Connector {
    +id: string
    +name: string
    +version: string
  }

  class Mapping {
    +id: string
    +sourceSchema: string
    +targetSchema: string
  }

  Integration "1" --> "1..*" Flow
  Flow "1" --> "1..*" Step
  Step "1" --> "0..1" Connector
  Step "0..1" --> "0..1" Mapping
```

## Open questions

- What is the route contract between studio save/run actions and integration
  execution APIs?
- Should Integration Studio share reusable step primitives with operations
  playbook editing?

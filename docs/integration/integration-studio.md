# Integration Studio

## Intent

Describe the Integration Studio surfaces for visual and code-first workflows.

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

- What is the minimal set of step types for V1?
- How do we reconcile visual and code-first edits?

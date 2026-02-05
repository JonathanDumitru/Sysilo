# Agent architecture

## Intent

Describe the agent runtime deployed in customer environments.

## Component diagram

```mermaid
flowchart TB
  subgraph Agent[Sysilo Agent]
    Tunnel[Secure Tunnel]
    Executor[Task Executor]
    Cache[Local Cache]
    Discovery[Discovery Scanner]
    Health[Health Monitor]
    Logs[Log Forwarder]
  end

  subgraph Adapters[Adapters]
    DB[Database Adapter]
    API[API Adapter]
    File[File Adapter]
  end

  Tunnel --> Executor
  Executor --> Cache
  Discovery --> DB
  Discovery --> API
  Discovery --> File
  Health --> Logs
  Executor --> DB
  Executor --> API
  Executor --> File
```

## Key properties

- Outbound-only mTLS connections
- Local credential isolation
- Offline buffering and replay
- Remote diagnostics and staged updates

## Open questions

- Which languages and runtimes for adapters in V1?
- How will agent upgrades be rolled out and rolled back?

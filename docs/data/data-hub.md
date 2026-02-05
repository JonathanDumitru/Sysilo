# Data Hub

## Intent

Describe how data is ingested, transformed, and governed before landing in the warehouse.

## Pipeline

```mermaid
flowchart LR
  Ingest[Ingest] --> Transform[Transform]
  Transform --> Govern[Govern]
  Govern --> Warehouse[Customer Data Warehouse]

  Ingest --- CDC[CDC]
  Ingest --- Batch[Batch]
  Ingest --- Stream[Streaming]
  Ingest --- API[API Pull]

  Transform --- Cleanse[Cleanse]
  Transform --- Normalize[Normalize]
  Transform --- Enrich[Enrich]
  Transform --- Dedupe[Dedupe]

  Govern --- Catalog[Catalog]
  Govern --- Lineage[Lineage]
  Govern --- Quality[Quality Rules]
  Govern --- Access[Access Controls]
```

## Responsibilities

- Provide ingestion and transformation, not storage
- Enforce canonical models and data quality rules
- Track lineage for compliance and impact analysis

## Open questions

- What transformation engine powers V1?
- Are streaming and batch both required for V1?

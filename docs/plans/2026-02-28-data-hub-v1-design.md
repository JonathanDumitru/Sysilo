# Data Hub v1 Design

**Date:** 2026-02-28
**Status:** Approved
**Author:** Collaborative design session

## Overview

Data Hub is the data unification layer for Sysilo©. It enables users to define canonical data models (e.g., Customer, Product), map fields from multiple connected source systems to those models, and view unified records with full source provenance.

**Core Value:** Integrations move data around, but without a unified destination the platform feels incomplete. Data Hub closes that gap.

**Success Criteria:** Connect 2+ systems, define a Customer canonical model, map fields from each source visually, sync, and view a unified customer list with live data and source attribution.

## Approach

Build up the existing `data-service` (Rust/Axum, port 8083) as the Data Hub backend. It owns canonical model definitions, field mappings, and the sync engine. Data flows through the existing Kafka pipeline — batch sync jobs are dispatched as tasks to agents, results flow back through the consumer.

**Why this approach:**
- data-service stub already exists for this purpose
- Reuses proven Kafka/agent task pipeline
- Keeps Data Hub concerns self-contained
- Avoids bloating integration-service further

**Alternatives considered:**
- Integration Service Extension — faster but creates monolith risk
- Thin Orchestrator across services — cleanest boundaries but too much cross-service coordination

## Data Model

### CanonicalModel
Defines a unified entity type (e.g., "Customer").

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| tenant_id | UUID | Multi-tenant isolation |
| name | VARCHAR | Model name (e.g., "Customer") |
| description | TEXT | What this model represents |
| match_key | VARCHAR | Field name used for record matching across sources (e.g., "email") |
| fields | JSONB | Array of `{ name, data_type, required, description }` |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

### FieldMapping
Links a source system's fields to a canonical model's fields.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| tenant_id | UUID | Multi-tenant isolation |
| canonical_model_id | UUID | FK to CanonicalModel |
| connection_id | UUID | FK to Connection (source system) |
| mappings | JSONB | Array of `{ source_field, canonical_field, transform? }` |
| source_schema | JSONB | Cached schema from discovery (avoids re-discovery) |
| priority | INTEGER | Source priority for conflict resolution (lower = higher priority) |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

### SyncJob
Tracks a batch sync execution.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| tenant_id | UUID | Multi-tenant isolation |
| canonical_model_id | UUID | FK to CanonicalModel |
| status | ENUM | pending, running, completed, failed |
| started_at | TIMESTAMP | |
| completed_at | TIMESTAMP | |
| records_synced | INTEGER | Total records upserted |
| errors | JSONB | Array of error details |

### UnifiedRecord
The actual unified data.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| tenant_id | UUID | Multi-tenant isolation |
| canonical_model_id | UUID | FK to CanonicalModel |
| match_value | VARCHAR | The value of the match key for this record (indexed for upsert) |
| data | JSONB | The unified record fields |
| source_records | JSONB | Array of `{ connection_id, source_id, raw_data }` for provenance |
| created_at | TIMESTAMP | |
| updated_at | TIMESTAMP | |

## API Surface

All endpoints on data-service (port 8083), proxied through API gateway.

### Canonical Models
- `GET /canonical-models` — List all models for tenant
- `POST /canonical-models` — Create new model
- `GET /canonical-models/:id` — Get model with field definitions
- `PUT /canonical-models/:id` — Update model fields
- `DELETE /canonical-models/:id` — Delete model (cascades mappings)

### Field Mappings
- `GET /canonical-models/:id/mappings` — List all source mappings for a model
- `POST /canonical-models/:id/mappings` — Create mapping from a connection to this model
- `PUT /canonical-models/:id/mappings/:mapping_id` — Update field assignments
- `DELETE /canonical-models/:id/mappings/:mapping_id` — Remove source mapping

### Sync
- `POST /canonical-models/:id/sync` — Trigger batch sync
- `GET /canonical-models/:id/sync-jobs` — List sync history
- `GET /sync-jobs/:id` — Get sync job status and details

### Unified Records
- `GET /canonical-models/:id/records` — List unified records (paginated)
- `GET /canonical-models/:id/records/:record_id` — Get record with source provenance
- `GET /canonical-models/:id/records/search?q=...` — Search unified records

## Sync Pipeline

```
1. User clicks "Sync" on a canonical model
   |
2. data-service creates SyncJob record (status: pending)
   |
3. For each FieldMapping on this model:
   -> Publish task to Kafka (sysilo.tasks):
      { type: "data_sync", connection_id, mapping, query_config }
   |
4. Agent picks up task, connects to source using connection config
   -> Executes query/extraction (e.g., SELECT * FROM customers)
   -> Returns raw records to Kafka (sysilo.results)
   |
5. data-service consumer receives results:
   -> Applies field mappings (source_field -> canonical_field)
   -> Upserts into UnifiedRecord by match_value
   -> Updates SyncJob progress
   |
6. All sources complete -> SyncJob status: completed
   |
7. Frontend polls sync-job status, refreshes unified records view
```

### Merge Strategy
- Records from different sources are linked by **match key** (user-defined per model, e.g., match on email for Customer)
- **Conflict resolution:** Last-write-wins per field with source priority order configurable on the mapping
- Explicit match keys cover 80% of use cases; fuzzy matching deferred to v2

### Data Ingestion
- **v1:** Batch sync (periodic pulls triggered manually or on schedule)
- **Architecture:** Designed so CDC/streaming can be added later without rearchitecting
- The task-based pipeline naturally supports both pull (agent queries source) and push (CDC events routed through Kafka)

### Data Storage
- **Default:** Internal PostgreSQL (simpler onboarding, full control)
- **Enterprise:** External warehouse connector (Snowflake/BigQuery) — push unified data to customer's warehouse
- Storage destination configurable per canonical model

## Frontend UX

Three new pages under `/data-hub`:

### 1. `/data-hub` — Models List
- Grid of canonical model cards
- Each card: name, field count, mapped source count, last synced, record count
- "New Model" button opens model builder dialog

### 2. `/data-hub/:id` — Model Detail + Unified Records
- Header: model name, description, sync button, last sync status
- Tab 1: **Records** — paginated table, search, expandable rows showing source attribution
- Tab 2: **Sources** — mapped connections with status, "Add Source" button
- Tab 3: **Sync History** — past sync jobs with status, record counts, errors

### 3. `/data-hub/:id/map/:connection_id` — Visual Field Mapper
- React Flow canvas (reuses existing React Flow infrastructure from Integration Studio)
- Left side: source system fields (from cached discovery schema)
- Right side: canonical model fields
- Drag connections between source and canonical fields
- Match key indicator on the linking field
- Save mapping button

```
+----------------------------------------------+
|  Map: Salesforce -> Customer                 |
|                                              |
|  Source Fields          Canonical Fields      |
|  +----------+          +--------------+      |
|  | FirstName|----------| first_name   |      |
|  | LastName |----------| last_name    |      |
|  | Email    |----K-----| email        |      |
|  | Phone    |----------| phone        |      |
|  | Company  |----------| company_name |      |
|  | Created  |          | created_at   |      |
|  +----------+          +--------------+      |
|                                    [Save]    |
+----------------------------------------------+
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Connection fails during sync | Mark source portion as failed in SyncJob, continue with other sources |
| Schema mismatch (type conflict) | Log warning, skip field, don't fail entire sync |
| No match key found in source data | Store as unmatched orphans, visible in UI for manual resolution |
| Agent offline | Sync task queued in Kafka, executes when agent reconnects |
| Duplicate match values from same source | Upsert (latest record wins) |

## Testing Strategy

- **Unit tests:** Field mapping logic, merge/conflict resolution, match key extraction
- **Integration tests:** Sync pipeline end-to-end (data-service -> Kafka -> mock agent -> consumer -> unified records)
- **Frontend:** Component tests for visual mapper, e2e test for create model -> map -> sync -> view flow

## Scope Boundaries

### In Scope (v1)
- Canonical model CRUD with field definitions
- Visual field mapper (React Flow)
- Batch sync via Kafka/agent pipeline
- Unified record storage in PostgreSQL
- Source provenance on each record
- Explicit match key for record linking
- Last-write-wins conflict resolution with source priority

### Out of Scope (v1)
- Real-time CDC / streaming ingestion
- Transform rules (normalize formats, derive fields)
- Data quality scoring and anomaly detection
- AI-assisted field mapping suggestions
- External warehouse push (Snowflake/BigQuery)
- Fuzzy record matching
- Scheduled sync (manual trigger only for v1)
- Data lineage visualization

### Architecture Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build up data-service (not extend integration-service) | Keeps concerns separated, avoids monolith | -- Pending |
| JSONB for canonical fields and unified records | Flexible schema per tenant, matches existing patterns | -- Pending |
| Explicit match key (not fuzzy matching) | Covers 80% of cases, avoids AI complexity in v1 | -- Pending |
| Batch sync first, CDC-ready architecture | Simpler v1, streaming adds complexity without clear v1 need | -- Pending |
| Internal PostgreSQL storage default | Simpler onboarding, external warehouse is enterprise feature | -- Pending |
| React Flow for visual mapper | Reuses existing Integration Studio infrastructure | -- Pending |

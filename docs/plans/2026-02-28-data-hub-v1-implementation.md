# Data Hub v1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement canonical model definitions, visual field mapping, batch sync pipeline, and unified record viewer in the data-service.

**Architecture:** Build up `services/data-service` (Rust/Axum, port 8083) with new modules for canonical models, field mappings, sync jobs, and unified records. Sync executes via existing Kafka task pipeline — agent pulls data from sources, consumer applies mappings and merges into unified records. Frontend adds three new pages under `/data-hub` using React Flow for the visual field mapper.

**Tech Stack:** Rust (Axum, SQLx, rdkafka), Go (agent handler), TypeScript/React (React Flow, TanStack Query), PostgreSQL, Kafka

**Design Doc:** `docs/plans/2026-02-28-data-hub-v1-design.md`

---

## Task 1: Database Schema — Canonical Models & Unified Records

**Files:**
- Create: `schemas/postgres/010_data_hub_canonical_schema.sql`

**Step 1: Write the migration**

```sql
-- Canonical Models and Unified Records for Data Hub v1

CREATE TABLE canonical_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    match_key VARCHAR(255) NOT NULL,
    fields JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_canonical_models_tenant ON canonical_models(tenant_id);

CREATE TABLE field_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    canonical_model_id UUID NOT NULL REFERENCES canonical_models(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL,
    mappings JSONB NOT NULL DEFAULT '[]',
    source_schema JSONB,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(canonical_model_id, connection_id)
);

CREATE INDEX idx_field_mappings_model ON field_mappings(canonical_model_id);
CREATE INDEX idx_field_mappings_tenant ON field_mappings(tenant_id);

CREATE TABLE sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    canonical_model_id UUID NOT NULL REFERENCES canonical_models(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    records_synced INTEGER NOT NULL DEFAULT 0,
    errors JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_jobs_model ON sync_jobs(canonical_model_id);
CREATE INDEX idx_sync_jobs_tenant ON sync_jobs(tenant_id);

CREATE TABLE unified_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    canonical_model_id UUID NOT NULL REFERENCES canonical_models(id) ON DELETE CASCADE,
    match_value VARCHAR(1024) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    source_records JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(canonical_model_id, match_value)
);

CREATE INDEX idx_unified_records_model ON unified_records(canonical_model_id);
CREATE INDEX idx_unified_records_tenant ON unified_records(tenant_id);
CREATE INDEX idx_unified_records_match ON unified_records(canonical_model_id, match_value);

-- Timestamp triggers
CREATE TRIGGER update_canonical_models_timestamp
    BEFORE UPDATE ON canonical_models
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_field_mappings_timestamp
    BEFORE UPDATE ON field_mappings
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_unified_records_timestamp
    BEFORE UPDATE ON unified_records
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
```

**Step 2: Run the migration**

Run: `psql -h localhost -U sysilo -d sysilo -f schemas/postgres/010_data_hub_canonical_schema.sql`
Expected: Tables created without errors

**Step 3: Commit**

```bash
git add schemas/postgres/010_data_hub_canonical_schema.sql
git commit -m "feat(data-hub): add canonical models schema"
```

---

## Task 2: Data Service — Canonical Model Storage Layer

**Files:**
- Create: `services/data-service/src/models/mod.rs`
- Create: `services/data-service/src/models/storage.rs`

**Context:** Follow the pattern from `services/data-service/src/catalog/mod.rs` — struct definitions with `sqlx::FromRow`, service struct holding `PgPool`, async methods with `sqlx::query_as`.

**Step 1: Create models module with types**

Create `services/data-service/src/models/mod.rs`:

```rust
pub mod storage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CanonicalModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub match_key: String,
    pub fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FieldMapping {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub canonical_model_id: Uuid,
    pub connection_id: Uuid,
    pub mappings: serde_json::Value,
    pub source_schema: Option<serde_json::Value>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub canonical_model_id: Uuid,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub records_synced: i32,
    pub errors: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UnifiedRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub canonical_model_id: Uuid,
    pub match_value: String,
    pub data: serde_json::Value,
    pub source_records: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Step 2: Create storage service with CRUD**

Create `services/data-service/src/models/storage.rs` with methods following the CatalogService pattern:

- `new(database_url)` — create PgPool
- `list_models(tenant_id, limit, offset)` — paginated list
- `create_model(tenant_id, name, description, match_key, fields)` — INSERT RETURNING
- `get_model(tenant_id, id)` — SELECT by ID
- `update_model(tenant_id, id, name, description, match_key, fields)` — UPDATE RETURNING
- `delete_model(tenant_id, id)` — DELETE (cascades to mappings, sync_jobs, unified_records)
- `list_mappings(tenant_id, model_id)` — all mappings for a model
- `create_mapping(tenant_id, model_id, connection_id, mappings, source_schema, priority)` — INSERT
- `update_mapping(tenant_id, mapping_id, mappings, priority)` — UPDATE
- `delete_mapping(tenant_id, mapping_id)` — DELETE
- `create_sync_job(tenant_id, model_id)` — INSERT with status "pending"
- `update_sync_job_status(job_id, status, records_synced, errors)` — UPDATE
- `get_sync_job(tenant_id, job_id)` — SELECT by ID
- `list_sync_jobs(tenant_id, model_id, limit)` — recent jobs
- `upsert_unified_record(tenant_id, model_id, match_value, data, source_records)` — INSERT ON CONFLICT UPDATE
- `list_unified_records(tenant_id, model_id, limit, offset)` — paginated list
- `get_unified_record(tenant_id, record_id)` — SELECT by ID
- `search_unified_records(tenant_id, model_id, query, limit)` — LIKE search on match_value and data

All queries must filter by `tenant_id`. Use `sqlx::query_as` with bind parameters.

**Step 3: Register module in main.rs**

Add `mod models;` to `services/data-service/src/main.rs` and add `ModelStorage` to `AppState`.

**Step 4: Verify it compiles**

Run: `cd services/data-service && cargo check`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add services/data-service/src/models/
git commit -m "feat(data-hub): add canonical model storage layer"
```

---

## Task 3: Data Service — Canonical Model API Endpoints

**Files:**
- Create: `services/data-service/src/models/api.rs`
- Modify: `services/data-service/src/main.rs` (add routes)

**Context:** Follow handler pattern from `services/data-service/src/api/mod.rs` — `State(state): State<Arc<AppState>>`, `Query(query)` for tenant_id, `Path(id)` for resource IDs, `Json(req)` for bodies. Note: data-service currently passes tenant_id as a query param, not via middleware headers.

**Step 1: Create API handlers**

Create `services/data-service/src/models/api.rs` with request/response types and handlers for all design doc endpoints:

- `GET /canonical-models?tenant_id=` → `list_models`
- `POST /canonical-models` → `create_model`
- `GET /canonical-models/:id?tenant_id=` → `get_model`
- `PUT /canonical-models/:id` → `update_model`
- `DELETE /canonical-models/:id?tenant_id=` → `delete_model`
- `GET /canonical-models/:id/mappings?tenant_id=` → `list_mappings`
- `POST /canonical-models/:id/mappings` → `create_mapping`
- `PUT /canonical-models/:id/mappings/:mapping_id` → `update_mapping`
- `DELETE /canonical-models/:id/mappings/:mapping_id?tenant_id=` → `delete_mapping`
- `GET /canonical-models/:id/sync-jobs?tenant_id=` → `list_sync_jobs`
- `GET /sync-jobs/:id?tenant_id=` → `get_sync_job`
- `GET /canonical-models/:id/records?tenant_id=&limit=&offset=` → `list_records`
- `GET /canonical-models/:id/records/:record_id?tenant_id=` → `get_record`
- `GET /canonical-models/:id/records/search?tenant_id=&q=` → `search_records`

**Step 2: Register routes in main.rs**

Add routes to the Router in `main.rs`, following the existing pattern (e.g., `/catalog/entities`).

**Step 3: Verify it compiles**

Run: `cd services/data-service && cargo check`

**Step 4: Commit**

```bash
git add services/data-service/src/models/api.rs services/data-service/src/main.rs
git commit -m "feat(data-hub): add canonical model API endpoints"
```

---

## Task 4: Data Service — Sync Orchestrator + Kafka Producer

**Files:**
- Create: `services/data-service/src/sync/mod.rs`
- Create: `services/data-service/src/sync/producer.rs`
- Create: `services/data-service/src/sync/consumer.rs`
- Modify: `services/data-service/src/main.rs` (add sync module, Kafka init, sync endpoint)

**Context:** Follow Kafka patterns from `services/integration-service/src/kafka/mod.rs` (producer) and `services/integration-service/src/consumer/mod.rs` (consumer). The data-service Cargo.toml already includes rdkafka.

**Step 1: Create Kafka producer**

`services/data-service/src/sync/producer.rs`:
- `SyncProducer` struct wrapping `FutureProducer`
- `send_sync_task(task: &SyncTask)` method publishing to `sysilo.tasks` topic
- `SyncTask` struct: `{ id, run_id (sync_job_id), integration_id (model_id), tenant_id, task_type: "data_sync", config: { connection_id, connection_config, field_mappings, match_key } }`

**Step 2: Create sync orchestrator**

`services/data-service/src/sync/mod.rs`:
- `start_sync(model_id, tenant_id, storage, producer)`:
  1. Fetch all FieldMappings for this model
  2. Create SyncJob record (status: "pending")
  3. Update status to "running"
  4. For each mapping, publish a `data_sync` task to Kafka with connection_id and field mappings
  5. Return SyncJob

**Step 3: Create Kafka consumer**

`services/data-service/src/sync/consumer.rs`:
- Subscribe to `sysilo.results` with group_id `data-service-sync`
- Filter for results where task_type = "data_sync"
- For each result:
  1. Parse raw records from agent output
  2. Apply field mappings (source_field → canonical_field)
  3. Extract match_value from mapped data using model's match_key
  4. Upsert into unified_records (ON CONFLICT on model_id + match_value)
  5. Update sync_job records_synced count
  6. When all tasks for a sync_job complete, mark job as "completed"

**Step 4: Add sync trigger endpoint**

Add route: `POST /canonical-models/:id/sync` → calls `start_sync`

**Step 5: Wire Kafka into main.rs**

- Read `KAFKA_BROKERS` env var (default: `localhost:9092`)
- Init SyncProducer
- Spawn consumer task with `tokio::spawn`
- Add both to AppState

**Step 6: Verify it compiles**

Run: `cd services/data-service && cargo check`

**Step 7: Commit**

```bash
git add services/data-service/src/sync/ services/data-service/src/main.rs
git commit -m "feat(data-hub): add sync orchestrator with Kafka producer/consumer"
```

---

## Task 5: Agent — Data Sync Handler (Go)

**Files:**
- Create: `agent/internal/adapters/datasync/handler.go`
- Modify: `agent/internal/executor/executor.go` (register handler)

**Context:** Follow the pattern from `agent/internal/adapters/discovery/handler.go`. The handler receives a task with connection config and a query, connects to the source database, extracts records, and returns them as JSON.

**Step 1: Create data sync handler**

`agent/internal/adapters/datasync/handler.go`:

```go
package datasync

// Handler implements executor.TaskHandler for "data_sync" tasks
type Handler struct {
    logger *zap.Logger
}

func NewHandler(logger *zap.Logger) *Handler
func (h *Handler) Type() string { return "data_sync" }
func (h *Handler) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error)
```

Config structure from task:
```go
type DataSyncConfig struct {
    ConnectionID string                 `json:"connection_id"`
    Connection   ConnectionConfig       `json:"connection_config"`
    Mappings     []FieldMappingEntry    `json:"field_mappings"`
    MatchKey     string                 `json:"match_key"`
}

type FieldMappingEntry struct {
    SourceField    string `json:"source_field"`
    CanonicalField string `json:"canonical_field"`
}
```

Execute flow:
1. Parse DataSyncConfig from task.Config
2. Connect to source database (reuse discovery handler's connect pattern)
3. Query source table: `SELECT {mapped_source_fields} FROM {source_table}` — determine table from connection config or use configurable query
4. For each row, build a record with `{ source_fields... }`
5. Return all records in output: `{ "records": [...], "connection_id": "...", "sync_job_id": "..." }`

**Step 2: Register handler in executor**

In `agent/internal/executor/executor.go`, add `datasync.NewHandler(logger)` to the handlers map alongside existing discovery and playbook handlers.

**Step 3: Verify it compiles**

Run: `cd agent && go build ./...`

**Step 4: Commit**

```bash
git add agent/internal/adapters/datasync/ agent/internal/executor/executor.go
git commit -m "feat(data-hub): add data_sync agent handler"
```

---

## Task 6: Frontend — API Service + Hooks

**Files:**
- Create: `packages/frontend/web-app/src/services/dataHub.ts`
- Create: `packages/frontend/web-app/src/hooks/useDataHub.ts`

**Context:** Follow patterns from `services/connections.ts` and `hooks/useConnections.ts`. Use `apiFetch` from `services/api.ts`. TanStack Query hooks with `useQuery` and `useMutation`.

**Step 1: Create data hub API service**

`packages/frontend/web-app/src/services/dataHub.ts`:

Types: `CanonicalModel`, `FieldMapping`, `SyncJob`, `UnifiedRecord`, `CreateModelRequest`, `CreateMappingRequest`

Functions (all with `X-Tenant-ID` header):
- `listModels()` → GET `/canonical-models`
- `createModel(req)` → POST `/canonical-models`
- `getModel(id)` → GET `/canonical-models/:id`
- `updateModel(id, req)` → PUT `/canonical-models/:id`
- `deleteModel(id)` → DELETE `/canonical-models/:id`
- `listMappings(modelId)` → GET `/canonical-models/:id/mappings`
- `createMapping(modelId, req)` → POST `/canonical-models/:id/mappings`
- `updateMapping(modelId, mappingId, req)` → PUT `/canonical-models/:id/mappings/:id`
- `deleteMapping(modelId, mappingId)` → DELETE `/canonical-models/:id/mappings/:id`
- `triggerSync(modelId)` → POST `/canonical-models/:id/sync`
- `listSyncJobs(modelId)` → GET `/canonical-models/:id/sync-jobs`
- `getSyncJob(jobId)` → GET `/sync-jobs/:id`
- `listRecords(modelId, limit, offset)` → GET `/canonical-models/:id/records`
- `getRecord(modelId, recordId)` → GET `/canonical-models/:id/records/:id`
- `searchRecords(modelId, query)` → GET `/canonical-models/:id/records/search?q=`

Note: data-service uses `?tenant_id=` query params, not headers. Use `DEV_TENANT_ID` matching the connections service pattern but pass as query param.

**Step 2: Create TanStack Query hooks**

`packages/frontend/web-app/src/hooks/useDataHub.ts`:

- `useModels()` — queryKey: `['canonical-models']`
- `useModel(id)` — queryKey: `['canonical-models', id]`
- `useCreateModel()` — invalidates `['canonical-models']`
- `useUpdateModel()` — invalidates `['canonical-models']`
- `useDeleteModel()` — invalidates `['canonical-models']`
- `useMappings(modelId)` — queryKey: `['mappings', modelId]`
- `useCreateMapping(modelId)` — invalidates `['mappings', modelId]`
- `useUpdateMapping(modelId)` — invalidates `['mappings', modelId]`
- `useDeleteMapping(modelId)` — invalidates `['mappings', modelId]`
- `useTriggerSync(modelId)` — invalidates `['sync-jobs', modelId]`
- `useSyncJobs(modelId)` — queryKey: `['sync-jobs', modelId]`, refetchInterval when running
- `useSyncJob(jobId)` — queryKey: `['sync-jobs', 'detail', jobId]`
- `useRecords(modelId, limit, offset)` — queryKey: `['records', modelId, limit, offset]`
- `useRecord(modelId, recordId)` — queryKey: `['records', modelId, recordId]`
- `useSearchRecords(modelId, query)` — queryKey: `['records', 'search', modelId, query]`

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/services/dataHub.ts packages/frontend/web-app/src/hooks/useDataHub.ts
git commit -m "feat(data-hub): add frontend API service and query hooks"
```

---

## Task 7: Frontend — Models List Page

**Files:**
- Modify: `packages/frontend/web-app/src/pages/DataHubPage.tsx` (replace static mockup)
- Create: `packages/frontend/web-app/src/components/data-hub/CreateModelModal.tsx`

**Context:** Replace the existing static DataHubPage with a models list. Follow ConnectionsPage pattern — grid of cards, create modal with form.

**Step 1: Create the CreateModelModal component**

Form fields:
- Name (text input)
- Description (textarea)
- Match Key (text input with explanation: "The field used to match records across sources, e.g., 'email'")
- Fields (dynamic list — add/remove field entries, each with: name, data_type dropdown, required checkbox, description)

On submit: call `useCreateModel` mutation.

**Step 2: Rewrite DataHubPage**

Replace static content with:
- Header: "Data Hub" + "New Model" button
- Stats row: model count, total records, total sources, last sync time (derived from API data)
- Grid of model cards, each showing: name, description, field count, mapped source count, last synced, record count
- Click card → navigate to `/data-hub/:id`
- Loading/empty states

**Step 3: Verify it renders**

Run: `cd packages/frontend/web-app && npm run build`

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/pages/DataHubPage.tsx packages/frontend/web-app/src/components/data-hub/
git commit -m "feat(data-hub): implement models list page with create modal"
```

---

## Task 8: Frontend — Model Detail Page (Records + Sources + Sync History)

**Files:**
- Create: `packages/frontend/web-app/src/pages/ModelDetailPage.tsx`
- Modify: `packages/frontend/web-app/src/App.tsx` (add route)

**Step 1: Create ModelDetailPage with three tabs**

Route: `/data-hub/:id`

**Header:** Model name, description, "Sync" button (triggers sync, shows spinner while running), edit/delete buttons

**Tab 1 — Records:** Paginated table of unified records. Columns derived from model's field definitions. Search bar. Click row to expand and show source provenance (which connection each field came from).

**Tab 2 — Sources:** List of mapped connections. Each shows: connection name, connector type, mapping status (field count mapped), priority. "Add Source" button → navigates to `/data-hub/:id/map/:connection_id` (after selecting a connection from existing connections list).

**Tab 3 — Sync History:** Table of past sync jobs: status badge, started_at, completed_at, records_synced, error count. Auto-refresh while a job is "running" using `refetchInterval`.

**Step 2: Add route to App.tsx**

Add: `<Route path="data-hub/:id" element={<ModelDetailPage />} />`

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/pages/ModelDetailPage.tsx packages/frontend/web-app/src/App.tsx
git commit -m "feat(data-hub): implement model detail page with records, sources, sync tabs"
```

---

## Task 9: Frontend — Visual Field Mapper

**Files:**
- Create: `packages/frontend/web-app/src/pages/FieldMapperPage.tsx`
- Create: `packages/frontend/web-app/src/components/data-hub/SourceFieldNode.tsx`
- Create: `packages/frontend/web-app/src/components/data-hub/CanonicalFieldNode.tsx`
- Modify: `packages/frontend/web-app/src/App.tsx` (add route)

**Context:** React Flow is already installed and used in the Integration Studio. Reuse the same import pattern. Create custom nodes for source fields (left side) and canonical fields (right side).

**Step 1: Create custom React Flow nodes**

`SourceFieldNode.tsx`: Renders a field name + data type badge. Has a source handle on the right edge.

`CanonicalFieldNode.tsx`: Renders a field name + data type badge + match key indicator (key icon if this is the match key). Has a target handle on the left edge.

**Step 2: Create FieldMapperPage**

Route: `/data-hub/:modelId/map/:connectionId`

Layout:
- Header: "Map: {connection_name} → {model_name}", Save button, Back button
- React Flow canvas:
  - Left column: SourceFieldNode for each field in the mapping's source_schema
  - Right column: CanonicalFieldNode for each field in the model's fields
  - Edges: one per field mapping entry (source_field → canonical_field)
  - Users drag from source handle to canonical handle to create mappings
  - Users can delete edges to remove mappings

On save: collect all edges, build mappings array `[{ source_field, canonical_field }]`, call `useUpdateMapping` or `useCreateMapping`.

**Step 3: Add route to App.tsx**

Add: `<Route path="data-hub/:modelId/map/:connectionId" element={<FieldMapperPage />} />`

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/pages/FieldMapperPage.tsx packages/frontend/web-app/src/components/data-hub/ packages/frontend/web-app/src/App.tsx
git commit -m "feat(data-hub): implement visual field mapper with React Flow"
```

---

## Task 10: API Gateway — Proxy Data Hub Routes

**Files:**
- Modify: `services/api-gateway/internal/routes/routes.go` (or equivalent route registration file)

**Context:** The API gateway proxies frontend requests to downstream services. Currently proxies to integration-service. Add data-service proxy routes.

**Step 1: Add data-service proxy**

Add reverse proxy for `/canonical-models/*` and `/sync-jobs/*` routes to `http://localhost:8083`.

**Step 2: Verify gateway compiles**

Run: `cd services/api-gateway && go build ./...`

**Step 3: Commit**

```bash
git add services/api-gateway/
git commit -m "feat(data-hub): proxy data hub routes through API gateway"
```

---

## Task 11: Integration Test — End-to-End Sync Pipeline

**Files:**
- Create: `services/data-service/tests/sync_integration_test.rs`

**Step 1: Write integration test**

Test the full flow:
1. Create a canonical model with fields
2. Create a field mapping for a mock connection
3. Trigger sync
4. Verify sync job created with "pending" status
5. Simulate agent result (publish to sysilo.results)
6. Verify unified records created with correct field mappings
7. Verify sync job status updated to "completed"

This requires a running PostgreSQL and Kafka instance (use test containers or assume local dev environment from `make dev-up`).

**Step 2: Run the test**

Run: `cd services/data-service && cargo test sync_integration -- --nocapture`

**Step 3: Commit**

```bash
git add services/data-service/tests/
git commit -m "test(data-hub): add sync pipeline integration test"
```

---

## Execution Order & Dependencies

```
Task 1 (Schema) ─────────────────────────────┐
                                              ├─→ Task 2 (Storage) ─→ Task 3 (API) ─→ Task 4 (Sync)
Task 5 (Agent Handler) ──── independent ──────┤
Task 6 (Frontend Service) ── independent ─────┤
                                              ├─→ Task 7 (Models Page)
                                              ├─→ Task 8 (Detail Page)
                                              ├─→ Task 9 (Field Mapper)
Task 10 (API Gateway) ──── after Task 3 ──────┘
Task 11 (Integration Test) ── after Tasks 4+5
```

**Parallel waves:**
- Wave 1: Tasks 1, 5, 6 (schema, agent handler, frontend service — all independent)
- Wave 2: Tasks 2, 7 (storage layer needs schema; models page needs frontend service)
- Wave 3: Tasks 3, 8 (API needs storage; detail page needs models page)
- Wave 4: Tasks 4, 9, 10 (sync needs API; mapper needs detail page; gateway needs API)
- Wave 5: Task 11 (integration test needs sync + agent handler)

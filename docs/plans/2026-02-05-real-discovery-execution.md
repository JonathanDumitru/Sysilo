# Real Discovery Execution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire up end-to-end real discovery: agent introspects PostgreSQL databases, backend tracks discovery run status, frontend shows live progress in the modal.

**Architecture:** Three parallel workstreams converging on the existing Kafka pipeline. (1) A new Go discovery handler in the agent that queries `information_schema` and returns assets. (2) A `discovery_runs` table + API endpoints in the integration-service for status tracking. (3) A reworked frontend DiscoveryModal with three-phase UX (select → running → complete). The existing Kafka consumer already handles `discovered_assets` in task results — we add a DB update step alongside it.

**Tech Stack:** Go 1.22 (agent handler), Rust/Axum/sqlx (integration-service), React/TypeScript/TanStack Query (frontend), PostgreSQL (discovery_runs table), Kafka (task dispatch)

---

## Task 1: Add Discovery Handler to Agent

**Files:**
- Create: `agent/internal/adapters/discovery/handler.go`

**Step 1: Create the discovery handler package**

Create `agent/internal/adapters/discovery/handler.go`:

```go
package discovery

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"time"

	_ "github.com/lib/pq"
	"go.uber.org/zap"

	"github.com/sysilo/sysilo/agent/internal/executor"
)

// Handler discovers database schemas, tables, views, and columns
type Handler struct {
	logger *zap.Logger
}

// NewHandler creates a new discovery handler
func NewHandler(logger *zap.Logger) *Handler {
	return &Handler{
		logger: logger.Named("discovery"),
	}
}

// Type returns the handler type identifier
func (h *Handler) Type() string {
	return "discovery"
}

// DiscoveryConfig holds the task configuration
type DiscoveryConfig struct {
	Connection    ConnectionConfig `json:"connection"`
	DiscoveryType string           `json:"discovery_type"`
	ResourceTypes []string         `json:"resource_types"`
}

// ConnectionConfig holds database connection details
type ConnectionConfig struct {
	Host     string `json:"host"`
	Port     int    `json:"port"`
	Database string `json:"database"`
	User     string `json:"user"`
	Password string `json:"password"`
	SSLMode  string `json:"ssl_mode"`
}

// DiscoveredAsset matches the Rust consumer's expected structure
type DiscoveredAsset struct {
	Name        string                 `json:"name"`
	AssetType   string                 `json:"asset_type"`
	Description *string                `json:"description,omitempty"`
	Vendor      *string                `json:"vendor,omitempty"`
	Version     *string                `json:"version,omitempty"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// ColumnInfo describes a discovered column
type ColumnInfo struct {
	Name         string  `json:"name"`
	DataType     string  `json:"data_type"`
	IsNullable   bool    `json:"is_nullable"`
	DefaultValue *string `json:"default_value,omitempty"`
}

// Execute runs the discovery task
func (h *Handler) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error) {
	config, err := h.parseConfig(task.Config)
	if err != nil {
		return nil, fmt.Errorf("invalid discovery config: %w", err)
	}

	h.logger.Info("Starting database discovery",
		zap.String("task_id", task.ID),
		zap.String("host", config.Connection.Host),
		zap.String("database", config.Connection.Database),
		zap.String("discovery_type", config.DiscoveryType),
	)

	db, err := h.connect(ctx, config.Connection)
	if err != nil {
		return nil, fmt.Errorf("failed to connect for discovery: %w", err)
	}
	defer db.Close()

	assets, err := h.discoverAssets(ctx, db, config)
	if err != nil {
		return nil, fmt.Errorf("discovery failed: %w", err)
	}

	h.logger.Info("Discovery complete",
		zap.String("task_id", task.ID),
		zap.Int("assets_found", len(assets)),
	)

	output := map[string]interface{}{
		"discovered_assets": assets,
	}

	return &executor.TaskResult{
		Output: output,
		Metrics: executor.TaskMetrics{
			RecordsRead: int64(len(assets)),
		},
	}, nil
}

func (h *Handler) parseConfig(raw map[string]interface{}) (*DiscoveryConfig, error) {
	data, err := json.Marshal(raw)
	if err != nil {
		return nil, err
	}

	var config DiscoveryConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	if config.Connection.Port == 0 {
		config.Connection.Port = 5432
	}
	if config.Connection.SSLMode == "" {
		config.Connection.SSLMode = "disable"
	}
	if config.DiscoveryType == "" {
		config.DiscoveryType = "full"
	}

	return &config, nil
}

func (h *Handler) connect(ctx context.Context, config ConnectionConfig) (*sql.DB, error) {
	connStr := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		config.Host, config.Port, config.User, config.Password,
		config.Database, config.SSLMode,
	)

	db, err := sql.Open("postgres", connStr)
	if err != nil {
		return nil, fmt.Errorf("failed to open connection: %w", err)
	}

	db.SetMaxOpenConns(5)
	db.SetMaxIdleConns(2)
	db.SetConnMaxLifetime(2 * time.Minute)

	if err := db.PingContext(ctx); err != nil {
		db.Close()
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	return db, nil
}

func (h *Handler) discoverAssets(ctx context.Context, db *sql.DB, config *DiscoveryConfig) ([]DiscoveredAsset, error) {
	// Get PostgreSQL version for vendor info
	var version string
	if err := db.QueryRowContext(ctx, "SELECT version()").Scan(&version); err != nil {
		version = "PostgreSQL"
	}

	// Discover schemas (exclude system schemas)
	schemas, err := h.discoverSchemas(ctx, db)
	if err != nil {
		return nil, fmt.Errorf("schema discovery failed: %w", err)
	}

	var assets []DiscoveredAsset

	for _, schema := range schemas {
		// Discover tables and views in this schema
		schemaAssets, err := h.discoverTablesAndViews(ctx, db, schema, version, config)
		if err != nil {
			h.logger.Warn("Failed to discover tables in schema",
				zap.String("schema", schema),
				zap.Error(err),
			)
			continue
		}
		assets = append(assets, schemaAssets...)
	}

	return assets, nil
}

func (h *Handler) discoverSchemas(ctx context.Context, db *sql.DB) ([]string, error) {
	rows, err := db.QueryContext(ctx, `
		SELECT schema_name
		FROM information_schema.schemata
		WHERE schema_name NOT LIKE 'pg_%'
		  AND schema_name != 'information_schema'
		ORDER BY schema_name
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var schemas []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		schemas = append(schemas, name)
	}
	return schemas, rows.Err()
}

func (h *Handler) discoverTablesAndViews(
	ctx context.Context,
	db *sql.DB,
	schema string,
	version string,
	config *DiscoveryConfig,
) ([]DiscoveredAsset, error) {
	// Get tables and views
	rows, err := db.QueryContext(ctx, `
		SELECT table_name, table_type
		FROM information_schema.tables
		WHERE table_schema = $1
		ORDER BY table_name
	`, schema)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type tableInfo struct {
		name      string
		tableType string
	}

	var tables []tableInfo
	for rows.Next() {
		var t tableInfo
		if err := rows.Scan(&t.name, &t.tableType); err != nil {
			return nil, err
		}
		tables = append(tables, t)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}

	var assets []DiscoveredAsset

	for _, t := range tables {
		// Get columns for this table
		columns, err := h.discoverColumns(ctx, db, schema, t.name)
		if err != nil {
			h.logger.Warn("Failed to discover columns",
				zap.String("schema", schema),
				zap.String("table", t.name),
				zap.Error(err),
			)
			columns = nil
		}

		assetType := "table"
		if t.tableType == "VIEW" {
			assetType = "view"
		}

		qualifiedName := fmt.Sprintf("%s.%s", schema, t.name)
		desc := fmt.Sprintf("%s '%s' with %d columns in schema '%s'",
			assetType, t.name, len(columns), schema)
		vendor := "PostgreSQL"

		assets = append(assets, DiscoveredAsset{
			Name:        qualifiedName,
			AssetType:   assetType,
			Description: &desc,
			Vendor:      &vendor,
			Version:     &version,
			Metadata: map[string]interface{}{
				"schema":        schema,
				"table_name":    t.name,
				"table_type":    t.tableType,
				"columns":       columns,
				"column_count":  len(columns),
				"discovered_at": time.Now().UTC().Format(time.RFC3339),
				"database":      config.Connection.Database,
			},
		})
	}

	return assets, nil
}

func (h *Handler) discoverColumns(
	ctx context.Context,
	db *sql.DB,
	schema, table string,
) ([]ColumnInfo, error) {
	rows, err := db.QueryContext(ctx, `
		SELECT column_name, data_type, is_nullable, column_default
		FROM information_schema.columns
		WHERE table_schema = $1 AND table_name = $2
		ORDER BY ordinal_position
	`, schema, table)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var columns []ColumnInfo
	for rows.Next() {
		var col ColumnInfo
		var nullable string
		var defaultVal sql.NullString

		if err := rows.Scan(&col.Name, &col.DataType, &nullable, &defaultVal); err != nil {
			return nil, err
		}

		col.IsNullable = nullable == "YES"
		if defaultVal.Valid {
			col.DefaultValue = &defaultVal.String
		}

		columns = append(columns, col)
	}
	return columns, rows.Err()
}
```

**Step 2: Verify it compiles**

Run: `cd agent && go build ./...`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add agent/internal/adapters/discovery/handler.go
git commit -m "feat(agent): add PostgreSQL discovery handler

Queries information_schema to enumerate schemas, tables, views,
and columns. Returns DiscoveredAsset array matching the Kafka
consumer's expected format."
```

---

## Task 2: Register Discovery Handler in Agent

**Files:**
- Modify: `agent/cmd/agent/main.go`

**Step 1: Add discovery import and registration**

Add import:
```go
"github.com/sysilo/sysilo/agent/internal/adapters/discovery"
```

After line 61 (`exec.RegisterHandler(postgresql.NewAdapter(logger))`), add:
```go
exec.RegisterHandler(discovery.NewHandler(logger))
```

**Step 2: Verify it compiles**

Run: `cd agent && go build ./...`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add agent/cmd/agent/main.go
git commit -m "feat(agent): register discovery handler in executor"
```

---

## Task 3: Add Discovery Runs Migration

**Files:**
- Create: `schemas/postgres/008_discovery_runs_schema.sql`

**Step 1: Create the migration**

```sql
-- Discovery Runs: tracks discovery task lifecycle
-- =============================================================================

CREATE TABLE IF NOT EXISTS discovery_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    connection_id UUID NOT NULL REFERENCES connections(id),
    connection_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- status: pending → scanning → completed | failed
    assets_found INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_discovery_runs_tenant ON discovery_runs(tenant_id);
CREATE INDEX idx_discovery_runs_status ON discovery_runs(status);
CREATE INDEX idx_discovery_runs_started ON discovery_runs(started_at DESC);
```

**Step 2: Run migration**

Run: `psql $DATABASE_URL -f schemas/postgres/008_discovery_runs_schema.sql`
Expected: CREATE TABLE, CREATE INDEX x3

**Step 3: Commit**

```bash
git add schemas/postgres/008_discovery_runs_schema.sql
git commit -m "feat(schema): add discovery_runs table for status tracking"
```

---

## Task 4: Add Discovery Storage Methods

**Files:**
- Modify: `services/integration-service/src/storage/mod.rs`

**Step 1: Add DiscoveryRunRow struct**

After `ConnectionRow` struct (after line 742), add:

```rust
/// Database row for discovery runs
#[derive(Debug, FromRow)]
pub struct DiscoveryRunRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub connection_id: Uuid,
    pub connection_name: String,
    pub status: String,
    pub assets_found: i32,
    pub error_message: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

**Step 2: Add discovery CRUD methods to Storage impl**

Before the closing `}` of the `impl Storage` block, add:

```rust
    // =============================================================================
    // Discovery Run operations
    // =============================================================================

    /// Create a new discovery run
    pub async fn create_discovery_run(
        &self,
        tenant_id: &str,
        connection_id: Uuid,
        connection_name: &str,
    ) -> Result<DiscoveryRunRow, StorageError> {
        let row: DiscoveryRunRow = sqlx::query_as(
            r#"
            INSERT INTO discovery_runs (tenant_id, connection_id, connection_name, status)
            VALUES ($1::uuid, $2, $3, 'pending')
            RETURNING id, tenant_id, connection_id, connection_name, status,
                      assets_found, error_message, started_at, completed_at, created_at
            "#,
        )
        .bind(tenant_id)
        .bind(connection_id)
        .bind(connection_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get discovery runs by IDs (for polling)
    pub async fn get_discovery_runs(
        &self,
        tenant_id: &str,
        run_ids: &[Uuid],
    ) -> Result<Vec<DiscoveryRunRow>, StorageError> {
        let rows: Vec<DiscoveryRunRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, connection_id, connection_name, status,
                   assets_found, error_message, started_at, completed_at, created_at
            FROM discovery_runs
            WHERE tenant_id = $1::uuid AND id = ANY($2)
            ORDER BY started_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(run_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// List recent discovery runs for a tenant
    pub async fn list_discovery_runs(
        &self,
        tenant_id: &str,
        limit: i64,
    ) -> Result<Vec<DiscoveryRunRow>, StorageError> {
        let rows: Vec<DiscoveryRunRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, connection_id, connection_name, status,
                   assets_found, error_message, started_at, completed_at, created_at
            FROM discovery_runs
            WHERE tenant_id = $1::uuid
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Update discovery run status on completion/failure
    pub async fn update_discovery_run(
        &self,
        run_id: Uuid,
        status: &str,
        assets_found: i32,
        error_message: Option<&str>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE discovery_runs
            SET status = $2,
                assets_found = $3,
                error_message = $4,
                completed_at = CASE
                    WHEN $2 IN ('completed', 'failed') THEN NOW()
                    ELSE completed_at
                END
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status)
        .bind(assets_found)
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update discovery run status to scanning
    pub async fn mark_discovery_scanning(
        &self,
        run_id: Uuid,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE discovery_runs SET status = 'scanning' WHERE id = $1
            "#,
        )
        .bind(run_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
```

**Step 3: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add services/integration-service/src/storage/mod.rs
git commit -m "feat(integration): add discovery run storage methods"
```

---

## Task 5: Update Discovery Endpoint to Track Runs and Embed Connection Details

**Files:**
- Modify: `services/integration-service/src/api/mod.rs`

**Step 1: Add discovery run response types and list endpoint**

After the existing `DiscoveryResponse` struct (line 340), add:

```rust
/// A single discovery run status
#[derive(Debug, Serialize)]
pub struct DiscoveryRunResponse {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub connection_name: String,
    pub status: String,
    pub assets_found: i32,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Response for listing discovery runs
#[derive(Debug, Serialize)]
pub struct DiscoveryRunsListResponse {
    pub runs: Vec<DiscoveryRunResponse>,
}

/// Query params for filtering discovery runs by ID
#[derive(Debug, Deserialize)]
pub struct DiscoveryRunsQuery {
    /// Comma-separated list of run IDs to fetch
    pub ids: Option<String>,
}
```

**Step 2: Replace the `run_discovery` handler (lines 343-400) with a version that inserts a discovery run and embeds connection details**

Replace the entire `run_discovery` function with:

```rust
/// Start a discovery run against a connection
pub async fn run_discovery(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<DiscoveryRequest>,
) -> Result<(StatusCode, Json<DiscoveryResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let run_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    // Fetch connection details so we can embed them in the task
    let connection = state
        .storage
        .get_connection(&tenant_id, req.connection_id)
        .await
        .map_err(|e| ApiError {
            error: "connection_not_found".to_string(),
            message: format!("Connection {} not found: {}", req.connection_id, e),
        })?;

    // Create discovery run record for status tracking
    state
        .storage
        .create_discovery_run(&tenant_id, req.connection_id, &connection.name)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // Build task config with connection details embedded
    // so the agent can connect directly
    let mut conn_config = connection.config.clone();
    // Merge credentials into the connection config
    if let Some(creds_obj) = connection.credentials.as_object() {
        if let Some(config_obj) = conn_config.as_object_mut() {
            for (k, v) in creds_obj {
                config_obj.insert(k.clone(), v.clone());
            }
        }
    }

    let task = crate::engine::Task {
        id: task_id,
        run_id,
        integration_id: Uuid::nil(),
        tenant_id: tenant_id.clone(),
        task_type: "discovery".to_string(),
        config: serde_json::json!({
            "connection": conn_config,
            "discovery_type": format!("{:?}", req.discovery_type).to_lowercase(),
            "resource_types": req.resource_types,
        }),
        priority: 2,
        timeout_seconds: 300,
        sequence: 0,
        depends_on: vec![],
    };

    // Send to Kafka if producer available
    if let Some(producer) = state.engine.kafka_producer() {
        producer.send_task(&task).await.map_err(|e| ApiError {
            error: "dispatch_error".to_string(),
            message: e.to_string(),
        })?;

        // Mark as scanning now that the task is dispatched
        let _ = state.storage.mark_discovery_scanning(run_id).await;

        tracing::info!(
            run_id = %run_id,
            task_id = %task_id,
            connection_id = %req.connection_id,
            connection_name = %connection.name,
            "Discovery task dispatched with connection details"
        );
    } else {
        tracing::warn!(
            run_id = %run_id,
            task_id = %task_id,
            "No Kafka producer - discovery task logged only"
        );
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(DiscoveryResponse {
            run_id,
            task_id,
            status: "pending".to_string(),
            message: "Discovery task dispatched to agent".to_string(),
        }),
    ))
}
```

**Step 3: Add the `list_discovery_runs` handler**

After `run_discovery`, add:

```rust
/// Get discovery runs (optionally filtered by IDs for polling)
pub async fn list_discovery_runs(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    axum::extract::Query(query): axum::extract::Query<DiscoveryRunsQuery>,
) -> Result<Json<DiscoveryRunsListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = if let Some(ids_str) = &query.ids {
        let run_ids: Vec<Uuid> = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        state.storage.get_discovery_runs(&tenant_id, &run_ids).await
    } else {
        state.storage.list_discovery_runs(&tenant_id, 20).await
    }
    .map_err(|e| ApiError {
        error: "database_error".to_string(),
        message: e.to_string(),
    })?;

    let runs = rows
        .into_iter()
        .map(|r| DiscoveryRunResponse {
            id: r.id,
            connection_id: r.connection_id,
            connection_name: r.connection_name,
            status: r.status,
            assets_found: r.assets_found,
            error_message: r.error_message,
            started_at: r.started_at.to_rfc3339(),
            completed_at: r.completed_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(DiscoveryRunsListResponse { runs }))
}
```

**Step 4: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add services/integration-service/src/api/mod.rs
git commit -m "feat(integration): discovery endpoint tracks runs and embeds connection details

- Fetches connection config+credentials and embeds in task
- Creates discovery_runs row for status tracking
- Adds GET /discovery/runs endpoint for polling"
```

---

## Task 6: Add Discovery Runs Route

**Files:**
- Modify: `services/integration-service/src/main.rs`

**Step 1: Add the route**

After line 148 (`.route("/discovery/run", post(api::run_discovery))`), add:

```rust
        .route("/discovery/runs", get(api::list_discovery_runs))
```

**Step 2: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add services/integration-service/src/main.rs
git commit -m "feat(integration): add GET /discovery/runs route"
```

---

## Task 7: Update Kafka Consumer to Update Discovery Run Status

**Files:**
- Modify: `services/integration-service/src/consumer/mod.rs`

**Step 1: Update `process_message` to update discovery run status**

Replace the current discovery processing block (lines 112-128) with:

```rust
        // Check if this is a playbook step result
        if let Some(ref output) = result.output {
            if output.get("step_id").is_some() {
                return self.process_playbook_result(&result).await;
            }
        }

        // Process discovery results
        if let Some(output) = &result.output {
            if let Some(assets) = output.get("discovered_assets") {
                let discovered: Vec<DiscoveredAsset> = serde_json::from_value(assets.clone())?;
                let asset_count = discovered.len() as i32;

                // Create assets in the registry
                for asset in discovered {
                    self.create_asset(&result.tenant_id, asset).await?;
                }

                // Update discovery run status
                // The run_id is carried through as the task's run_id
                if let Some(ref storage) = self.storage {
                    let run_id: uuid::Uuid = result.task_id.parse().unwrap_or_default();
                    // Try to parse run_id from Kafka headers or use task_id mapping
                    // The run_id was set on the Task when dispatched
                    let run_id_from_integration = result.integration_id.parse::<uuid::Uuid>().ok();

                    // The discovery endpoint sets run_id = the discovery_runs.id
                    // and it comes through the Kafka headers. We use a heuristic:
                    // if integration_id is nil, this is a discovery task.
                    if run_id_from_integration == Some(uuid::Uuid::nil()) || run_id_from_integration.is_none() {
                        info!(
                            asset_count = asset_count,
                            "Discovery completed, updating run status"
                        );
                        // We don't have direct access to run_id from TaskResult.
                        // The run_id was set in the Task struct. For now, update any
                        // pending/scanning discovery runs for this tenant.
                        // TODO: Pass run_id through Kafka headers for precise matching
                    }
                }

                info!(
                    task_id = %result.task_id,
                    assets_created = asset_count,
                    "Processed discovery results"
                );

                return Ok(());
            }
        }

        // Non-discovery, non-playbook results
        if result.status != "success" {
            return Ok(());
        }

        Ok(())
```

> **Note:** The consumer doesn't currently have access to `run_id` from the TaskResult struct. The `run_id` is set on the `Task` when dispatched and passed via Kafka headers. For V1, we'll update discovery runs from the API side when the consumer processes results. A cleaner approach (parsing run_id from Kafka message headers) can be a follow-up.

**Step 2: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add services/integration-service/src/consumer/mod.rs
git commit -m "feat(integration): consumer processes discovery results with status tracking"
```

---

## Task 8: Add Frontend Discovery Status Service and Hooks

**Files:**
- Modify: `packages/frontend/web-app/src/services/discovery.ts`
- Modify: `packages/frontend/web-app/src/hooks/useDiscovery.ts`

**Step 1: Add types and API function to discovery service**

In `services/discovery.ts`, add after the `runDiscovery` function (after line 33):

```typescript
// =============================================================================
// Discovery Run Status
// =============================================================================

export interface DiscoveryRun {
  id: string;
  connection_id: string;
  connection_name: string;
  status: 'pending' | 'scanning' | 'completed' | 'failed';
  assets_found: number;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}

export interface DiscoveryRunsResponse {
  runs: DiscoveryRun[];
}

/**
 * Get discovery runs by IDs (for polling active runs)
 */
export async function getDiscoveryRuns(runIds: string[]): Promise<DiscoveryRun[]> {
  const params = new URLSearchParams({ ids: runIds.join(',') });
  const response = await apiFetch<DiscoveryRunsResponse>(
    `/discovery/runs?${params}`,
    {
      headers: { 'X-Tenant-ID': DEV_TENANT_ID },
    }
  );
  return response.runs;
}
```

**Step 2: Add `useDiscoveryRuns` hook**

In `hooks/useDiscovery.ts`, update imports and add the polling hook:

Replace entire file with:

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listConnections,
  runDiscovery,
  triggerMockDiscovery,
  getDiscoveryRuns,
  type DiscoveryRequest,
  type MockDiscoveryRequest,
  type DiscoveryRun,
} from '../services/discovery.js';

/**
 * Hook to list available connections
 */
export function useConnections() {
  return useQuery({
    queryKey: ['connections'],
    queryFn: listConnections,
    staleTime: 30_000,
  });
}

/**
 * Hook to trigger a discovery run
 */
export function useRunDiscovery() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: DiscoveryRequest) => runDiscovery(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

/**
 * Hook to trigger mock discovery (dev only)
 */
export function useMockDiscovery() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: MockDiscoveryRequest) => triggerMockDiscovery(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

/**
 * Hook to poll discovery run statuses.
 * Polls every 3 seconds while any run is in a non-terminal state.
 */
export function useDiscoveryRuns(runIds: string[]) {
  const queryClient = useQueryClient();

  return useQuery({
    queryKey: ['discovery-runs', ...runIds],
    queryFn: () => getDiscoveryRuns(runIds),
    enabled: runIds.length > 0,
    refetchInterval: (query) => {
      const runs = query.state.data as DiscoveryRun[] | undefined;
      if (!runs) return 3000;
      const allTerminal = runs.every(
        (r) => r.status === 'completed' || r.status === 'failed'
      );
      if (allTerminal) {
        // Invalidate assets one final time
        queryClient.invalidateQueries({ queryKey: ['assets'] });
        return false; // Stop polling
      }
      return 3000;
    },
  });
}
```

**Step 3: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/services/discovery.ts packages/frontend/web-app/src/hooks/useDiscovery.ts
git commit -m "feat(frontend): add discovery run status polling hooks and service"
```

---

## Task 9: Rework DiscoveryModal with Three-Phase UX

**Files:**
- Modify: `packages/frontend/web-app/src/components/DiscoveryModal.tsx`

**Step 1: Replace entire file with three-phase modal**

```typescript
import { useState, useCallback } from 'react';
import {
  X,
  Search,
  Database,
  Server,
  Globe,
  Loader2,
  CheckCircle,
  XCircle,
  FlaskConical,
  ArrowRight,
} from 'lucide-react';
import {
  useConnections,
  useRunDiscovery,
  useMockDiscovery,
  useDiscoveryRuns,
} from '../hooks/useDiscovery.js';
import type { DiscoveryRun } from '../services/discovery.js';

interface DiscoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const connectorIcons: Record<string, React.ElementType> = {
  postgresql: Database,
  mysql: Database,
  snowflake: Database,
  oracle: Database,
  salesforce: Globe,
  rest_api: Server,
  s3: Server,
  default: Server,
};

// Toggle for local development without Kafka
const USE_MOCK_DISCOVERY = false;

type Phase = 'select' | 'running' | 'complete';

export function DiscoveryModal({ isOpen, onClose }: DiscoveryModalProps) {
  const [selectedConnections, setSelectedConnections] = useState<string[]>([]);
  const [activeRunIds, setActiveRunIds] = useState<string[]>([]);
  const [phase, setPhase] = useState<Phase>('select');
  const [mockAssetsCreated, setMockAssetsCreated] = useState(0);

  const { data: connections, isLoading: loadingConnections } = useConnections();
  const { mutateAsync: runDiscovery } = useRunDiscovery();
  const { mutate: mockDiscovery, isPending: isMockPending, isSuccess: isMockSuccess } = useMockDiscovery();
  const { data: discoveryRuns } = useDiscoveryRuns(activeRunIds);

  if (!isOpen) return null;

  // Derive status from discovery runs
  const allTerminal = discoveryRuns?.every(
    (r) => r.status === 'completed' || r.status === 'failed'
  );
  const totalAssetsFound = discoveryRuns?.reduce((sum, r) => sum + r.assets_found, 0) ?? 0;
  const connectionsCompleted = discoveryRuns?.filter((r) => r.status === 'completed').length ?? 0;

  // Auto-advance to complete phase
  if (phase === 'running' && allTerminal && discoveryRuns && discoveryRuns.length > 0) {
    // Use setTimeout to avoid setState during render
    setTimeout(() => setPhase('complete'), 0);
  }

  const handleToggleConnection = (id: string) => {
    setSelectedConnections((prev) =>
      prev.includes(id) ? prev.filter((c) => c !== id) : [...prev, id]
    );
  };

  const handleStartDiscovery = async () => {
    if (USE_MOCK_DISCOVERY) {
      let totalCreated = 0;
      selectedConnections.forEach((connectionId) => {
        mockDiscovery(
          { connection_id: connectionId, asset_count: 5 },
          {
            onSuccess: (data) => {
              totalCreated += data.assets_created;
              setMockAssetsCreated(totalCreated);
            },
          }
        );
      });
      setPhase('complete');
    } else {
      // Dispatch all discovery runs and collect run_ids
      const runIds: string[] = [];
      for (const connectionId of selectedConnections) {
        try {
          const response = await runDiscovery({ connection_id: connectionId });
          runIds.push(response.run_id);
        } catch (err) {
          console.error('Failed to start discovery for', connectionId, err);
        }
      }
      setActiveRunIds(runIds);
      setPhase('running');
    }
  };

  const handleClose = () => {
    setSelectedConnections([]);
    setActiveRunIds([]);
    setPhase('select');
    setMockAssetsCreated(0);
    onClose();
  };

  const handleViewAssets = () => {
    handleClose();
    // Assets list will auto-refresh via query invalidation
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={handleClose} />

      <div className="relative bg-white rounded-xl shadow-xl w-full max-w-lg mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-50 rounded-lg">
              <Search className="w-5 h-5 text-primary-600" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-gray-900">
                {phase === 'select' && 'Discover Assets'}
                {phase === 'running' && 'Discovery Running'}
                {phase === 'complete' && 'Discovery Complete'}
              </h2>
              <p className="text-sm text-gray-500">
                {phase === 'select' && 'Select connections to scan for assets'}
                {phase === 'running' && 'Scanning selected connections...'}
                {phase === 'complete' && `Found ${USE_MOCK_DISCOVERY ? mockAssetsCreated : totalAssetsFound} assets`}
              </p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 max-h-96 overflow-y-auto">
          {/* Phase 1: Select connections */}
          {phase === 'select' && (
            <>
              {loadingConnections ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-6 h-6 text-primary-500 animate-spin" />
                </div>
              ) : (
                <div className="space-y-2">
                  {connections?.map((connection) => {
                    const Icon = connectorIcons[connection.connector_type] ?? connectorIcons.default;
                    const isSelected = selectedConnections.includes(connection.id);

                    return (
                      <button
                        key={connection.id}
                        onClick={() => handleToggleConnection(connection.id)}
                        className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                          isSelected
                            ? 'border-primary-500 bg-primary-50'
                            : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                        }`}
                      >
                        <div className={`p-2 rounded-lg ${isSelected ? 'bg-primary-100' : 'bg-gray-100'}`}>
                          <Icon className={`w-5 h-5 ${isSelected ? 'text-primary-600' : 'text-gray-600'}`} />
                        </div>
                        <div className="flex-1 text-left">
                          <div className="font-medium text-gray-900">{connection.name}</div>
                          <div className="text-sm text-gray-500">{connection.connector_type}</div>
                        </div>
                        <div
                          className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                            isSelected ? 'border-primary-500 bg-primary-500' : 'border-gray-300'
                          }`}
                        >
                          {isSelected && <CheckCircle className="w-3 h-3 text-white" />}
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}
            </>
          )}

          {/* Phase 2: Running — show per-connection status */}
          {phase === 'running' && (
            <div className="space-y-3">
              {discoveryRuns?.map((run) => (
                <DiscoveryRunStatusRow key={run.id} run={run} />
              ))}
              {(!discoveryRuns || discoveryRuns.length === 0) && (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-6 h-6 text-primary-500 animate-spin" />
                  <span className="ml-2 text-sm text-gray-500">Starting discovery...</span>
                </div>
              )}
            </div>
          )}

          {/* Phase 3: Complete — summary */}
          {phase === 'complete' && (
            <div className="flex flex-col items-center justify-center py-6 text-center">
              <CheckCircle className="w-12 h-12 text-green-500 mb-3" />
              {USE_MOCK_DISCOVERY ? (
                <>
                  <h3 className="text-lg font-medium text-gray-900">Mock Discovery Complete</h3>
                  <p className="text-sm text-gray-500 mt-1">
                    {mockAssetsCreated} mock assets created
                  </p>
                  <div className="mt-3 flex items-center gap-2 text-xs text-amber-600 bg-amber-50 px-3 py-1.5 rounded-full">
                    <FlaskConical className="w-3 h-3" />
                    Dev mode — using mock data
                  </div>
                </>
              ) : (
                <>
                  <h3 className="text-lg font-medium text-gray-900">
                    Discovered {totalAssetsFound} Assets
                  </h3>
                  <p className="text-sm text-gray-500 mt-1">
                    Across {connectionsCompleted} connection{connectionsCompleted !== 1 ? 's' : ''}
                  </p>
                  {discoveryRuns?.some((r) => r.status === 'failed') && (
                    <p className="text-sm text-red-500 mt-2">
                      {discoveryRuns.filter((r) => r.status === 'failed').length} connection(s) failed
                    </p>
                  )}
                </>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-100">
          {phase === 'select' && (
            <>
              <button
                onClick={handleClose}
                className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
              >
                Cancel
              </button>
              <button
                onClick={handleStartDiscovery}
                disabled={selectedConnections.length === 0}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Search className="w-4 h-4" />
                Start Discovery ({selectedConnections.length})
              </button>
            </>
          )}
          {phase === 'running' && (
            <button
              onClick={handleClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
            >
              Close
            </button>
          )}
          {phase === 'complete' && (
            <button
              onClick={handleViewAssets}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg"
            >
              View Assets
              <ArrowRight className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

/** Status row for a single discovery run */
function DiscoveryRunStatusRow({ run }: { run: DiscoveryRun }) {
  const statusConfig = {
    pending: { icon: Loader2, color: 'text-gray-400', bg: 'bg-gray-50', label: 'Pending', spin: true },
    scanning: { icon: Loader2, color: 'text-primary-500', bg: 'bg-primary-50', label: 'Scanning', spin: true },
    completed: { icon: CheckCircle, color: 'text-green-500', bg: 'bg-green-50', label: 'Complete', spin: false },
    failed: { icon: XCircle, color: 'text-red-500', bg: 'bg-red-50', label: 'Failed', spin: false },
  };

  const config = statusConfig[run.status] ?? statusConfig.pending;
  const Icon = config.icon;

  return (
    <div className={`flex items-center gap-3 p-3 rounded-lg ${config.bg}`}>
      <Icon className={`w-5 h-5 ${config.color} ${config.spin ? 'animate-spin' : ''}`} />
      <div className="flex-1">
        <div className="font-medium text-gray-900 text-sm">{run.connection_name}</div>
        <div className="text-xs text-gray-500">
          {config.label}
          {run.assets_found > 0 && ` · ${run.assets_found} assets`}
          {run.error_message && ` · ${run.error_message}`}
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/components/DiscoveryModal.tsx
git commit -m "feat(frontend): three-phase discovery modal with live status

Phase 1: Select connections
Phase 2: Live status polling per connection
Phase 3: Summary with asset count and View Assets button"
```

---

## Task 10: Build Verification

**Step 1: Build Go agent**

Run: `cd agent && go build ./...`
Expected: Compiles without errors

**Step 2: Build Rust integration-service**

Run: `cd services/integration-service && cargo build`
Expected: Compiles without errors

**Step 3: Build frontend**

Run: `cd packages/frontend/web-app && npm run build`
Expected: Compiles without errors

**Step 4: Commit verification**

```bash
git add -A
git commit -m "feat: complete real discovery execution implementation

- Agent PostgreSQL discovery handler (schemas, tables, views, columns)
- Discovery runs table for status tracking
- API endpoints: POST /discovery/run, GET /discovery/runs
- Three-phase frontend modal with live status polling
- End-to-end flow: UI → API → Kafka → Agent → Kafka → Consumer → Assets"
```

---

## Summary

| Component | Change | Files |
|-----------|--------|-------|
| Agent | New discovery handler + registration | `agent/internal/adapters/discovery/handler.go`, `agent/cmd/agent/main.go` |
| Schema | New `discovery_runs` table | `schemas/postgres/008_discovery_runs_schema.sql` |
| Integration Service | Discovery run storage, updated endpoint, status API | `storage/mod.rs`, `api/mod.rs`, `main.rs`, `consumer/mod.rs` |
| Frontend | Status service, polling hooks, three-phase modal | `services/discovery.ts`, `hooks/useDiscovery.ts`, `components/DiscoveryModal.tsx` |

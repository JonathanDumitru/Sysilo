# Automation Playbooks Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a visual workflow automation system for operational runbooks, migration playbooks, and integration orchestration.

**Architecture:** React Flow visual editor (frontend) + Rust/Axum API (integration-service) + PostgreSQL storage + Kafka task dispatch for step execution.

**Tech Stack:** React Flow (@xyflow/react v12), TypeScript, Rust/Axum, SQLx, PostgreSQL (JSONB), Kafka

**Note:** The existing `/rationalization/playbooks` route is for migration playbooks. Our new automation playbooks will be under `/operations/playbooks/*` in the Operations Center.

---

## Task 1: Database Schema for Playbooks

**Files:**
- Create: `services/integration-service/migrations/20260204_create_playbooks.sql`

**Step 1: Write the migration SQL**

```sql
-- Automation Playbooks schema
CREATE TABLE playbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    steps JSONB NOT NULL DEFAULT '[]',
    variables JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_playbooks_tenant ON playbooks(tenant_id);

CREATE TABLE playbook_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    playbook_id UUID NOT NULL REFERENCES playbooks(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    variables JSONB NOT NULL DEFAULT '{}',
    step_states JSONB NOT NULL DEFAULT '[]',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_playbook_runs_playbook ON playbook_runs(playbook_id);
CREATE INDEX idx_playbook_runs_tenant ON playbook_runs(tenant_id);
CREATE INDEX idx_playbook_runs_status ON playbook_runs(status);
```

**Step 2: Apply the migration**

Run: `cd services/integration-service && sqlx migrate add create_playbooks`

Then copy the SQL content into the generated file.

**Step 3: Commit**

```bash
git add services/integration-service/migrations/
git commit -m "feat(playbooks): add database schema for automation playbooks"
```

---

## Task 2: Rust Domain Types for Playbooks

**Files:**
- Create: `services/integration-service/src/playbooks/mod.rs`
- Modify: `services/integration-service/src/main.rs`

**Step 1: Write the failing test**

```rust
// In src/playbooks/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_type_serialization() {
        let step = Step {
            id: "step-1".to_string(),
            step_type: StepType::Integration,
            name: "Test Step".to_string(),
            config: serde_json::json!({"integration_id": "uuid"}),
            on_success: vec!["step-2".to_string()],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("integration"));
    }

    #[test]
    fn test_run_status_default() {
        let status = RunStatus::default();
        assert!(matches!(status, RunStatus::Pending));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd services/integration-service && cargo test playbooks`
Expected: FAIL with "unresolved import"

**Step 3: Write minimal implementation**

```rust
// src/playbooks/mod.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trigger types for playbooks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    #[default]
    Manual,
    Scheduled,
    Webhook,
    Event,
}

/// Step types within a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Integration,
    Webhook,
    Wait,
    Condition,
    Approval,
}

/// Variable definition for playbook inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: String,
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

/// A single step in the playbook workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub step_type: StepType,
    pub name: String,
    pub config: serde_json::Value,
    #[serde(default)]
    pub on_success: Vec<String>,
    #[serde(default)]
    pub on_failure: Vec<String>,
}

/// Status of a playbook run
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

/// Status of a single step within a run
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// State of a step during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub step_id: String,
    pub status: StepStatus,
    #[serde(default)]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub output: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Full playbook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub steps: Vec<Step>,
    pub variables: Vec<Variable>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A running instance of a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRun {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub tenant_id: Uuid,
    pub status: RunStatus,
    pub variables: serde_json::Value,
    pub step_states: Vec<StepState>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

**Step 4: Register module in main.rs**

Add to `src/main.rs`:
```rust
mod playbooks;
```

**Step 5: Run test to verify it passes**

Run: `cd services/integration-service && cargo test playbooks`
Expected: PASS

**Step 6: Commit**

```bash
git add services/integration-service/src/playbooks/ services/integration-service/src/main.rs
git commit -m "feat(playbooks): add Rust domain types for playbooks and runs"
```

---

## Task 3: Playbook Storage Layer

**Files:**
- Modify: `services/integration-service/src/storage/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod playbook_tests {
    use super::*;

    #[tokio::test]
    async fn test_playbook_row_from_row() {
        // This will fail until we add PlaybookRow struct
        let _: PlaybookRow;
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd services/integration-service && cargo test playbook_row`
Expected: FAIL with "cannot find type `PlaybookRow`"

**Step 3: Write the storage implementation**

Add to `src/storage/mod.rs`:

```rust
use crate::playbooks::{RunStatus, StepState, TriggerType};

/// Database row for playbooks
#[derive(Debug, FromRow)]
pub struct PlaybookRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub steps: serde_json::Value,
    pub variables: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database row for playbook runs
#[derive(Debug, FromRow)]
pub struct PlaybookRunRow {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub tenant_id: Uuid,
    pub status: String,
    pub variables: serde_json::Value,
    pub step_states: serde_json::Value,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Storage {
    // Playbook CRUD operations

    /// List playbooks for a tenant
    pub async fn list_playbooks(
        &self,
        tenant_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PlaybookRow>, StorageError> {
        let rows: Vec<PlaybookRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, trigger_type,
                   steps, variables, created_at, updated_at
            FROM playbooks
            WHERE tenant_id = $1::uuid
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get a single playbook
    pub async fn get_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
    ) -> Result<PlaybookRow, StorageError> {
        let row: Option<PlaybookRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, trigger_type,
                   steps, variables, created_at, updated_at
            FROM playbooks
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("Playbook {}", playbook_id)))
    }

    /// Create a new playbook
    pub async fn create_playbook(
        &self,
        tenant_id: &str,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        steps: serde_json::Value,
        variables: serde_json::Value,
    ) -> Result<PlaybookRow, StorageError> {
        let row: PlaybookRow = sqlx::query_as(
            r#"
            INSERT INTO playbooks (tenant_id, name, description, trigger_type, steps, variables)
            VALUES ($1::uuid, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, description, trigger_type,
                      steps, variables, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(steps)
        .bind(variables)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Update a playbook
    pub async fn update_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        steps: serde_json::Value,
        variables: serde_json::Value,
    ) -> Result<PlaybookRow, StorageError> {
        let row: PlaybookRow = sqlx::query_as(
            r#"
            UPDATE playbooks
            SET name = $3, description = $4, trigger_type = $5,
                steps = $6, variables = $7, updated_at = NOW()
            WHERE tenant_id = $1::uuid AND id = $2
            RETURNING id, tenant_id, name, description, trigger_type,
                      steps, variables, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(steps)
        .bind(variables)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Delete a playbook
    pub async fn delete_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
    ) -> Result<(), StorageError> {
        let result = sqlx::query(
            r#"
            DELETE FROM playbooks
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Playbook {}", playbook_id)));
        }

        Ok(())
    }

    // Playbook Run operations

    /// Create a new playbook run
    pub async fn create_playbook_run(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        variables: serde_json::Value,
        step_states: serde_json::Value,
    ) -> Result<PlaybookRunRow, StorageError> {
        let row: PlaybookRunRow = sqlx::query_as(
            r#"
            INSERT INTO playbook_runs (tenant_id, playbook_id, variables, step_states, status)
            VALUES ($1::uuid, $2, $3, $4, 'pending')
            RETURNING id, playbook_id, tenant_id, status, variables,
                      step_states, started_at, completed_at
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(variables)
        .bind(step_states)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get a playbook run by ID
    pub async fn get_playbook_run(
        &self,
        tenant_id: &str,
        run_id: Uuid,
    ) -> Result<PlaybookRunRow, StorageError> {
        let row: Option<PlaybookRunRow> = sqlx::query_as(
            r#"
            SELECT id, playbook_id, tenant_id, status, variables,
                   step_states, started_at, completed_at
            FROM playbook_runs
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("PlaybookRun {}", run_id)))
    }

    /// List runs for a playbook
    pub async fn list_playbook_runs(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PlaybookRunRow>, StorageError> {
        let rows: Vec<PlaybookRunRow> = sqlx::query_as(
            r#"
            SELECT id, playbook_id, tenant_id, status, variables,
                   step_states, started_at, completed_at
            FROM playbook_runs
            WHERE tenant_id = $1::uuid AND playbook_id = $2
            ORDER BY started_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Update playbook run status and step states
    pub async fn update_playbook_run(
        &self,
        run_id: Uuid,
        status: &str,
        step_states: serde_json::Value,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE playbook_runs
            SET status = $2, step_states = $3,
                completed_at = CASE
                    WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW()
                    ELSE completed_at
                END
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status)
        .bind(step_states)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd services/integration-service && cargo test playbook_row`
Expected: PASS

**Step 5: Commit**

```bash
git add services/integration-service/src/storage/mod.rs
git commit -m "feat(playbooks): add storage layer for playbooks and runs"
```

---

## Task 4: Playbook API Endpoints

**Files:**
- Create: `services/integration-service/src/playbooks/api.rs`
- Modify: `services/integration-service/src/playbooks/mod.rs`
- Modify: `services/integration-service/src/main.rs`

**Step 1: Write the API handlers**

Create `src/playbooks/api.rs`:

```rust
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
use crate::middleware::TenantContext;
use crate::AppState;

use super::{Step, TriggerType, Variable};

// Request/Response types

#[derive(Debug, Serialize)]
pub struct PlaybookListResponse {
    pub playbooks: Vec<PlaybookSummary>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct PlaybookSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub step_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaybookRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub trigger_type: TriggerType,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub variables: Vec<Variable>,
}

#[derive(Debug, Serialize)]
pub struct PlaybookResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub steps: Vec<Step>,
    pub variables: Vec<Variable>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RunPlaybookRequest {
    #[serde(default)]
    pub variables: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct PlaybookRunResponse {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub status: String,
    pub variables: serde_json::Value,
    pub step_states: serde_json::Value,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlaybookRunListResponse {
    pub runs: Vec<PlaybookRunResponse>,
}

// Handlers

pub async fn list_playbooks(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<PlaybookListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = state
        .storage
        .list_playbooks(&tenant_id, 100, 0)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let playbooks: Vec<PlaybookSummary> = rows
        .into_iter()
        .map(|r| {
            let steps: Vec<Step> = serde_json::from_value(r.steps).unwrap_or_default();
            PlaybookSummary {
                id: r.id,
                name: r.name,
                description: r.description,
                trigger_type: r.trigger_type,
                step_count: steps.len(),
                created_at: r.created_at.to_rfc3339(),
                updated_at: r.updated_at.to_rfc3339(),
            }
        })
        .collect();

    let total = playbooks.len() as i64;

    Ok(Json(PlaybookListResponse { playbooks, total }))
}

pub async fn create_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<CreatePlaybookRequest>,
) -> Result<(StatusCode, Json<PlaybookResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let trigger_str = serde_json::to_string(&req.trigger_type)
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|_| "manual".to_string());

    let steps_json = serde_json::to_value(&req.steps).map_err(|e| ApiError {
        error: "invalid_steps".to_string(),
        message: e.to_string(),
    })?;

    let variables_json = serde_json::to_value(&req.variables).map_err(|e| ApiError {
        error: "invalid_variables".to_string(),
        message: e.to_string(),
    })?;

    let row = state
        .storage
        .create_playbook(
            &tenant_id,
            &req.name,
            req.description.as_deref(),
            &trigger_str,
            steps_json,
            variables_json,
        )
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(PlaybookResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            trigger_type: row.trigger_type,
            steps: serde_json::from_value(row.steps).unwrap_or_default(),
            variables: serde_json::from_value(row.variables).unwrap_or_default(),
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }),
    ))
}

pub async fn get_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaybookResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_playbook(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(PlaybookResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        trigger_type: row.trigger_type,
        steps: serde_json::from_value(row.steps).unwrap_or_default(),
        variables: serde_json::from_value(row.variables).unwrap_or_default(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

pub async fn update_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreatePlaybookRequest>,
) -> Result<Json<PlaybookResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let trigger_str = serde_json::to_string(&req.trigger_type)
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|_| "manual".to_string());

    let steps_json = serde_json::to_value(&req.steps).map_err(|e| ApiError {
        error: "invalid_steps".to_string(),
        message: e.to_string(),
    })?;

    let variables_json = serde_json::to_value(&req.variables).map_err(|e| ApiError {
        error: "invalid_variables".to_string(),
        message: e.to_string(),
    })?;

    let row = state
        .storage
        .update_playbook(
            &tenant_id,
            id,
            &req.name,
            req.description.as_deref(),
            &trigger_str,
            steps_json,
            variables_json,
        )
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(PlaybookResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        trigger_type: row.trigger_type,
        steps: serde_json::from_value(row.steps).unwrap_or_default(),
        variables: serde_json::from_value(row.variables).unwrap_or_default(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

pub async fn delete_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    state
        .storage
        .delete_playbook(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn run_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<RunPlaybookRequest>,
) -> Result<(StatusCode, Json<PlaybookRunResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the playbook
    let playbook = state
        .storage
        .get_playbook(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Parse steps to create initial step states
    let steps: Vec<Step> = serde_json::from_value(playbook.steps).unwrap_or_default();
    let initial_states: Vec<super::StepState> = steps
        .iter()
        .map(|s| super::StepState {
            step_id: s.id.clone(),
            status: super::StepStatus::Pending,
            started_at: None,
            completed_at: None,
            output: None,
            error: None,
        })
        .collect();

    let step_states_json = serde_json::to_value(&initial_states).unwrap_or_default();

    // Create the run
    let row = state
        .storage
        .create_playbook_run(&tenant_id, id, req.variables.clone(), step_states_json)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // TODO: Dispatch first step(s) to Kafka

    Ok((
        StatusCode::ACCEPTED,
        Json(PlaybookRunResponse {
            id: row.id,
            playbook_id: row.playbook_id,
            status: row.status,
            variables: row.variables,
            step_states: row.step_states,
            started_at: row.started_at.to_rfc3339(),
            completed_at: row.completed_at.map(|t| t.to_rfc3339()),
        }),
    ))
}

pub async fn list_playbook_runs(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaybookRunListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = state
        .storage
        .list_playbook_runs(&tenant_id, id, 50)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let runs: Vec<PlaybookRunResponse> = rows
        .into_iter()
        .map(|r| PlaybookRunResponse {
            id: r.id,
            playbook_id: r.playbook_id,
            status: r.status,
            variables: r.variables,
            step_states: r.step_states,
            started_at: r.started_at.to_rfc3339(),
            completed_at: r.completed_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(PlaybookRunListResponse { runs }))
}

pub async fn get_playbook_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_playbook_run(&tenant_id, run_id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(PlaybookRunResponse {
        id: row.id,
        playbook_id: row.playbook_id,
        status: row.status,
        variables: row.variables,
        step_states: row.step_states,
        started_at: row.started_at.to_rfc3339(),
        completed_at: row.completed_at.map(|t| t.to_rfc3339()),
    }))
}

pub async fn approve_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_playbook_run(&tenant_id, run_id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: "Run is not waiting for approval".to_string(),
        });
    }

    // TODO: Resume execution by dispatching next steps

    state
        .storage
        .update_playbook_run(run_id, "running", row.step_states.clone())
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(PlaybookRunResponse {
        id: row.id,
        playbook_id: row.playbook_id,
        status: "running".to_string(),
        variables: row.variables,
        step_states: row.step_states,
        started_at: row.started_at.to_rfc3339(),
        completed_at: None,
    }))
}

pub async fn reject_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_playbook_run(&tenant_id, run_id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: "Run is not waiting for approval".to_string(),
        });
    }

    state
        .storage
        .update_playbook_run(run_id, "failed", row.step_states.clone())
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(PlaybookRunResponse {
        id: row.id,
        playbook_id: row.playbook_id,
        status: "failed".to_string(),
        variables: row.variables,
        step_states: row.step_states,
        started_at: row.started_at.to_rfc3339(),
        completed_at: Some(chrono::Utc::now().to_rfc3339()),
    }))
}
```

**Step 2: Update playbooks/mod.rs**

Add to `src/playbooks/mod.rs`:
```rust
pub mod api;
```

**Step 3: Register routes in main.rs**

Add the playbook routes in `src/main.rs`:

```rust
use playbooks::api as playbooks_api;

// In the router setup, add:
    .route("/playbooks", axum::routing::get(playbooks_api::list_playbooks).post(playbooks_api::create_playbook))
    .route("/playbooks/:id", axum::routing::get(playbooks_api::get_playbook).put(playbooks_api::update_playbook).delete(playbooks_api::delete_playbook))
    .route("/playbooks/:id/run", axum::routing::post(playbooks_api::run_playbook))
    .route("/playbooks/:id/runs", axum::routing::get(playbooks_api::list_playbook_runs))
    .route("/runs/:id", axum::routing::get(playbooks_api::get_playbook_run))
    .route("/runs/:id/approve", axum::routing::post(playbooks_api::approve_run))
    .route("/runs/:id/reject", axum::routing::post(playbooks_api::reject_run))
```

**Step 4: Build to verify compilation**

Run: `cd services/integration-service && cargo build`
Expected: SUCCESS

**Step 5: Commit**

```bash
git add services/integration-service/src/playbooks/ services/integration-service/src/main.rs
git commit -m "feat(playbooks): add REST API endpoints for playbooks CRUD and execution"
```

---

## Task 5: Frontend API Service for Playbooks

**Files:**
- Create: `packages/frontend/web-app/src/services/playbooks.ts`

**Step 1: Create the playbooks service**

```typescript
import { apiFetch } from './api';

// Types matching backend
export type TriggerType = 'manual' | 'scheduled' | 'webhook' | 'event';
export type StepType = 'integration' | 'webhook' | 'wait' | 'condition' | 'approval';
export type RunStatus = 'pending' | 'running' | 'waiting_approval' | 'completed' | 'failed' | 'cancelled';
export type StepStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped';

export interface Variable {
  name: string;
  var_type: string;
  required: boolean;
  default_value?: string;
}

export interface Step {
  id: string;
  step_type: StepType;
  name: string;
  config: Record<string, unknown>;
  on_success: string[];
  on_failure: string[];
}

export interface StepState {
  step_id: string;
  status: StepStatus;
  started_at?: string;
  completed_at?: string;
  output?: Record<string, unknown>;
  error?: string;
}

export interface Playbook {
  id: string;
  name: string;
  description?: string;
  trigger_type: TriggerType;
  steps: Step[];
  variables: Variable[];
  created_at: string;
  updated_at: string;
}

export interface PlaybookSummary {
  id: string;
  name: string;
  description?: string;
  trigger_type: TriggerType;
  step_count: number;
  created_at: string;
  updated_at: string;
}

export interface PlaybookRun {
  id: string;
  playbook_id: string;
  status: RunStatus;
  variables: Record<string, unknown>;
  step_states: StepState[];
  started_at: string;
  completed_at?: string;
}

export interface ListPlaybooksResponse {
  playbooks: PlaybookSummary[];
  total: number;
}

export interface ListRunsResponse {
  runs: PlaybookRun[];
}

// API functions

export async function listPlaybooks(): Promise<ListPlaybooksResponse> {
  return apiFetch<ListPlaybooksResponse>('/integrations/playbooks');
}

export async function getPlaybook(id: string): Promise<Playbook> {
  return apiFetch<Playbook>(`/integrations/playbooks/${id}`);
}

export interface CreatePlaybookRequest {
  name: string;
  description?: string;
  trigger_type?: TriggerType;
  steps?: Step[];
  variables?: Variable[];
}

export async function createPlaybook(request: CreatePlaybookRequest): Promise<Playbook> {
  return apiFetch<Playbook>('/integrations/playbooks', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export async function updatePlaybook(id: string, request: CreatePlaybookRequest): Promise<Playbook> {
  return apiFetch<Playbook>(`/integrations/playbooks/${id}`, {
    method: 'PUT',
    body: JSON.stringify(request),
  });
}

export async function deletePlaybook(id: string): Promise<void> {
  await apiFetch(`/integrations/playbooks/${id}`, {
    method: 'DELETE',
  });
}

export interface RunPlaybookRequest {
  variables?: Record<string, unknown>;
}

export async function runPlaybook(id: string, request: RunPlaybookRequest = {}): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbooks/${id}/run`, {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export async function listPlaybookRuns(playbookId: string): Promise<ListRunsResponse> {
  return apiFetch<ListRunsResponse>(`/integrations/playbooks/${playbookId}/runs`);
}

export async function getPlaybookRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/runs/${runId}`);
}

export async function approveRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/runs/${runId}/approve`, {
    method: 'POST',
  });
}

export async function rejectRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/runs/${runId}/reject`, {
    method: 'POST',
  });
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/services/playbooks.ts
git commit -m "feat(playbooks): add frontend API service for playbooks"
```

---

## Task 6: React Query Hooks for Playbooks

**Files:**
- Create: `packages/frontend/web-app/src/hooks/usePlaybooks.ts`

**Step 1: Create the hooks**

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listPlaybooks,
  getPlaybook,
  createPlaybook,
  updatePlaybook,
  deletePlaybook,
  runPlaybook,
  listPlaybookRuns,
  getPlaybookRun,
  approveRun,
  rejectRun,
  type CreatePlaybookRequest,
  type RunPlaybookRequest,
  type Playbook,
  type PlaybookRun,
} from '../services/playbooks';

// Query keys
export const playbookKeys = {
  all: ['playbooks'] as const,
  list: () => [...playbookKeys.all, 'list'] as const,
  detail: (id: string) => [...playbookKeys.all, 'detail', id] as const,
  runs: (playbookId: string) => [...playbookKeys.all, 'runs', playbookId] as const,
  run: (runId: string) => [...playbookKeys.all, 'run', runId] as const,
};

// List playbooks
export function usePlaybooks() {
  return useQuery({
    queryKey: playbookKeys.list(),
    queryFn: listPlaybooks,
  });
}

// Get single playbook
export function usePlaybook(id: string) {
  return useQuery({
    queryKey: playbookKeys.detail(id),
    queryFn: () => getPlaybook(id),
    enabled: !!id,
  });
}

// Create playbook
export function useCreatePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreatePlaybookRequest) => createPlaybook(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
    },
  });
}

// Update playbook
export function useUpdatePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, request }: { id: string; request: CreatePlaybookRequest }) =>
      updatePlaybook(id, request),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
      queryClient.invalidateQueries({ queryKey: playbookKeys.detail(data.id) });
    },
  });
}

// Delete playbook
export function useDeletePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deletePlaybook(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
    },
  });
}

// Run playbook
export function useRunPlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, request }: { id: string; request?: RunPlaybookRequest }) =>
      runPlaybook(id, request),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(variables.id) });
    },
  });
}

// List runs for a playbook
export function usePlaybookRuns(playbookId: string) {
  return useQuery({
    queryKey: playbookKeys.runs(playbookId),
    queryFn: () => listPlaybookRuns(playbookId),
    enabled: !!playbookId,
  });
}

// Get single run with polling for active runs
export function usePlaybookRun(runId: string, refetchInterval?: number) {
  return useQuery({
    queryKey: playbookKeys.run(runId),
    queryFn: () => getPlaybookRun(runId),
    enabled: !!runId,
    refetchInterval: (query) => {
      const data = query.state.data;
      // Only poll if run is active
      if (data && ['pending', 'running', 'waiting_approval'].includes(data.status)) {
        return refetchInterval ?? 2000;
      }
      return false;
    },
  });
}

// Approve run
export function useApproveRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (runId: string) => approveRun(runId),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.run(data.id) });
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(data.playbook_id) });
    },
  });
}

// Reject run
export function useRejectRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (runId: string) => rejectRun(runId),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.run(data.id) });
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(data.playbook_id) });
    },
  });
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/hooks/usePlaybooks.ts
git commit -m "feat(playbooks): add React Query hooks for playbooks"
```

---

## Task 7: Playbook Step Node Components

**Files:**
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/index.ts`
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/IntegrationStepNode.tsx`
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/WebhookStepNode.tsx`
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/WaitStepNode.tsx`
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/ConditionStepNode.tsx`
- Create: `packages/frontend/web-app/src/components/playbooks/nodes/ApprovalStepNode.tsx`

**Step 1: Create base node styles and common interface**

Create `src/components/playbooks/nodes/index.ts`:

```typescript
export * from './IntegrationStepNode';
export * from './WebhookStepNode';
export * from './WaitStepNode';
export * from './ConditionStepNode';
export * from './ApprovalStepNode';

import type { StepStatus } from '../../../services/playbooks';

export interface StepNodeData extends Record<string, unknown> {
  id: string;
  name: string;
  config: Record<string, unknown>;
  status?: StepStatus;
}

export const statusStyles: Record<string, string> = {
  pending: 'border-gray-300 bg-white',
  running: 'border-blue-400 bg-blue-50 animate-pulse',
  completed: 'border-green-400 bg-green-50',
  failed: 'border-red-400 bg-red-50',
  skipped: 'border-gray-300 bg-gray-100',
};
```

**Step 2: Create IntegrationStepNode**

Create `src/components/playbooks/nodes/IntegrationStepNode.tsx`:

```typescript
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Workflow } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const IntegrationStepNode = memo(function IntegrationStepNode({ data, selected }: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-indigo-100 rounded border border-indigo-200">
          <Workflow className="w-4 h-4 text-indigo-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Integration</div>
        </div>
      </div>

      {nodeData.config?.integration_id && (
        <div className="mt-2 text-xs text-gray-500 truncate">
          ID: {String(nodeData.config.integration_id).slice(0, 8)}...
        </div>
      )}

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});
```

**Step 3: Create WebhookStepNode**

Create `src/components/playbooks/nodes/WebhookStepNode.tsx`:

```typescript
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Globe } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const WebhookStepNode = memo(function WebhookStepNode({ data, selected }: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-orange-100 rounded border border-orange-200">
          <Globe className="w-4 h-4 text-orange-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">
            {String(nodeData.config?.method ?? 'POST')} Webhook
          </div>
        </div>
      </div>

      {nodeData.config?.url && (
        <div className="mt-2 text-xs text-gray-500 truncate">
          {String(nodeData.config.url)}
        </div>
      )}

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});
```

**Step 4: Create WaitStepNode**

Create `src/components/playbooks/nodes/WaitStepNode.tsx`:

```typescript
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Clock } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const WaitStepNode = memo(function WaitStepNode({ data, selected }: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;
  const duration = nodeData.config?.duration_seconds as number | undefined;

  const formatDuration = (seconds?: number) => {
    if (!seconds) return 'Not set';
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h`;
  };

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[140px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-gray-100 rounded border border-gray-200">
          <Clock className="w-4 h-4 text-gray-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Wait {formatDuration(duration)}</div>
        </div>
      </div>

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});
```

**Step 5: Create ConditionStepNode**

Create `src/components/playbooks/nodes/ConditionStepNode.tsx`:

```typescript
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { GitBranch } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const ConditionStepNode = memo(function ConditionStepNode({ data, selected }: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[160px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-purple-100 rounded border border-purple-200">
          <GitBranch className="w-4 h-4 text-purple-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Condition</div>
        </div>
      </div>

      {nodeData.config?.expression && (
        <div className="mt-2 text-xs text-gray-500 font-mono truncate">
          {String(nodeData.config.expression)}
        </div>
      )}

      {/* Two output handles for true/false branches */}
      <Handle
        type="source"
        position={Position.Right}
        id="true"
        className="!bg-green-500"
        style={{ top: '30%' }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="false"
        className="!bg-red-500"
        style={{ top: '70%' }}
      />
    </div>
  );
});
```

**Step 6: Create ApprovalStepNode**

Create `src/components/playbooks/nodes/ApprovalStepNode.tsx`:

```typescript
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { UserCheck } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const ApprovalStepNode = memo(function ApprovalStepNode({ data, selected }: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-yellow-100 rounded border border-yellow-200">
          <UserCheck className="w-4 h-4 text-yellow-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Manual Approval</div>
        </div>
      </div>

      {nodeData.config?.message && (
        <div className="mt-2 text-xs text-gray-500 line-clamp-2">
          {String(nodeData.config.message)}
        </div>
      )}

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});
```

**Step 7: Commit**

```bash
git add packages/frontend/web-app/src/components/playbooks/
git commit -m "feat(playbooks): add React Flow step node components"
```

---

## Task 8: Playbook Editor Toolbox Component

**Files:**
- Create: `packages/frontend/web-app/src/components/playbooks/PlaybookToolbox.tsx`

**Step 1: Create the toolbox component**

```typescript
import { Workflow, Globe, Clock, GitBranch, UserCheck } from 'lucide-react';

const stepTypes = [
  {
    type: 'integration',
    label: 'Integration',
    description: 'Run an existing integration',
    icon: Workflow,
    color: 'indigo',
    defaultData: {
      name: 'Run Integration',
      config: { integration_id: '' },
    },
  },
  {
    type: 'webhook',
    label: 'Webhook',
    description: 'HTTP request to external URL',
    icon: Globe,
    color: 'orange',
    defaultData: {
      name: 'HTTP Request',
      config: { url: '', method: 'POST', headers: {}, body: '' },
    },
  },
  {
    type: 'wait',
    label: 'Wait',
    description: 'Pause for a duration',
    icon: Clock,
    color: 'gray',
    defaultData: {
      name: 'Wait',
      config: { duration_seconds: 60 },
    },
  },
  {
    type: 'condition',
    label: 'Condition',
    description: 'Branch based on expression',
    icon: GitBranch,
    color: 'purple',
    defaultData: {
      name: 'Check Condition',
      config: { expression: '' },
    },
  },
  {
    type: 'approval',
    label: 'Approval',
    description: 'Wait for manual approval',
    icon: UserCheck,
    color: 'yellow',
    defaultData: {
      name: 'Require Approval',
      config: { message: 'Please approve to continue' },
    },
  },
];

const colorMap: Record<string, string> = {
  indigo: 'bg-indigo-50 border-indigo-200 hover:border-indigo-300',
  orange: 'bg-orange-50 border-orange-200 hover:border-orange-300',
  gray: 'bg-gray-50 border-gray-200 hover:border-gray-300',
  purple: 'bg-purple-50 border-purple-200 hover:border-purple-300',
  yellow: 'bg-yellow-50 border-yellow-200 hover:border-yellow-300',
};

const iconColorMap: Record<string, string> = {
  indigo: 'text-indigo-600',
  orange: 'text-orange-600',
  gray: 'text-gray-600',
  purple: 'text-purple-600',
  yellow: 'text-yellow-600',
};

export function PlaybookToolbox() {
  const onDragStart = (
    event: React.DragEvent,
    stepType: string,
    defaultData: Record<string, unknown>
  ) => {
    event.dataTransfer.setData('application/reactflow', stepType);
    event.dataTransfer.setData('application/nodedata', JSON.stringify(defaultData));
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className="w-64 bg-white border-r border-gray-200 p-4 overflow-y-auto">
      <h3 className="text-sm font-semibold text-gray-900 mb-3">Step Types</h3>
      <p className="text-xs text-gray-500 mb-4">Drag steps onto the canvas to build your playbook</p>

      <div className="space-y-2">
        {stepTypes.map((step) => {
          const Icon = step.icon;
          return (
            <div
              key={step.type}
              draggable
              onDragStart={(e) => onDragStart(e, step.type, step.defaultData)}
              className={`p-3 rounded-lg border-2 cursor-grab active:cursor-grabbing transition-colors ${colorMap[step.color]}`}
            >
              <div className="flex items-center gap-2">
                <Icon className={`w-4 h-4 ${iconColorMap[step.color]}`} />
                <span className="text-sm font-medium text-gray-900">{step.label}</span>
              </div>
              <p className="text-xs text-gray-500 mt-1">{step.description}</p>
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/components/playbooks/PlaybookToolbox.tsx
git commit -m "feat(playbooks): add step toolbox component for drag-and-drop"
```

---

## Task 9: Playbook Editor Page

**Files:**
- Create: `packages/frontend/web-app/src/pages/AutomationPlaybookEditorPage.tsx`

**Step 1: Create the editor page**

```typescript
import { useCallback, useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Panel,
  useNodesState,
  useEdgesState,
  addEdge,
  type Connection,
  type Node,
  type Edge,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Save, Play, ArrowLeft, Loader2 } from 'lucide-react';

import {
  IntegrationStepNode,
  WebhookStepNode,
  WaitStepNode,
  ConditionStepNode,
  ApprovalStepNode,
} from '../components/playbooks/nodes';
import { PlaybookToolbox } from '../components/playbooks/PlaybookToolbox';
import { usePlaybook, useCreatePlaybook, useUpdatePlaybook, useRunPlaybook } from '../hooks/usePlaybooks';
import type { Step } from '../services/playbooks';

const nodeTypes: NodeTypes = {
  integration: IntegrationStepNode,
  webhook: WebhookStepNode,
  wait: WaitStepNode,
  condition: ConditionStepNode,
  approval: ApprovalStepNode,
};

export function AutomationPlaybookEditorPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isNew = !id;

  const { data: playbook, isLoading } = usePlaybook(id ?? '');
  const createMutation = useCreatePlaybook();
  const updateMutation = useUpdatePlaybook();
  const runMutation = useRunPlaybook();

  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [playbookName, setPlaybookName] = useState(isNew ? 'Untitled Playbook' : '');
  const [playbookDescription, setPlaybookDescription] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  // Load playbook data
  useEffect(() => {
    if (playbook) {
      setPlaybookName(playbook.name);
      setPlaybookDescription(playbook.description ?? '');

      // Convert steps to nodes
      const loadedNodes: Node[] = playbook.steps.map((step, index) => ({
        id: step.id,
        type: step.step_type,
        position: { x: 100 + index * 250, y: 200 },
        data: {
          id: step.id,
          name: step.name,
          config: step.config,
        },
      }));

      // Create edges from on_success links
      const loadedEdges: Edge[] = [];
      playbook.steps.forEach((step) => {
        step.on_success.forEach((targetId) => {
          loadedEdges.push({
            id: `${step.id}-${targetId}`,
            source: step.id,
            target: targetId,
            type: 'default',
          });
        });
      });

      setNodes(loadedNodes);
      setEdges(loadedEdges);
    }
  }, [playbook, setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const type = event.dataTransfer.getData('application/reactflow');
      const nodeData = JSON.parse(event.dataTransfer.getData('application/nodedata'));

      if (!type) return;

      const newId = `${type}-${Date.now()}`;
      const position = {
        x: event.clientX - 350,
        y: event.clientY - 100,
      };

      const newNode: Node = {
        id: newId,
        type,
        position,
        data: {
          id: newId,
          ...nodeData,
        },
      };

      setNodes((nds) => nds.concat(newNode));
    },
    [setNodes]
  );

  const handleSave = async () => {
    setIsSaving(true);

    // Convert nodes/edges back to steps
    const steps: Step[] = nodes.map((node) => {
      const outgoingEdges = edges.filter((e) => e.source === node.id);
      return {
        id: node.id,
        step_type: node.type as Step['step_type'],
        name: (node.data as Record<string, unknown>).name as string,
        config: (node.data as Record<string, unknown>).config as Record<string, unknown>,
        on_success: outgoingEdges.map((e) => e.target),
        on_failure: [],
      };
    });

    try {
      if (isNew) {
        const created = await createMutation.mutateAsync({
          name: playbookName,
          description: playbookDescription || undefined,
          steps,
        });
        navigate(`/operations/playbooks/${created.id}/edit`, { replace: true });
      } else {
        await updateMutation.mutateAsync({
          id: id!,
          request: {
            name: playbookName,
            description: playbookDescription || undefined,
            steps,
          },
        });
      }
    } finally {
      setIsSaving(false);
    }
  };

  const handleRun = async () => {
    if (!id) return;
    const run = await runMutation.mutateAsync({ id, request: {} });
    navigate(`/operations/playbooks/${id}/runs/${run.id}`);
  };

  if (!isNew && isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-4rem)]">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  return (
    <div className="h-[calc(100vh-4rem)] flex flex-col -m-6">
      {/* Header */}
      <div className="h-14 border-b border-gray-200 bg-white flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate('/operations/playbooks')}
            className="p-2 text-gray-500 hover:text-gray-700 rounded-lg hover:bg-gray-100"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <input
            type="text"
            value={playbookName}
            onChange={(e) => setPlaybookName(e.target.value)}
            className="text-lg font-semibold text-gray-900 bg-transparent border-none outline-none focus:ring-0"
            placeholder="Playbook name..."
          />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-200 rounded-lg hover:bg-gray-50 disabled:opacity-50"
          >
            {isSaving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            Save
          </button>
          {!isNew && (
            <button
              onClick={handleRun}
              disabled={runMutation.isPending}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              {runMutation.isPending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Play className="w-4 h-4" />
              )}
              Run
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 flex">
        {/* Toolbox */}
        <PlaybookToolbox />

        {/* Canvas */}
        <div className="flex-1 bg-gray-50">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onDragOver={onDragOver}
            onDrop={onDrop}
            nodeTypes={nodeTypes}
            fitView
            snapToGrid
            snapGrid={[15, 15]}
          >
            <Background gap={15} size={1} />
            <Controls />
            <MiniMap
              nodeStrokeWidth={3}
              zoomable
              pannable
              className="bg-white border border-gray-200 rounded-lg"
            />
            <Panel position="top-right" className="bg-white rounded-lg shadow-sm border border-gray-200 p-2">
              <div className="text-xs text-gray-500">
                {nodes.length} steps · {edges.length} connections
              </div>
            </Panel>
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/pages/AutomationPlaybookEditorPage.tsx
git commit -m "feat(playbooks): add visual playbook editor page"
```

---

## Task 10: Playbooks List Page

**Files:**
- Create: `packages/frontend/web-app/src/pages/AutomationPlaybooksListPage.tsx`

**Step 1: Create the list page**

```typescript
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Plus,
  Search,
  Play,
  Edit,
  Trash2,
  Loader2,
  AlertCircle,
  Workflow,
  Clock,
  Zap,
  Globe,
} from 'lucide-react';

import { usePlaybooks, useDeletePlaybook, useRunPlaybook } from '../hooks/usePlaybooks';
import type { TriggerType } from '../services/playbooks';

const triggerIcons: Record<TriggerType, React.ElementType> = {
  manual: Play,
  scheduled: Clock,
  webhook: Globe,
  event: Zap,
};

const triggerLabels: Record<TriggerType, string> = {
  manual: 'Manual',
  scheduled: 'Scheduled',
  webhook: 'Webhook',
  event: 'Event',
};

export function AutomationPlaybooksListPage() {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');

  const { data, isLoading, error } = usePlaybooks();
  const deleteMutation = useDeletePlaybook();
  const runMutation = useRunPlaybook();

  const playbooks = data?.playbooks ?? [];
  const filteredPlaybooks = playbooks.filter(
    (p) =>
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleDelete = async (id: string, name: string) => {
    if (confirm(`Delete playbook "${name}"? This cannot be undone.`)) {
      await deleteMutation.mutateAsync(id);
    }
  };

  const handleRun = async (id: string) => {
    const run = await runMutation.mutateAsync({ id, request: {} });
    navigate(`/operations/playbooks/${id}/runs/${run.id}`);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Automation Playbooks</h1>
          <p className="text-gray-500">Visual workflow automation for operations and integrations</p>
        </div>
        <button
          onClick={() => navigate('/operations/playbooks/new')}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          New Playbook
        </button>
      </div>

      {/* Search */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search playbooks..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
      </div>

      {/* Loading */}
      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-primary-500 animate-spin" />
          <span className="ml-2 text-gray-500">Loading playbooks...</span>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="flex items-center gap-3 p-4 bg-red-50 border border-red-200 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <div>
            <p className="font-medium text-red-800">Failed to load playbooks</p>
            <p className="text-sm text-red-600">{error.message}</p>
          </div>
        </div>
      )}

      {/* Empty */}
      {!isLoading && !error && filteredPlaybooks.length === 0 && (
        <div className="text-center py-12">
          <Workflow className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900">No playbooks found</h3>
          <p className="text-gray-500 mt-1">
            {searchQuery ? 'Try a different search term' : 'Create your first automation playbook'}
          </p>
          {!searchQuery && (
            <button
              onClick={() => navigate('/operations/playbooks/new')}
              className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
            >
              <Plus className="w-4 h-4" />
              New Playbook
            </button>
          )}
        </div>
      )}

      {/* Playbooks Table */}
      {!isLoading && !error && filteredPlaybooks.length > 0 && (
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="bg-gray-50 border-b border-gray-200">
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">
                  Name
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">
                  Trigger
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">
                  Steps
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">
                  Last Updated
                </th>
                <th className="text-right text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {filteredPlaybooks.map((playbook) => {
                const TriggerIcon = triggerIcons[playbook.trigger_type];
                return (
                  <tr key={playbook.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4">
                      <button
                        onClick={() => navigate(`/operations/playbooks/${playbook.id}/edit`)}
                        className="text-left"
                      >
                        <div className="font-medium text-gray-900 hover:text-primary-600">
                          {playbook.name}
                        </div>
                        {playbook.description && (
                          <div className="text-sm text-gray-500 truncate max-w-md">
                            {playbook.description}
                          </div>
                        )}
                      </button>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2 text-sm text-gray-600">
                        <TriggerIcon className="w-4 h-4" />
                        {triggerLabels[playbook.trigger_type]}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-gray-600">{playbook.step_count} steps</span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-gray-500">
                        {new Date(playbook.updated_at).toLocaleDateString()}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleRun(playbook.id)}
                          disabled={runMutation.isPending}
                          className="p-2 text-gray-500 hover:text-green-600 hover:bg-green-50 rounded-lg"
                          title="Run"
                        >
                          <Play className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => navigate(`/operations/playbooks/${playbook.id}/edit`)}
                          className="p-2 text-gray-500 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                          title="Edit"
                        >
                          <Edit className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(playbook.id, playbook.name)}
                          disabled={deleteMutation.isPending}
                          className="p-2 text-gray-500 hover:text-red-600 hover:bg-red-50 rounded-lg"
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/pages/AutomationPlaybooksListPage.tsx
git commit -m "feat(playbooks): add playbooks list page"
```

---

## Task 11: Playbook Run Detail Page

**Files:**
- Create: `packages/frontend/web-app/src/pages/PlaybookRunDetailPage.tsx`

**Step 1: Create the run detail page**

```typescript
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Panel,
  type Node,
  type Edge,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { ArrowLeft, Loader2, CheckCircle, XCircle, Clock, AlertCircle } from 'lucide-react';

import {
  IntegrationStepNode,
  WebhookStepNode,
  WaitStepNode,
  ConditionStepNode,
  ApprovalStepNode,
} from '../components/playbooks/nodes';
import { usePlaybook, usePlaybookRun, useApproveRun, useRejectRun } from '../hooks/usePlaybooks';

const nodeTypes: NodeTypes = {
  integration: IntegrationStepNode,
  webhook: WebhookStepNode,
  wait: WaitStepNode,
  condition: ConditionStepNode,
  approval: ApprovalStepNode,
};

const statusIcons = {
  pending: Clock,
  running: Loader2,
  waiting_approval: AlertCircle,
  completed: CheckCircle,
  failed: XCircle,
  cancelled: XCircle,
};

const statusColors = {
  pending: 'text-gray-500 bg-gray-100',
  running: 'text-blue-600 bg-blue-100',
  waiting_approval: 'text-yellow-600 bg-yellow-100',
  completed: 'text-green-600 bg-green-100',
  failed: 'text-red-600 bg-red-100',
  cancelled: 'text-gray-600 bg-gray-100',
};

export function PlaybookRunDetailPage() {
  const { id: playbookId, runId } = useParams();
  const navigate = useNavigate();

  const { data: playbook } = usePlaybook(playbookId ?? '');
  const { data: run, isLoading } = usePlaybookRun(runId ?? '', 2000);
  const approveMutation = useApproveRun();
  const rejectMutation = useRejectRun();

  if (isLoading || !playbook || !run) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-4rem)]">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  // Build step status map
  const stepStatusMap = new Map(run.step_states.map((s) => [s.step_id, s.status]));

  // Convert steps to nodes with status
  const nodes: Node[] = playbook.steps.map((step, index) => ({
    id: step.id,
    type: step.step_type,
    position: { x: 100 + index * 250, y: 200 },
    data: {
      id: step.id,
      name: step.name,
      config: step.config,
      status: stepStatusMap.get(step.id) ?? 'pending',
    },
  }));

  // Create edges
  const edges: Edge[] = [];
  playbook.steps.forEach((step) => {
    step.on_success.forEach((targetId) => {
      edges.push({
        id: `${step.id}-${targetId}`,
        source: step.id,
        target: targetId,
        type: 'default',
      });
    });
  });

  const StatusIcon = statusIcons[run.status];

  return (
    <div className="h-[calc(100vh-4rem)] flex flex-col -m-6">
      {/* Header */}
      <div className="h-14 border-b border-gray-200 bg-white flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate(`/operations/playbooks/${playbookId}/runs`)}
            className="p-2 text-gray-500 hover:text-gray-700 rounded-lg hover:bg-gray-100"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <div>
            <h1 className="font-semibold text-gray-900">{playbook.name}</h1>
            <p className="text-xs text-gray-500">Run {runId?.slice(0, 8)}...</p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium ${statusColors[run.status]}`}>
            <StatusIcon className={`w-4 h-4 ${run.status === 'running' ? 'animate-spin' : ''}`} />
            {run.status.replace('_', ' ')}
          </div>
          {run.status === 'waiting_approval' && (
            <div className="flex items-center gap-2">
              <button
                onClick={() => rejectMutation.mutate(runId!)}
                disabled={rejectMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-red-700 bg-red-50 rounded-lg hover:bg-red-100"
              >
                Reject
              </button>
              <button
                onClick={() => approveMutation.mutate(runId!)}
                disabled={approveMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-green-600 rounded-lg hover:bg-green-700"
              >
                Approve
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="flex-1 flex">
        {/* Canvas (read-only) */}
        <div className="flex-1 bg-gray-50">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            fitView
            nodesDraggable={false}
            nodesConnectable={false}
            elementsSelectable={false}
          >
            <Background gap={15} size={1} />
            <Controls showInteractive={false} />
            <MiniMap
              nodeStrokeWidth={3}
              zoomable
              pannable
              className="bg-white border border-gray-200 rounded-lg"
            />
            <Panel position="top-left" className="bg-white rounded-lg shadow-sm border border-gray-200 p-3">
              <div className="text-sm font-medium text-gray-900 mb-2">Run Progress</div>
              <div className="space-y-1 text-xs">
                {run.step_states.map((state) => (
                  <div key={state.step_id} className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${
                      state.status === 'completed' ? 'bg-green-500' :
                      state.status === 'failed' ? 'bg-red-500' :
                      state.status === 'running' ? 'bg-blue-500 animate-pulse' :
                      'bg-gray-300'
                    }`} />
                    <span className="text-gray-600 truncate max-w-[150px]">
                      {playbook.steps.find(s => s.id === state.step_id)?.name ?? state.step_id}
                    </span>
                  </div>
                ))}
              </div>
            </Panel>
          </ReactFlow>
        </div>

        {/* Sidebar */}
        <div className="w-80 bg-white border-l border-gray-200 p-4 overflow-y-auto">
          <h3 className="font-semibold text-gray-900 mb-4">Run Details</h3>

          <div className="space-y-4">
            <div>
              <label className="text-xs font-medium text-gray-500">Started</label>
              <p className="text-sm text-gray-900">
                {new Date(run.started_at).toLocaleString()}
              </p>
            </div>

            {run.completed_at && (
              <div>
                <label className="text-xs font-medium text-gray-500">Completed</label>
                <p className="text-sm text-gray-900">
                  {new Date(run.completed_at).toLocaleString()}
                </p>
              </div>
            )}

            {Object.keys(run.variables).length > 0 && (
              <div>
                <label className="text-xs font-medium text-gray-500">Variables</label>
                <pre className="mt-1 p-2 bg-gray-50 rounded text-xs overflow-x-auto">
                  {JSON.stringify(run.variables, null, 2)}
                </pre>
              </div>
            )}

            <div>
              <label className="text-xs font-medium text-gray-500 mb-2 block">Step Timeline</label>
              <div className="space-y-2">
                {run.step_states.map((state) => {
                  const step = playbook.steps.find(s => s.id === state.step_id);
                  return (
                    <div
                      key={state.step_id}
                      className="p-2 bg-gray-50 rounded-lg text-sm"
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-gray-900">{step?.name ?? state.step_id}</span>
                        <span className={`text-xs px-1.5 py-0.5 rounded ${
                          state.status === 'completed' ? 'bg-green-100 text-green-700' :
                          state.status === 'failed' ? 'bg-red-100 text-red-700' :
                          state.status === 'running' ? 'bg-blue-100 text-blue-700' :
                          'bg-gray-100 text-gray-600'
                        }`}>
                          {state.status}
                        </span>
                      </div>
                      {state.error && (
                        <p className="mt-1 text-xs text-red-600">{state.error}</p>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add packages/frontend/web-app/src/pages/PlaybookRunDetailPage.tsx
git commit -m "feat(playbooks): add run detail page with live status"
```

---

## Task 12: Register Routes and Navigation

**Files:**
- Modify: `packages/frontend/web-app/src/App.tsx`
- Modify: `packages/frontend/web-app/src/components/layout/Sidebar.tsx`

**Step 1: Add routes to App.tsx**

Add imports:
```typescript
import { AutomationPlaybooksListPage } from './pages/AutomationPlaybooksListPage';
import { AutomationPlaybookEditorPage } from './pages/AutomationPlaybookEditorPage';
import { PlaybookRunDetailPage } from './pages/PlaybookRunDetailPage';
```

Add routes after `{/* Operations Center */}`:
```typescript
<Route path="operations/playbooks" element={<AutomationPlaybooksListPage />} />
<Route path="operations/playbooks/new" element={<AutomationPlaybookEditorPage />} />
<Route path="operations/playbooks/:id/edit" element={<AutomationPlaybookEditorPage />} />
<Route path="operations/playbooks/:id/runs" element={<AutomationPlaybooksListPage />} />
<Route path="operations/playbooks/:id/runs/:runId" element={<PlaybookRunDetailPage />} />
```

**Step 2: Add navigation link to Sidebar**

In the Operations Center section of Sidebar.tsx, add:
```typescript
{ name: 'Playbooks', href: '/operations/playbooks', icon: Workflow },
```

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/App.tsx packages/frontend/web-app/src/components/layout/Sidebar.tsx
git commit -m "feat(playbooks): register routes and add navigation"
```

---

## Task 13: Build and Verify

**Step 1: Build backend**

Run: `cd services/integration-service && cargo build`
Expected: SUCCESS

**Step 2: Build frontend**

Run: `cd packages/frontend/web-app && npm run build`
Expected: SUCCESS

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat(playbooks): complete automation playbooks feature"
```

---

## Implementation Phases Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1. Backend Foundation | 1-4 | Database schema, domain types, storage, API |
| 2. Frontend Services | 5-6 | API service and React Query hooks |
| 3. Visual Editor | 7-9 | Step nodes, toolbox, editor page |
| 4. List & Run Views | 10-12 | List page, run detail, routing |
| 5. Integration | 13 | Build verification |

**Total estimated time:** 4-6 hours

**Future enhancements (not in this plan):**
- Kafka step dispatch and result handling
- Condition expression evaluator
- Scheduled trigger support
- Webhook trigger endpoint
- Step configuration panels

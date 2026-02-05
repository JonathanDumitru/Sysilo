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
use crate::playbooks::executor::PlaybookExecutor;
use crate::playbooks::{Step, StepState, StepStatus, TriggerType, Variable};
use crate::storage::{PlaybookRow, PlaybookRunRow};
use crate::AppState;

// =============================================================================
// Request/Response Types
// =============================================================================

/// Response for listing playbooks
#[derive(Debug, Serialize)]
pub struct PlaybookListResponse {
    pub playbooks: Vec<PlaybookSummary>,
    pub total: i64,
}

/// Summary of a playbook for list views
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

/// Full playbook response
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

/// Request to create a new playbook
#[derive(Debug, Deserialize)]
pub struct CreatePlaybookRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub trigger_type: TriggerType,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub variables: Vec<Variable>,
}

/// Request to update an existing playbook
#[derive(Debug, Deserialize)]
pub struct UpdatePlaybookRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub steps: Vec<Step>,
    pub variables: Vec<Variable>,
}

/// Request to run a playbook
#[derive(Debug, Deserialize)]
pub struct RunPlaybookRequest {
    #[serde(default)]
    pub variables: serde_json::Value,
}

/// Response for a playbook run
#[derive(Debug, Serialize)]
pub struct PlaybookRunResponse {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub status: String,
    pub variables: serde_json::Value,
    pub step_states: Vec<StepState>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Response for listing playbook runs
#[derive(Debug, Serialize)]
pub struct PlaybookRunListResponse {
    pub runs: Vec<PlaybookRunSummary>,
    pub total: i64,
}

/// Summary of a playbook run for list views
#[derive(Debug, Serialize)]
pub struct PlaybookRunSummary {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Request to approve or reject a run (optional reason)
#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

/// Response for approval/rejection
#[derive(Debug, Serialize)]
pub struct ApprovalResponse {
    pub run_id: Uuid,
    pub status: String,
    pub message: String,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn playbook_row_to_response(row: PlaybookRow) -> Result<PlaybookResponse, ApiError> {
    let steps: Vec<Step> = serde_json::from_value(row.steps).map_err(|e| ApiError {
        error: "invalid_data".to_string(),
        message: format!("Failed to parse steps: {}", e),
    })?;

    let variables: Vec<Variable> = serde_json::from_value(row.variables).map_err(|e| ApiError {
        error: "invalid_data".to_string(),
        message: format!("Failed to parse variables: {}", e),
    })?;

    Ok(PlaybookResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        trigger_type: row.trigger_type,
        steps,
        variables,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    })
}

fn playbook_row_to_summary(row: PlaybookRow) -> PlaybookSummary {
    let step_count = row
        .steps
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    PlaybookSummary {
        id: row.id,
        name: row.name,
        description: row.description,
        trigger_type: row.trigger_type,
        step_count,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

fn run_row_to_response(row: PlaybookRunRow) -> Result<PlaybookRunResponse, ApiError> {
    let step_states: Vec<StepState> =
        serde_json::from_value(row.step_states).map_err(|e| ApiError {
            error: "invalid_data".to_string(),
            message: format!("Failed to parse step_states: {}", e),
        })?;

    Ok(PlaybookRunResponse {
        id: row.id,
        playbook_id: row.playbook_id,
        status: row.status,
        variables: row.variables,
        step_states,
        started_at: row.started_at.to_rfc3339(),
        completed_at: row.completed_at.map(|t| t.to_rfc3339()),
    })
}

fn run_row_to_summary(row: PlaybookRunRow) -> PlaybookRunSummary {
    PlaybookRunSummary {
        id: row.id,
        playbook_id: row.playbook_id,
        status: row.status,
        started_at: row.started_at.to_rfc3339(),
        completed_at: row.completed_at.map(|t| t.to_rfc3339()),
    }
}

// =============================================================================
// Playbook CRUD Handlers
// =============================================================================

/// GET /playbooks - List all playbooks for the tenant
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

    let playbooks: Vec<PlaybookSummary> = rows.into_iter().map(playbook_row_to_summary).collect();
    let total = playbooks.len() as i64;

    Ok(Json(PlaybookListResponse { playbooks, total }))
}

/// POST /playbooks - Create a new playbook
pub async fn create_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<CreatePlaybookRequest>,
) -> Result<(StatusCode, Json<PlaybookResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Serialize trigger_type to string
    let trigger_type = serde_json::to_value(&req.trigger_type)
        .map_err(|e| ApiError {
            error: "serialization_error".to_string(),
            message: e.to_string(),
        })?
        .as_str()
        .unwrap_or("manual")
        .to_string();

    let steps = serde_json::to_value(&req.steps).map_err(|e| ApiError {
        error: "invalid_steps".to_string(),
        message: e.to_string(),
    })?;

    let variables = serde_json::to_value(&req.variables).map_err(|e| ApiError {
        error: "invalid_variables".to_string(),
        message: e.to_string(),
    })?;

    let row = state
        .storage
        .create_playbook(
            &tenant_id,
            &req.name,
            req.description.as_deref(),
            &trigger_type,
            steps,
            variables,
        )
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let response = playbook_row_to_response(row)?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /playbooks/:id - Get a single playbook
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

    let response = playbook_row_to_response(row)?;
    Ok(Json(response))
}

/// PUT /playbooks/:id - Update an existing playbook
pub async fn update_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaybookRequest>,
) -> Result<Json<PlaybookResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Serialize trigger_type to string
    let trigger_type = serde_json::to_value(&req.trigger_type)
        .map_err(|e| ApiError {
            error: "serialization_error".to_string(),
            message: e.to_string(),
        })?
        .as_str()
        .unwrap_or("manual")
        .to_string();

    let steps = serde_json::to_value(&req.steps).map_err(|e| ApiError {
        error: "invalid_steps".to_string(),
        message: e.to_string(),
    })?;

    let variables = serde_json::to_value(&req.variables).map_err(|e| ApiError {
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
            &trigger_type,
            steps,
            variables,
        )
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let response = playbook_row_to_response(row)?;
    Ok(Json(response))
}

/// DELETE /playbooks/:id - Delete a playbook
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

// =============================================================================
// Playbook Run Handlers
// =============================================================================

/// POST /playbooks/:id/run - Start a new playbook run
pub async fn run_playbook(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<RunPlaybookRequest>,
) -> Result<(StatusCode, Json<PlaybookRunResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the playbook to retrieve its steps
    let playbook = state
        .storage
        .get_playbook(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Parse steps to create initial step states
    let steps: Vec<Step> = serde_json::from_value(playbook.steps).map_err(|e| ApiError {
        error: "invalid_playbook".to_string(),
        message: format!("Failed to parse playbook steps: {}", e),
    })?;

    // Initialize step states (all pending)
    let initial_step_states: Vec<StepState> = steps
        .iter()
        .map(|step| StepState {
            step_id: step.id.clone(),
            status: StepStatus::Pending,
            started_at: None,
            completed_at: None,
            output: None,
            error: None,
        })
        .collect();

    let step_states_json = serde_json::to_value(&initial_step_states).map_err(|e| ApiError {
        error: "serialization_error".to_string(),
        message: e.to_string(),
    })?;

    let row = state
        .storage
        .create_playbook_run(&tenant_id, id, req.variables.clone(), step_states_json)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // Dispatch starting steps to Kafka (if producer available)
    if let Some(producer) = state.engine.kafka_producer() {
        if let Err(e) = PlaybookExecutor::start_run(
            producer,
            &state.storage,
            row.id,
            id,
            &tenant_id,
            &steps,
            &req.variables,
        )
        .await
        {
            tracing::error!(
                run_id = %row.id,
                error = %e,
                "Failed to dispatch playbook steps - run created but not started"
            );
            // Run is created but stays in pending - user can retry
        }
    } else {
        tracing::warn!(
            run_id = %row.id,
            "No Kafka producer available - playbook run created but steps not dispatched"
        );
    }

    tracing::info!(
        playbook_id = %id,
        run_id = %row.id,
        tenant_id = %tenant_id,
        "Playbook run started"
    );

    let response = run_row_to_response(row)?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// GET /playbooks/:id/runs - List runs for a playbook
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

    let runs: Vec<PlaybookRunSummary> = rows.into_iter().map(run_row_to_summary).collect();
    let total = runs.len() as i64;

    Ok(Json(PlaybookRunListResponse { runs, total }))
}

/// GET /runs/:id - Get a single playbook run
pub async fn get_playbook_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    let response = run_row_to_response(row)?;
    Ok(Json(response))
}

/// POST /runs/:id/approve - Approve a playbook run waiting for approval
pub async fn approve_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<ApprovalResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the current run
    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Check if the run is waiting for approval
    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: format!(
                "Run is in '{}' state, expected 'waiting_approval'",
                row.status
            ),
        });
    }

    // Update status to running (resume execution)
    state
        .storage
        .update_playbook_run(id, "running", row.step_states)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(
        run_id = %id,
        tenant_id = %tenant_id,
        reason = ?req.reason,
        "Playbook run approved"
    );

    Ok(Json(ApprovalResponse {
        run_id: id,
        status: "running".to_string(),
        message: "Run approved and resumed".to_string(),
    }))
}

/// POST /runs/:id/reject - Reject a playbook run waiting for approval
pub async fn reject_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<ApprovalResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the current run
    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Check if the run is waiting for approval
    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: format!(
                "Run is in '{}' state, expected 'waiting_approval'",
                row.status
            ),
        });
    }

    // Update status to cancelled
    state
        .storage
        .update_playbook_run(id, "cancelled", row.step_states)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(
        run_id = %id,
        tenant_id = %tenant_id,
        reason = ?req.reason,
        "Playbook run rejected"
    );

    Ok(Json(ApprovalResponse {
        run_id: id,
        status: "cancelled".to_string(),
        message: req
            .reason
            .unwrap_or_else(|| "Run rejected by user".to_string()),
    }))
}

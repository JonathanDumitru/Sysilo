use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::IntegrationDefinition;
use crate::middleware::TenantContext;
use crate::AppState;

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        (status, Json(self)).into_response()
    }
}

// Health endpoints

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "healthy"}))
}

pub async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.storage.health_check().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ready"}))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not ready", "reason": "database unavailable"})),
        ),
    }
}

// Integration endpoints

#[derive(Debug, Serialize)]
pub struct IntegrationListResponse {
    pub integrations: Vec<IntegrationSummary>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct IntegrationSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

pub async fn list_integrations(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<IntegrationListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = state
        .storage
        .list_integrations(&tenant_id, 100, 0)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let integrations: Vec<IntegrationSummary> = rows
        .into_iter()
        .map(|r| IntegrationSummary {
            id: r.id,
            name: r.name,
            description: r.description,
            status: r.status,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    let total = integrations.len() as i64;

    Ok(Json(IntegrationListResponse {
        integrations,
        total,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateIntegrationRequest {
    pub name: String,
    pub description: Option<String>,
    pub definition: IntegrationDefinition,
}

#[derive(Debug, Serialize)]
pub struct IntegrationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub definition: serde_json::Value,
    pub version: i32,
    pub status: String,
    pub created_at: String,
}

pub async fn create_integration(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<CreateIntegrationRequest>,
) -> Result<(StatusCode, Json<IntegrationResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let definition = serde_json::to_value(&req.definition).map_err(|e| ApiError {
        error: "invalid_definition".to_string(),
        message: e.to_string(),
    })?;

    let row = state
        .storage
        .create_integration(&tenant_id, &req.name, req.description.as_deref(), definition)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(IntegrationResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            definition: row.definition,
            version: row.version,
            status: row.status,
            created_at: row.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn get_integration(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<IntegrationResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_integration(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(IntegrationResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        definition: row.definition,
        version: row.version,
        status: row.status,
        created_at: row.created_at.to_rfc3339(),
    }))
}

// Run endpoints

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub status: String,
    pub trigger_type: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub metrics: serde_json::Value,
}

pub async fn run_integration(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<RunResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the integration
    let integration = state
        .storage
        .get_integration(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Parse the definition
    let definition: IntegrationDefinition =
        serde_json::from_value(integration.definition.clone()).map_err(|e| ApiError {
            error: "invalid_definition".to_string(),
            message: e.to_string(),
        })?;

    // Start the run via the engine
    let run = state
        .engine
        .start_run(id, tenant_id.clone(), definition, "manual".to_string())
        .await
        .map_err(|e| ApiError {
            error: "execution_error".to_string(),
            message: e.to_string(),
        })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(RunResponse {
            id: run.id,
            integration_id: run.integration_id,
            status: format!("{:?}", run.status).to_lowercase(),
            trigger_type: run.trigger_type,
            started_at: run.started_at.map(|t| t.to_rfc3339()),
            completed_at: run.completed_at.map(|t| t.to_rfc3339()),
            error_message: run.error_message,
            metrics: serde_json::to_value(&run.metrics).unwrap_or_default(),
        }),
    ))
}

pub async fn get_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RunResponse>, ApiError> {
    // First check in-memory active runs
    if let Some(run) = state.engine.get_run(id).await {
        return Ok(Json(RunResponse {
            id: run.id,
            integration_id: run.integration_id,
            status: format!("{:?}", run.status).to_lowercase(),
            trigger_type: run.trigger_type,
            started_at: run.started_at.map(|t| t.to_rfc3339()),
            completed_at: run.completed_at.map(|t| t.to_rfc3339()),
            error_message: run.error_message,
            metrics: serde_json::to_value(&run.metrics).unwrap_or_default(),
        }));
    }

    let tenant_id = tenant.tenant_id.to_string();

    // Fall back to database
    let row = state
        .storage
        .get_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(RunResponse {
        id: row.id,
        integration_id: row.integration_id,
        status: row.status,
        trigger_type: row.trigger_type,
        started_at: row.started_at.map(|t| t.to_rfc3339()),
        completed_at: row.completed_at.map(|t| t.to_rfc3339()),
        error_message: row.error_message,
        metrics: row.metrics,
    }))
}

pub async fn cancel_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<RunResponse>, ApiError> {
    let run = state.engine.cancel_run(id).await.map_err(|e| ApiError {
        error: "cancel_error".to_string(),
        message: e.to_string(),
    })?;

    Ok(Json(RunResponse {
        id: run.id,
        integration_id: run.integration_id,
        status: format!("{:?}", run.status).to_lowercase(),
        trigger_type: run.trigger_type,
        started_at: run.started_at.map(|t| t.to_rfc3339()),
        completed_at: run.completed_at.map(|t| t.to_rfc3339()),
        error_message: run.error_message,
        metrics: serde_json::to_value(&run.metrics).unwrap_or_default(),
    }))
}

// Discovery endpoints

/// Request to start a discovery run
#[derive(Debug, Deserialize)]
pub struct DiscoveryRequest {
    /// Connection to discover against
    pub connection_id: Uuid,
    /// Type of discovery (full or incremental)
    #[serde(default)]
    pub discovery_type: DiscoveryType,
    /// Optional resource type filters
    #[serde(default)]
    pub resource_types: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryType {
    #[default]
    Full,
    Incremental,
}

/// Response from starting discovery
#[derive(Debug, Serialize)]
pub struct DiscoveryResponse {
    pub run_id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Start a discovery run against a connection
pub async fn run_discovery(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<DiscoveryRequest>,
) -> Result<(StatusCode, Json<DiscoveryResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let run_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    // Create a discovery task
    let task = crate::engine::Task {
        id: task_id,
        run_id,
        integration_id: Uuid::nil(), // No integration - direct discovery
        tenant_id: tenant_id.clone(),
        task_type: "discovery".to_string(),
        config: serde_json::json!({
            "connection_id": req.connection_id,
            "discovery_type": format!("{:?}", req.discovery_type).to_lowercase(),
            "resource_types": req.resource_types,
        }),
        priority: 2,
        timeout_seconds: 300, // 5 minute timeout for discovery
        sequence: 0,
        depends_on: vec![],
    };

    // Send to Kafka if producer available
    if let Some(producer) = state.engine.kafka_producer() {
        producer.send_task(&task).await.map_err(|e| ApiError {
            error: "dispatch_error".to_string(),
            message: e.to_string(),
        })?;

        tracing::info!(
            run_id = %run_id,
            task_id = %task_id,
            connection_id = %req.connection_id,
            "Discovery task dispatched"
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

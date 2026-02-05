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

// =============================================================================
// Development/Mock endpoints (only for local testing)
// =============================================================================

/// Request to simulate discovery results (dev only)
#[derive(Debug, Deserialize)]
pub struct MockDiscoveryRequest {
    /// Connection ID that was "discovered"
    pub connection_id: Uuid,
    /// Number of mock assets to generate (default: 5)
    #[serde(default = "default_asset_count")]
    pub asset_count: usize,
}

fn default_asset_count() -> usize {
    5
}

/// Response from mock discovery
#[derive(Debug, Serialize)]
pub struct MockDiscoveryResponse {
    pub message: String,
    pub assets_created: usize,
}

/// Generate mock discovery results and send them to asset-service
/// This bypasses Kafka entirely for local development testing
pub async fn mock_discovery_result(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<MockDiscoveryRequest>,
) -> Result<Json<MockDiscoveryResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Generate mock discovered assets based on connection type
    let mock_assets = generate_mock_assets(req.connection_id, req.asset_count);

    // Send each asset to the asset-service
    let client = reqwest::Client::new();
    let asset_service_url = state
        .config
        .consumer
        .asset_service_url
        .clone();

    let mut created_count = 0;

    for asset in &mock_assets {
        let create_request = serde_json::json!({
            "name": asset.name,
            "asset_type": asset.asset_type,
            "description": asset.description,
            "vendor": asset.vendor,
            "version": asset.version,
            "status": "active",
            "metadata": asset.metadata,
            "tags": asset.tags,
        });

        match client
            .post(format!("{}/api/v1/assets", asset_service_url))
            .header("X-Tenant-ID", &tenant_id)
            .header("Content-Type", "application/json")
            .json(&create_request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                created_count += 1;
                tracing::info!(
                    asset_name = %asset.name,
                    asset_type = %asset.asset_type,
                    "Mock asset created"
                );
            }
            Ok(response) => {
                tracing::warn!(
                    asset_name = %asset.name,
                    status = %response.status(),
                    "Failed to create mock asset"
                );
            }
            Err(e) => {
                tracing::error!(
                    asset_name = %asset.name,
                    error = %e,
                    "Error creating mock asset"
                );
            }
        }
    }

    Ok(Json(MockDiscoveryResponse {
        message: format!(
            "Mock discovery complete for connection {}",
            req.connection_id
        ),
        assets_created: created_count,
    }))
}

/// Mock asset for generation
struct MockAsset {
    name: String,
    asset_type: String,
    description: Option<String>,
    vendor: Option<String>,
    version: Option<String>,
    metadata: serde_json::Value,
    tags: Vec<String>,
}

/// Generate realistic mock assets based on connection type
fn generate_mock_assets(connection_id: Uuid, count: usize) -> Vec<MockAsset> {
    // Use connection_id to deterministically generate different asset sets
    let seed = connection_id.as_u128() as usize;

    let database_assets = vec![
        ("users", "Stores user accounts and profiles"),
        ("orders", "Customer order transactions"),
        ("products", "Product catalog and inventory"),
        ("sessions", "Active user sessions"),
        ("audit_logs", "System audit trail"),
        ("payments", "Payment transactions"),
        ("notifications", "User notifications queue"),
        ("analytics_events", "User behavior events"),
    ];

    let api_assets = vec![
        ("User API", "RESTful API for user management"),
        ("Orders API", "Order processing endpoints"),
        ("Auth Service", "Authentication and authorization"),
        ("Notification Service", "Push and email notifications"),
        ("Search API", "Full-text search endpoints"),
        ("Analytics API", "Reporting and dashboards"),
    ];

    let vendors = vec!["PostgreSQL", "MySQL", "Snowflake", "Salesforce", "AWS", "Azure"];
    let versions = vec!["15.4", "8.0", "2024.1", "v2", "3.11", "14.2"];

    let mut assets = Vec::new();

    for i in 0..count {
        let idx = (seed + i) % 8;
        let is_database = (seed + i) % 3 != 0;

        if is_database {
            let (name, desc) = &database_assets[idx % database_assets.len()];
            assets.push(MockAsset {
                name: format!("{}_{}", name, (seed + i) % 100),
                asset_type: "database".to_string(),
                description: Some(desc.to_string()),
                vendor: Some(vendors[idx % vendors.len()].to_string()),
                version: Some(versions[idx % versions.len()].to_string()),
                metadata: serde_json::json!({
                    "discovered_at": chrono::Utc::now().to_rfc3339(),
                    "connection_id": connection_id.to_string(),
                    "row_count_estimate": (idx + 1) * 10000,
                }),
                tags: vec![
                    "discovered".to_string(),
                    if idx % 2 == 0 { "production" } else { "staging" }.to_string(),
                ],
            });
        } else {
            let (name, desc) = &api_assets[idx % api_assets.len()];
            assets.push(MockAsset {
                name: name.to_string(),
                asset_type: "api".to_string(),
                description: Some(desc.to_string()),
                vendor: Some(vendors[(idx + 2) % vendors.len()].to_string()),
                version: Some(format!("v{}.{}", idx % 3 + 1, idx % 10)),
                metadata: serde_json::json!({
                    "discovered_at": chrono::Utc::now().to_rfc3339(),
                    "connection_id": connection_id.to_string(),
                    "endpoint_count": (idx + 1) * 5,
                }),
                tags: vec![
                    "discovered".to_string(),
                    "api".to_string(),
                ],
            });
        }
    }

    assets
}

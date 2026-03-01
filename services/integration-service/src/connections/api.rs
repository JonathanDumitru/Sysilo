use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
use crate::connections::{validate_config, AuthType, ConnectorType};
use crate::middleware::TenantContext;
use crate::storage::ConnectionRow;
use crate::AppState;

// === Response types ===

#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub name: String,
    pub connector_type: String,
    pub auth_type: String,
    pub config: serde_json::Value,
    pub has_credentials: bool,
    pub status: String,
    pub last_tested_at: Option<String>,
    pub last_test_status: Option<String>,
    pub last_test_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionListResponse {
    pub connections: Vec<ConnectionResponse>,
    pub total: usize,
}

// === Request types ===

#[derive(Debug, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub connector_type: ConnectorType,
    pub auth_type: AuthType,
    pub config: serde_json::Value,
    #[serde(default)]
    pub credentials: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConnectionRequest {
    pub name: String,
    pub config: serde_json::Value,
    pub credentials: Option<serde_json::Value>,
}

// === Helper to convert row to response ===

fn row_to_response(row: ConnectionRow) -> ConnectionResponse {
    ConnectionResponse {
        id: row.id,
        name: row.name,
        connector_type: row.connector_type,
        auth_type: row.auth_type,
        config: row.config,
        has_credentials: row.credentials != serde_json::json!({}),
        status: row.status,
        last_tested_at: row.last_tested_at.map(|t| t.to_rfc3339()),
        last_test_status: row.last_test_status,
        last_test_error: row.last_test_error,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

fn parse_connector_type(raw: &str) -> Result<ConnectorType, ApiError> {
    serde_json::from_value(serde_json::json!(raw)).map_err(|e| ApiError {
        error: "invalid_connector_type".to_string(),
        message: e.to_string(),
        status: Some(StatusCode::BAD_REQUEST),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })
}

fn enforce_production_guard(headers: &HeaderMap, environment: &str) -> Result<(), ApiError> {
    if environment != "prod" {
        return Ok(());
    }

    let confirmed = headers
        .get("x-production-confirmed")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !confirmed {
        return Err(ApiError {
            error: "production_confirmation_required".to_string(),
            message: "Production mutation requires explicit confirmation".to_string(),
            status: Some(StatusCode::FORBIDDEN),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        });
    }

    let reason = headers
        .get("x-change-reason")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or("");
    if reason.is_empty() {
        return Err(ApiError {
            error: "production_change_reason_required".to_string(),
            message: "Production mutation requires a non-empty change reason".to_string(),
            status: Some(StatusCode::BAD_REQUEST),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        });
    }

    Ok(())
}

// === Handlers ===

/// GET /connections
pub async fn list_connections(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<ConnectionListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();

    let rows = state
        .storage
        .list_connections(&tenant_id, &environment)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    let total = rows.len();
    let connections: Vec<ConnectionResponse> = rows.into_iter().map(row_to_response).collect();

    Ok(Json(ConnectionListResponse { connections, total }))
}

/// POST /connections
pub async fn create_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    headers: HeaderMap,
    Json(req): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<ConnectionResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();
    let limits = &tenant.plan_limits;

    enforce_production_guard(&headers, &environment)?;

    // Check connection count limit
    if !limits.is_unlimited(limits.max_connections) {
        let count = state
            .storage
            .count_connections(&tenant_id, &environment)
            .await
            .map_err(|e| ApiError::internal("database_error", e.to_string()))?;
        if count >= limits.max_connections {
            return Err(ApiError::limit_reached(
                "connections",
                count,
                limits.max_connections,
                &tenant.plan_name,
            ));
        }
    }

    validate_config(&req.connector_type, &req.config).map_err(|e| ApiError {
        error: "validation_error".to_string(),
        message: e,
        status: Some(StatusCode::BAD_REQUEST),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let row = state
        .storage
        .create_connection(
            &tenant_id,
            &environment,
            &req.name,
            &req.connector_type.to_string(),
            &req.auth_type.to_string(),
            req.config,
            req.credentials,
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    tracing::info!(
        connection_id = %row.id,
        connector_type = %row.connector_type,
        "Connection created"
    );

    Ok((StatusCode::CREATED, Json(row_to_response(row))))
}

/// GET /connections/:id
pub async fn get_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();

    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &environment, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
            status: Some(StatusCode::NOT_FOUND),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        })?;

    Ok(Json(row_to_response(row)))
}

/// PUT /connections/:id
pub async fn update_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnectionRequest>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    // Get existing to validate config against its connector_type
    let existing = state
        .storage
        .get_connection_in_environment(&tenant_id, &environment, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
            status: Some(StatusCode::NOT_FOUND),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        })?;

    let connector_type = parse_connector_type(&existing.connector_type)?;

    validate_config(&connector_type, &req.config).map_err(|e| ApiError {
        error: "validation_error".to_string(),
        message: e,
        status: Some(StatusCode::BAD_REQUEST),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let row = state
        .storage
        .update_connection(
            &tenant_id,
            &environment,
            id,
            &req.name,
            req.config,
            req.credentials,
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(row_to_response(row)))
}

/// DELETE /connections/:id
pub async fn delete_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    state
        .storage
        .delete_connection(&tenant_id, &environment, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
            status: Some(StatusCode::NOT_FOUND),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        })?;

    tracing::info!(connection_id = %id, "Connection deleted");

    Ok(StatusCode::NO_CONTENT)
}

/// POST /connections/:id/test
pub async fn test_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    let conn = state
        .storage
        .get_connection_in_environment(&tenant_id, &environment, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
            status: Some(StatusCode::NOT_FOUND),
            resource: None,
            current: None,
            limit: None,
            plan: None,
        })?;

    // Validate config shape as a basic "test"
    // Real connectivity testing will be added when agents support it
    let connector_type = parse_connector_type(&conn.connector_type)?;

    let (status, test_status, test_error) = match validate_config(&connector_type, &conn.config) {
        Ok(()) => ("active", "success", None),
        Err(e) => ("error", "failure", Some(e)),
    };

    state
        .storage
        .update_connection_test_status(
            &tenant_id,
            &environment,
            id,
            status,
            test_status,
            test_error.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    // Re-fetch to get updated fields
    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &environment, id)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(row_to_response(row)))
}

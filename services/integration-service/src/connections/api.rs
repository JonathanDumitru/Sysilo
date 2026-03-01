use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
#[path = "../connectors/specs.rs"]
mod specs;
use crate::connections::registry;
use crate::connections::{
    determine_next_status, has_credentials, sanitize_config_for_response,
    validate_and_normalize_credentials, AuthType, ConnectionLifecycleAction,
    ConnectionLifecycleStatus, ConnectorType,
};
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
    pub desired_status: Option<String>,
}

// === Helper to convert row to response ===

fn row_to_response(row: ConnectionRow) -> ConnectionResponse {
    ConnectionResponse {
        id: row.id,
        name: row.name,
        connector_type: row.connector_type,
        auth_type: row.auth_type,
        config: sanitize_config_for_response(&row.config),
        has_credentials: has_credentials(&row.credentials),
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

fn parse_auth_type(raw: &str) -> Result<AuthType, ApiError> {
    serde_json::from_value(serde_json::json!(raw)).map_err(|e| ApiError {
        error: "invalid_auth_type".to_string(),
        message: e.to_string(),
        status: Some(StatusCode::BAD_REQUEST),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })
}

fn production_request_error(error: &str, message: &str, status: StatusCode) -> ApiError {
    ApiError {
        error: error.to_string(),
        message: message.to_string(),
        status: Some(status),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    }
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
        return Err(production_request_error(
            "production_confirmation_required",
            "Production mutation requires explicit confirmation",
            StatusCode::FORBIDDEN,
        ));
    }

    let reason = headers
        .get("x-change-reason")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or("");
    if reason.is_empty() {
        return Err(production_request_error(
            "production_change_reason_required",
            "Production mutation requires a non-empty change reason",
            StatusCode::BAD_REQUEST,
        ));
    }

    Ok(())
}

fn normalize_credential_payload(
    auth_type: &AuthType,
    payload: serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let Some(obj) = payload.as_object() else {
        return Err(production_request_error(
            "validation_error",
            "credentials must be a JSON object",
            StatusCode::BAD_REQUEST,
        ));
    };

    if obj.is_empty() {
        return Ok(serde_json::json!({}));
    }

    validate_and_normalize_credentials(auth_type, &payload).map_err(|e| {
        production_request_error("validation_error", &e, StatusCode::BAD_REQUEST)
    })
}

async fn reset_to_draft(
    state: &Arc<AppState>,
    tenant_id: &str,
    environment: &str,
    id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE connections
        SET status = $2,
            last_tested_at = NULL,
            last_test_status = NULL,
            last_test_error = NULL,
            updated_at = NOW()
        WHERE id = $1
          AND tenant_id = $3::uuid
          AND config->>'_environment' = $4
        "#,
    )
    .bind(id)
    .bind(ConnectionLifecycleStatus::Draft.as_str())
    .bind(tenant_id)
    .bind(environment)
    .execute(state.storage.pool())
    .await
    .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(())
}

async fn set_status_only(
    state: &Arc<AppState>,
    tenant_id: &str,
    environment: &str,
    id: Uuid,
    status: ConnectionLifecycleStatus,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE connections
        SET status = $2,
            updated_at = NOW()
        WHERE id = $1
          AND tenant_id = $3::uuid
          AND config->>'_environment' = $4
        "#,
    )
    .bind(id)
    .bind(status.as_str())
    .bind(tenant_id)
    .bind(environment)
    .execute(state.storage.pool())
    .await
    .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(())
}

fn normalized_config_for_compare(config: &serde_json::Value) -> serde_json::Value {
    sanitize_config_for_response(config)
}

fn string_config_value(config: &serde_json::Value, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn ensure_connection_scope(
    row: &ConnectionRow,
    team_id: &str,
    environment: &str,
) -> Result<(), ApiError> {
    let row_environment = string_config_value(&row.config, "_environment");
    let row_team_id = string_config_value(&row.config, "_team_id");
    if row_environment.as_deref() != Some(environment) || row_team_id.as_deref() != Some(team_id) {
        return Err(production_request_error(
            "scope_mismatch",
            "Connection scope does not match request context",
            StatusCode::FORBIDDEN,
        ));
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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();

    let rows = state
        .storage
        .list_connections(&tenant_id, &team_id, &environment)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;
    for row in &rows {
        ensure_connection_scope(row, &team_id, &environment)?;
    }

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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();
    let limits = &tenant.plan_limits;

    enforce_production_guard(&headers, &environment)?;

    // Check connection count limit
    if !limits.is_unlimited(limits.max_connections) {
        let count = state
            .storage
            .count_connections(&tenant_id, &team_id, &environment)
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

    specs::validate_connector_spec(&req.connector_type, &req.auth_type, &req.config).map_err(|e| {
        production_request_error("validation_error", &e, StatusCode::BAD_REQUEST)
    })?;

    let credentials = normalize_credential_payload(&req.auth_type, req.credentials)?;

    let row = state
        .storage
        .create_connection(
            &tenant_id,
            &team_id,
            &environment,
            &req.name,
            &req.connector_type.to_string(),
            &req.auth_type.to_string(),
            req.config,
            credentials,
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    reset_to_draft(&state, &tenant_id, &environment, row.id).await?;

    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, row.id)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;
    ensure_connection_scope(&row, &team_id, &environment)?;

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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();

    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, id)
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
    ensure_connection_scope(&row, &team_id, &environment)?;

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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    // Get existing to validate config against its connector_type
    let existing = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, id)
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
    ensure_connection_scope(&existing, &team_id, &environment)?;

    let connector_type = parse_connector_type(&existing.connector_type)?;
    let auth_type = parse_auth_type(&existing.auth_type)?;

    specs::validate_connector_spec(&connector_type, &auth_type, &req.config).map_err(|e| {
        production_request_error("validation_error", &e, StatusCode::BAD_REQUEST)
    })?;

    let credentials = match req.credentials {
        Some(payload) => Some(normalize_credential_payload(&auth_type, payload)?),
        None => None,
    };

    let config_changed = normalized_config_for_compare(&existing.config) != req.config;
    let credentials_replaced = credentials.is_some();
    let mut requires_retest = config_changed || credentials_replaced;

    if req.desired_status.as_deref() == Some("active") && requires_retest {
        return Err(production_request_error(
            "activation_requires_retest",
            "Configuration or credential updates require a fresh successful test before activation",
            StatusCode::CONFLICT,
        ));
    }

    let row = state
        .storage
        .update_connection(
            &tenant_id,
            &team_id,
            &environment,
            id,
            &req.name,
            req.config,
            credentials,
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    if req.desired_status.as_deref() == Some("active") {
        let next = determine_next_status(
            &row.status,
            row.last_test_status.as_deref(),
            ConnectionLifecycleAction::Activate,
        )
        .map_err(|e| production_request_error("activation_blocked", &e, StatusCode::CONFLICT))?;
        set_status_only(&state, &tenant_id, &environment, id, next).await?;
        requires_retest = false;
    } else if let Some(other) = req.desired_status.as_deref() {
        return Err(production_request_error(
            "invalid_status_transition",
            &format!("Unsupported desired_status '{}'. Only 'active' is allowed.", other),
            StatusCode::BAD_REQUEST,
        ));
    }

    if requires_retest {
        reset_to_draft(&state, &tenant_id, &environment, id).await?;
    }

    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, id)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;
    ensure_connection_scope(&row, &team_id, &environment)?;

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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    state
        .storage
        .delete_connection(&tenant_id, &team_id, &environment, id)
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
    let team_id = tenant.team_id.to_string();
    let environment = tenant.environment.clone();

    enforce_production_guard(&headers, &environment)?;

    let conn = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, id)
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
    ensure_connection_scope(&conn, &team_id, &environment)?;

    let connector_type = parse_connector_type(&conn.connector_type)?;
    let auth_type = parse_auth_type(&conn.auth_type)?;

    let check = registry::health_check(&connector_type, &auth_type, &conn.config, &conn.credentials);
    let (next_status, test_status, test_error) = if check.healthy {
        let status = determine_next_status(
            &conn.status,
            conn.last_test_status.as_deref(),
            ConnectionLifecycleAction::TestSuccess,
        )
        .map_err(|e| production_request_error("invalid_status_transition", &e, StatusCode::CONFLICT))?;
        (status, "success", None)
    } else {
        let error_message = check.message.clone();
        let status = determine_next_status(
            &conn.status,
            conn.last_test_status.as_deref(),
            ConnectionLifecycleAction::TestFailure,
        )
        .map_err(|e| production_request_error("invalid_status_transition", &e, StatusCode::CONFLICT))?;
        (status, "failure", Some(error_message))
    };

    state
        .storage
        .update_connection_test_status(
            &tenant_id,
            &team_id,
            &environment,
            id,
            next_status.as_str(),
            test_status,
            test_error.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    if check.healthy {
        tracing::info!(
            connection_id = %id,
            connector_type = %connector_type,
            latency_ms = check.latency_ms,
            "connection_health_check_success"
        );
    } else {
        tracing::warn!(
            connection_id = %id,
            connector_type = %connector_type,
            latency_ms = check.latency_ms,
            error = %check.message,
            "connection_health_check_failure"
        );
    }

    // Re-fetch to get updated fields
    let row = state
        .storage
        .get_connection_in_environment(&tenant_id, &team_id, &environment, id)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;
    ensure_connection_scope(&row, &team_id, &environment)?;

    Ok(Json(row_to_response(row)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scoped_row(team_id: &str, environment: &str) -> ConnectionRow {
        ConnectionRow {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "example".to_string(),
            connector_type: "slack".to_string(),
            auth_type: "oauth2".to_string(),
            config: serde_json::json!({
                "_team_id": team_id,
                "_environment": environment
            }),
            credentials: serde_json::json!({}),
            status: "draft".to_string(),
            last_tested_at: None,
            last_test_status: None,
            last_test_error: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn cross_scope_denial_rejects_mismatched_environment() {
        let row = scoped_row("team-alpha", "dev");
        let err = ensure_connection_scope(&row, "team-alpha", "prod").unwrap_err();
        assert_eq!(err.error, "scope_mismatch");
    }

    #[test]
    fn cross_scope_denial_rejects_mismatched_team() {
        let row = scoped_row("team-alpha", "dev");
        let err = ensure_connection_scope(&row, "team-beta", "dev").unwrap_err();
        assert_eq!(err.error, "scope_mismatch");
    }
}

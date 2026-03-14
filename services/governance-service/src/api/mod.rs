use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::AppState;
use crate::standards::{CreateStandardRequest, UpdateStandardRequest};
use crate::policies::{CreatePolicyRequest, UpdatePolicyRequest, EvaluatePoliciesRequest};
use crate::approvals::{CreateWorkflowRequest, UpdateWorkflowRequest, CreateApprovalRequestInput, DecideRequest};
use crate::audit::AuditQueryParams;
use crate::federated::{
    CreateDomainRequest, UpdateDomainRequest,
    CreateDomainPolicyRequest, UpdateDomainPolicyRequest,
    FederatedEvaluateRequest, InheritanceQueryParams, HealthTrendsParams,
};

// ============================================================================
// Common Types
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// Extract tenant_id from headers
fn get_tenant_id(headers: &axum::http::HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

fn get_user_id(headers: &axum::http::HeaderMap) -> Option<Uuid> {
    headers
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

// ============================================================================
// Health Handlers
// ============================================================================

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "governance-service",
        "timestamp": Utc::now()
    }))
}

pub async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.audit.health_check().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ready"}))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"status": "not ready"}))),
    }
}

// ============================================================================
// Standards Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListStandardsParams {
    pub category: Option<String>,
    pub status: Option<String>,
}

pub async fn list_standards(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListStandardsParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.standards.list_standards(tenant_id, params.category, params.status).await {
        Ok(standards) => (StatusCode::OK, Json(ApiResponse::success(standards))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_standard(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.standards.get_standard(tenant_id, id).await {
        Ok(Some(standard)) => (StatusCode::OK, Json(ApiResponse::success(standard))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Standard not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_standard(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateStandardRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.standards.create_standard(tenant_id, req, user_id).await {
        Ok(standard) => (StatusCode::CREATED, Json(ApiResponse::success(standard))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_standard(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStandardRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.standards.update_standard(tenant_id, id, req, user_id).await {
        Ok(Some(standard)) => (StatusCode::OK, Json(ApiResponse::success(standard))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Standard not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn delete_standard(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.standards.delete_standard(tenant_id, id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"deleted": true})))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Standard not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Policies Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPoliciesParams {
    pub scope: Option<String>,
    pub enabled_only: Option<bool>,
}

pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListPoliciesParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.list_policies(tenant_id, params.scope, params.enabled_only.unwrap_or(false)).await {
        Ok(policies) => (StatusCode::OK, Json(ApiResponse::success(policies))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.get_policy(tenant_id, id).await {
        Ok(Some(policy)) => (StatusCode::OK, Json(ApiResponse::success(policy))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Policy not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreatePolicyRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.create_policy(tenant_id, req).await {
        Ok(policy) => (StatusCode::CREATED, Json(ApiResponse::success(policy))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_policy(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePolicyRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.update_policy(tenant_id, id, req).await {
        Ok(Some(policy)) => (StatusCode::OK, Json(ApiResponse::success(policy))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Policy not found"))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn delete_policy(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.delete_policy(tenant_id, id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"deleted": true})))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Policy not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn evaluate_policies(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<EvaluatePoliciesRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.policies.evaluate_policies(tenant_id, req).await {
        Ok(results) => (StatusCode::OK, Json(ApiResponse::success(results))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListViolationsParams {
    pub status: Option<String>,
    pub policy_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_violations(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListViolationsParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    match state.policies.list_violations(tenant_id, params.status, params.policy_id, limit, offset).await {
        Ok((violations, total)) => (StatusCode::OK, Json(ApiResponse::success(PaginatedResponse {
            items: violations,
            total,
            limit,
            offset,
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ResolveViolationRequest {
    pub resolution_note: Option<String>,
}

pub async fn resolve_violation(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<ResolveViolationRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.policies.resolve_violation(tenant_id, id, user_id, req.resolution_note).await {
        Ok(Some(violation)) => (StatusCode::OK, Json(ApiResponse::success(violation))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Violation not found or already resolved"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Approvals Handlers
// ============================================================================

pub async fn list_workflows(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.approvals.list_workflows(tenant_id).await {
        Ok(workflows) => (StatusCode::OK, Json(ApiResponse::success(workflows))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_workflow(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.approvals.get_workflow(tenant_id, id).await {
        Ok(Some(workflow)) => (StatusCode::OK, Json(ApiResponse::success(workflow))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Workflow not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.approvals.create_workflow(tenant_id, req, user_id).await {
        Ok(workflow) => (StatusCode::CREATED, Json(ApiResponse::success(workflow))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_workflow(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.approvals.update_workflow(tenant_id, id, req).await {
        Ok(Some(workflow)) => (StatusCode::OK, Json(ApiResponse::success(workflow))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Workflow not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsParams {
    pub status: Option<String>,
    pub approver_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_requests(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListRequestsParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let requester_id = get_user_id(&headers);
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    match state.approvals.list_requests(
        tenant_id,
        params.status,
        requester_id,
        params.approver_id,
        limit,
        offset,
    ).await {
        Ok((requests, total)) => (StatusCode::OK, Json(ApiResponse::success(PaginatedResponse {
            items: requests,
            total,
            limit,
            offset,
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_request(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.approvals.get_request(tenant_id, id).await {
        Ok(Some(request)) => (StatusCode::OK, Json(ApiResponse::success(request))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Request not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_request(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateApprovalRequestInput>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let requester_id = match get_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error("User ID required"))).into_response(),
    };

    match state.approvals.create_request(tenant_id, requester_id, req).await {
        Ok(request) => (StatusCode::CREATED, Json(ApiResponse::success(request))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn decide_request(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<DecideRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let approver_id = match get_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error("User ID required"))).into_response(),
    };

    match state.approvals.decide(tenant_id, id, approver_id, req).await {
        Ok(request) => (StatusCode::OK, Json(ApiResponse::success(request))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Audit Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AuditQueryApiParams {
    pub actor_id: Option<Uuid>,
    pub actor_type: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn query_audit_log(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<AuditQueryApiParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let query_params = AuditQueryParams {
        actor_id: params.actor_id,
        actor_type: params.actor_type,
        action: params.action,
        resource_type: params.resource_type,
        resource_id: params.resource_id,
        start_time: params.start_time,
        end_time: params.end_time,
        limit: params.limit,
        offset: params.offset,
    };

    let limit = query_params.limit.unwrap_or(100);
    let offset = query_params.offset.unwrap_or(0);

    match state.audit.query(tenant_id, query_params).await {
        Ok((entries, total)) => (StatusCode::OK, Json(ApiResponse::success(PaginatedResponse {
            items: entries,
            total,
            limit,
            offset,
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportAuditParams {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

pub async fn export_audit_log(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ExportAuditParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.audit.export(tenant_id, params.start_time, params.end_time).await {
        Ok(entries) => (StatusCode::OK, Json(ApiResponse::success(entries))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Compliance Handlers
// ============================================================================

pub async fn list_frameworks(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.compliance.list_frameworks().await {
        Ok(frameworks) => (StatusCode::OK, Json(ApiResponse::success(frameworks))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ComplianceStatusParams {
    pub framework_id: Uuid,
}

pub async fn get_compliance_status(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ComplianceStatusParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.compliance.get_status(tenant_id, params.framework_id).await {
        Ok(statuses) => (StatusCode::OK, Json(ApiResponse::success(statuses))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RunAssessmentRequest {
    pub framework_id: Uuid,
}

pub async fn run_assessment(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RunAssessmentRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.compliance.run_assessment(tenant_id, req.framework_id).await {
        Ok(result) => (StatusCode::OK, Json(ApiResponse::success(result))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerateReportParams {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

pub async fn generate_report(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(framework): Path<String>,
    Query(params): Query<GenerateReportParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    // Get framework by name or ID
    let framework_result = if let Ok(framework_id) = Uuid::parse_str(&framework) {
        state.compliance.get_framework(framework_id).await
    } else {
        state.compliance.get_framework_by_name(&framework).await
    };

    let framework = match framework_result {
        Ok(Some(f)) => f,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Framework not found"))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    };

    let start_time = params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(365));
    let end_time = params.end_time.unwrap_or_else(Utc::now);

    match state.compliance.generate_report(tenant_id, framework.id, start_time, end_time).await {
        Ok(report) => (StatusCode::OK, Json(ApiResponse::success(report))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

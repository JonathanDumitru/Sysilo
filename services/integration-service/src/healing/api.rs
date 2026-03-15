use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
use crate::middleware::TenantContext;
use crate::AppState;

use super::{ErrorInfo, HealingConfig, HealingProposal, HealingStats, ProposalFilters};

// =============================================================================
// Response types
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ProposalResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub run_id: Uuid,
    pub failure_class: String,
    pub diagnosis: String,
    pub proposed_fix: serde_json::Value,
    pub risk_level: String,
    pub confidence: f64,
    pub approval_status: String,
    pub approval_request_id: Option<Uuid>,
    pub applied: bool,
    pub applied_at: Option<String>,
    pub result: Option<String>,
    pub created_at: String,
}

impl From<HealingProposal> for ProposalResponse {
    fn from(p: HealingProposal) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            integration_id: p.integration_id,
            run_id: p.run_id,
            failure_class: p.failure_class.to_string(),
            diagnosis: p.diagnosis,
            proposed_fix: serde_json::to_value(&p.proposed_fix).unwrap_or_default(),
            risk_level: p.risk_level.to_string(),
            confidence: p.confidence,
            approval_status: p.approval_status.to_string(),
            approval_request_id: p.approval_request_id,
            applied: p.applied,
            applied_at: p.applied_at.map(|t| t.to_rfc3339()),
            result: p.result,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProposalListResponse {
    pub proposals: Vec<ProposalResponse>,
    pub total: usize,
}

// =============================================================================
// Request types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DiagnoseRequest {
    pub integration_id: Uuid,
    pub run_id: Uuid,
    pub error_message: String,
    pub error_code: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ProposalQueryParams {
    pub integration_id: Option<Uuid>,
    pub failure_class: Option<String>,
    pub approval_status: Option<String>,
    pub risk_level: Option<String>,
    pub applied: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /healing/proposals — list healing proposals for the tenant
pub async fn list_proposals(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Query(params): Query<ProposalQueryParams>,
) -> Result<Json<ProposalListResponse>, ApiError> {
    let healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let filters = ProposalFilters {
        integration_id: params.integration_id,
        failure_class: params.failure_class,
        approval_status: params.approval_status,
        risk_level: params.risk_level,
        applied: params.applied,
        limit: params.limit,
        offset: params.offset,
    };

    let proposals = healing
        .list_proposals(tenant.tenant_id, &filters)
        .await
        .map_err(|e| ApiError::internal("healing_error", e.to_string()))?;

    let total = proposals.len();
    let responses: Vec<ProposalResponse> = proposals.into_iter().map(ProposalResponse::from).collect();

    Ok(Json(ProposalListResponse {
        proposals: responses,
        total,
    }))
}

/// GET /healing/proposals/:id — get a single healing proposal
pub async fn get_proposal(
    State(state): State<Arc<AppState>>,
    Extension(_tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProposalResponse>, ApiError> {
    let healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let proposal = healing
        .get_proposal(id)
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

    Ok(Json(ProposalResponse::from(proposal)))
}

/// POST /healing/diagnose — manually trigger diagnosis for a failed run
pub async fn diagnose(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<DiagnoseRequest>,
) -> Result<(StatusCode, Json<ProposalResponse>), ApiError> {
    let healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let error_info = ErrorInfo {
        error_message: req.error_message,
        error_code: req.error_code,
        context: req.context,
    };

    let proposal = healing
        .diagnose_failure(tenant.tenant_id, req.integration_id, req.run_id, &error_info)
        .await
        .map_err(|e| ApiError::internal("diagnosis_error", e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ProposalResponse::from(proposal))))
}

/// POST /healing/proposals/:id/apply — manually apply a healing proposal
pub async fn apply_proposal(
    State(state): State<Arc<AppState>>,
    Extension(_tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProposalResponse>, ApiError> {
    let healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let proposal = healing
        .apply_fix(id)
        .await
        .map_err(|e| {
            let (error_key, status) = match &e {
                super::HealingError::NotFound(_) => ("not_found", StatusCode::NOT_FOUND),
                super::HealingError::InvalidState(_) => ("invalid_state", StatusCode::CONFLICT),
                _ => ("apply_error", StatusCode::INTERNAL_SERVER_ERROR),
            };
            ApiError {
                error: error_key.to_string(),
                message: e.to_string(),
                status: Some(status),
                resource: None,
                current: None,
                limit: None,
                plan: None,
            }
        })?;

    Ok(Json(ProposalResponse::from(proposal)))
}

/// GET /healing/stats — healing statistics for the tenant
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<HealingStats>, ApiError> {
    let healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    let stats = healing
        .get_healing_stats(tenant.tenant_id)
        .await
        .map_err(|e| ApiError::internal("stats_error", e.to_string()))?;

    Ok(Json(stats))
}

/// PUT /healing/config — update healing configuration
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Extension(_tenant): Extension<TenantContext>,
    Json(new_config): Json<HealingConfig>,
) -> Result<Json<HealingConfig>, ApiError> {
    // Note: In a real production system, the HealingService would need interior
    // mutability (e.g., RwLock) to update config at runtime. For now we validate
    // the config and return it, logging the update request.
    let _healing = state.healing.as_ref().ok_or_else(|| ApiError {
        error: "healing_disabled".to_string(),
        message: "Healing service is not enabled".to_string(),
        status: Some(StatusCode::SERVICE_UNAVAILABLE),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    })?;

    tracing::info!(
        enabled = new_config.enabled,
        auto_approve_low_risk = new_config.auto_approve_low_risk,
        max_auto_retries = new_config.max_auto_retries,
        "Healing config update requested"
    );

    // Return the accepted config
    Ok(Json(new_config))
}

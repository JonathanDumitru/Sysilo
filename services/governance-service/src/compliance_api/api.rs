use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use super::{GenerateReportRequest, PolicyDecisionRequest, RecordRegulatoryChangeRequest};

// =========================================================================
// Governance API — Policy Decision Endpoint
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct TenantHeader {
    pub tenant_id: Uuid,
}

pub async fn evaluate_policy(
    State(state): State<Arc<AppState>>,
    Query(tenant): Query<TenantHeader>,
    Json(req): Json<PolicyDecisionRequest>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.evaluate_policy_decision(tenant.tenant_id, req).await {
        Ok(decision) => Json(serde_json::json!(decision)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// =========================================================================
// Compliance Scoring
// =========================================================================

pub async fn get_compliance_scores(
    State(state): State<Arc<AppState>>,
    Query(tenant): Query<TenantHeader>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.get_compliance_scores(tenant.tenant_id).await {
        Ok(scores) => Json(serde_json::json!({"scores": scores})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScoreRequest {
    pub tenant_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
}

pub async fn calculate_score(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScoreRequest>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.calculate_compliance_score(req.tenant_id, &req.entity_type, req.entity_id).await {
        Ok(score) => Json(serde_json::json!(score)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// =========================================================================
// Compliance Reports
// =========================================================================

pub async fn generate_report(
    State(state): State<Arc<AppState>>,
    Query(tenant): Query<TenantHeader>,
    Json(req): Json<GenerateReportRequest>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.generate_compliance_report(tenant.tenant_id, req, None).await {
        Ok(report) => (StatusCode::CREATED, Json(serde_json::json!(report))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ReportListQuery {
    pub tenant_id: Uuid,
    pub framework: Option<String>,
}

pub async fn list_reports(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ReportListQuery>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.list_compliance_reports(params.tenant_id, params.framework.as_deref()).await {
        Ok(reports) => Json(serde_json::json!({"reports": reports})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// =========================================================================
// Regulatory Change Tracking
// =========================================================================

pub async fn record_regulatory_change(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordRegulatoryChangeRequest>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.record_regulatory_change(req).await {
        Ok(change) => (StatusCode::CREATED, Json(serde_json::json!(change))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RegulatoryChangesQuery {
    pub status: Option<String>,
}

pub async fn list_regulatory_changes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RegulatoryChangesQuery>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.list_regulatory_changes(params.status.as_deref()).await {
        Ok(changes) => Json(serde_json::json!({"changes": changes})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn approve_regulatory_change(
    State(state): State<Arc<AppState>>,
    Path(change_id): Path<Uuid>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    // In production, reviewed_by comes from auth context
    let reviewed_by = Uuid::new_v4();

    match svc.approve_regulatory_change(change_id, reviewed_by).await {
        Ok(change) => Json(serde_json::json!(change)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// =========================================================================
// Policy Decision History
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct DecisionHistoryQuery {
    pub tenant_id: Uuid,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_decision_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DecisionHistoryQuery>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    match svc.get_decision_history(params.tenant_id, limit, offset).await {
        Ok(decisions) => Json(serde_json::json!({"decisions": decisions})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_decision_analytics(
    State(state): State<Arc<AppState>>,
    Query(tenant): Query<TenantHeader>,
) -> impl IntoResponse {
    let svc = match &state.compliance_api {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Compliance API not available"}))).into_response(),
    };

    match svc.get_decision_analytics(tenant.tenant_id).await {
        Ok(analytics) => Json(analytics).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

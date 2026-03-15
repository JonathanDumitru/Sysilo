use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::AppState;
use crate::immune_system::{
    CreateDangerSignalRequest, DangerSignalFilters, RecordRemediationRequest,
};

// ============================================================================
// Helper: extract tenant / user from headers (mirrors crate::api pattern)
// ============================================================================

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
// Danger Signal Handlers
// ============================================================================

/// POST /immune/signals - Report a new danger signal
pub async fn report_danger_signal(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateDangerSignalRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.report_danger_signal(tenant_id, req).await {
        Ok(signal) => (StatusCode::CREATED, Json(ApiResponse::success(signal))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// GET /immune/signals - List danger signals with optional filters
#[derive(Debug, Deserialize)]
pub struct ListSignalsParams {
    pub signal_type: Option<String>,
    pub severity: Option<String>,
    pub acknowledged: Option<bool>,
    pub resolved: Option<bool>,
    pub integration_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_danger_signals(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListSignalsParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    let filters = DangerSignalFilters {
        signal_type: params.signal_type,
        severity: params.severity,
        acknowledged: params.acknowledged,
        resolved: params.resolved,
        integration_id: params.integration_id,
        limit: params.limit,
        offset: params.offset,
    };

    match state.immune_system.list_danger_signals(tenant_id, filters).await {
        Ok(signals) => (StatusCode::OK, Json(ApiResponse::success(signals))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// POST /immune/signals/:id/acknowledge - Acknowledge a danger signal
pub async fn acknowledge_signal(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    let user_id = match get_user_id(&headers) {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("User ID required")),
            )
                .into_response()
        }
    };

    match state.immune_system.acknowledge_signal(tenant_id, id, user_id).await {
        Ok(Some(signal)) => (StatusCode::OK, Json(ApiResponse::success(signal))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Signal not found or already acknowledged")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// POST /immune/signals/:id/resolve - Resolve a danger signal
pub async fn resolve_signal(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.resolve_signal(tenant_id, id).await {
        Ok(Some(signal)) => (StatusCode::OK, Json(ApiResponse::success(signal))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Signal not found or already resolved")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Correlation Handlers (T-Cell Coordinator)
// ============================================================================

/// POST /immune/correlate - Run T-cell correlation on recent unresolved signals
pub async fn correlate_signals(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.correlate_signals(tenant_id).await {
        Ok(result) => (StatusCode::OK, Json(ApiResponse::success(result))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Auto-Remediation Handlers
// ============================================================================

/// POST /immune/signals/:id/remediate - Attempt auto-remediation for a signal
pub async fn attempt_auto_remediation(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.attempt_auto_remediation(tenant_id, id).await {
        Ok(attempt) => (StatusCode::OK, Json(ApiResponse::success(attempt))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Immune Memory Handlers (B-Cell Memory)
// ============================================================================

/// POST /immune/memory - Record a remediation outcome
pub async fn record_remediation(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RecordRemediationRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.record_remediation(tenant_id, req).await {
        Ok(memory) => (StatusCode::OK, Json(ApiResponse::success(memory))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// GET /immune/memory - List immune memories for a tenant
pub async fn list_immune_memories(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.list_immune_memories(tenant_id).await {
        Ok(memories) => (StatusCode::OK, Json(ApiResponse::success(memories))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Vaccination Handlers (Cross-Tenant Protection)
// ============================================================================

/// POST /immune/vaccinate - Distribute a vaccination from a proven countermeasure
#[derive(Debug, Deserialize)]
pub struct VaccinateRequest {
    pub failure_signature: String,
}

pub async fn distribute_vaccination(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<VaccinateRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state
        .immune_system
        .distribute_vaccination(tenant_id, req.failure_signature)
        .await
    {
        Ok(record) => (StatusCode::CREATED, Json(ApiResponse::success(record))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// GET /immune/vaccinations - Get vaccination history
pub async fn get_vaccination_history(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.immune_system.get_vaccination_history().await {
        Ok(records) => (StatusCode::OK, Json(ApiResponse::success(records))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Status & Scoring Handlers
// ============================================================================

/// GET /immune/status - Get overall immune system status for a tenant
pub async fn get_immune_status(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.get_immune_status(tenant_id).await {
        Ok(status) => (StatusCode::OK, Json(ApiResponse::success(status))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

/// GET /immune/resilience - Get composite resilience score for a tenant
pub async fn get_system_resilience_score(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => {
            return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response()
        }
    };

    match state.immune_system.get_system_resilience_score(tenant_id).await {
        Ok(score) => (StatusCode::OK, Json(ApiResponse::success(score))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )
            .into_response(),
    }
}

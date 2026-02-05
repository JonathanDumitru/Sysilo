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
use crate::metrics::{MetricInput, TimeBucket};
use crate::alerts::{CreateAlertRuleRequest, UpdateAlertRuleRequest};
use crate::incidents::{CreateIncidentRequest, UpdateIncidentRequest, AddIncidentEventRequest};
use crate::notifications::{CreateChannelRequest, UpdateChannelRequest};

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

// Extract tenant_id from headers (simplified - in production would use JWT)
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
        "service": "ops-service",
        "timestamp": Utc::now()
    }))
}

pub async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.metrics.health_check().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ready"}))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"status": "not ready"}))),
    }
}

// ============================================================================
// Metrics Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MetricsQueryParams {
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub metric_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub async fn query_metrics(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<MetricsQueryParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let limit = params.limit.unwrap_or(1000).min(10000);

    match state.metrics.query_metrics(
        tenant_id,
        params.resource_type,
        params.resource_id,
        params.metric_name,
        params.start_time,
        params.end_time,
        limit,
    ).await {
        Ok(metrics) => (StatusCode::OK, Json(ApiResponse::success(metrics))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct IngestMetricsRequest {
    pub metrics: Vec<MetricInput>,
}

pub async fn ingest_metrics(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<IngestMetricsRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.metrics.ingest_metrics(tenant_id, req.metrics).await {
        Ok(count) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
            "ingested": count
        })))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AggregationsQueryParams {
    pub metric_name: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub bucket: Option<String>,
}

pub async fn get_aggregations(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<AggregationsQueryParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let bucket = match params.bucket.as_deref() {
        Some("minute") => TimeBucket::Minute,
        Some("day") => TimeBucket::Day,
        _ => TimeBucket::Hour,
    };

    match state.metrics.get_aggregations(
        tenant_id,
        params.metric_name,
        params.resource_type,
        params.resource_id,
        params.start_time,
        params.end_time,
        bucket,
    ).await {
        Ok(aggs) => (StatusCode::OK, Json(ApiResponse::success(aggs))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Alert Rules Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListRulesParams {
    pub enabled_only: Option<bool>,
}

pub async fn list_alert_rules(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListRulesParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.list_rules(tenant_id, params.enabled_only.unwrap_or(false)).await {
        Ok(rules) => (StatusCode::OK, Json(ApiResponse::success(rules))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_alert_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.get_rule(tenant_id, id).await {
        Ok(Some(rule)) => (StatusCode::OK, Json(ApiResponse::success(rule))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Rule not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_alert_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAlertRuleRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.create_rule(tenant_id, req).await {
        Ok(rule) => (StatusCode::CREATED, Json(ApiResponse::success(rule))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_alert_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAlertRuleRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.update_rule(tenant_id, id, req).await {
        Ok(Some(rule)) => (StatusCode::OK, Json(ApiResponse::success(rule))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Rule not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn delete_alert_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.delete_rule(tenant_id, id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"deleted": true})))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Rule not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Alert Instances Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListInstancesParams {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_alert_instances(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListInstancesParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let limit = params.limit.unwrap_or(100).min(1000);

    match state.alerts.list_instances(tenant_id, params.status, limit).await {
        Ok(instances) => (StatusCode::OK, Json(ApiResponse::success(instances))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn acknowledge_alert(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = match get_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error("User ID required"))).into_response(),
    };

    match state.alerts.acknowledge_alert(tenant_id, id, user_id).await {
        Ok(Some(instance)) => (StatusCode::OK, Json(ApiResponse::success(instance))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Alert not found or already resolved"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn resolve_alert(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.alerts.resolve_alert(tenant_id, id).await {
        Ok(Some(instance)) => (StatusCode::OK, Json(ApiResponse::success(instance))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Alert not found or already resolved"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Incidents Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListIncidentsParams {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_incidents(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListIncidentsParams>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    match state.incidents.list_incidents(tenant_id, params.status, params.severity, limit, offset).await {
        Ok((incidents, total)) => (StatusCode::OK, Json(ApiResponse::success(PaginatedResponse {
            items: incidents,
            total,
            limit,
            offset,
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_incident(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.incidents.get_incident(tenant_id, id).await {
        Ok(Some(incident)) => (StatusCode::OK, Json(ApiResponse::success(incident))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Incident not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_incident(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.incidents.create_incident(tenant_id, req, user_id).await {
        Ok(incident) => (StatusCode::CREATED, Json(ApiResponse::success(incident))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_incident(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateIncidentRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.incidents.update_incident(tenant_id, id, req, user_id).await {
        Ok(Some(incident)) => (StatusCode::OK, Json(ApiResponse::success(incident))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Incident not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_incident_events(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let _tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.incidents.list_events(id).await {
        Ok(events) => (StatusCode::OK, Json(ApiResponse::success(events))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn add_incident_event(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<AddIncidentEventRequest>,
) -> impl IntoResponse {
    let _tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.incidents.add_event(id, req.event_type, req.content, req.metadata, user_id).await {
        Ok(event) => (StatusCode::CREATED, Json(ApiResponse::success(event))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ResolveIncidentRequest {
    pub resolution_note: Option<String>,
}

pub async fn resolve_incident(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<ResolveIncidentRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    let user_id = get_user_id(&headers);

    match state.incidents.resolve_incident(tenant_id, id, req.resolution_note, user_id).await {
        Ok(Some(incident)) => (StatusCode::OK, Json(ApiResponse::success(incident))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Incident not found or already resolved"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

// ============================================================================
// Notification Channels Handlers
// ============================================================================

pub async fn list_channels(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.list_channels(tenant_id).await {
        Ok(channels) => (StatusCode::OK, Json(ApiResponse::success(channels))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn get_channel(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.get_channel(tenant_id, id).await {
        Ok(Some(channel)) => (StatusCode::OK, Json(ApiResponse::success(channel))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Channel not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.create_channel(tenant_id, req).await {
        Ok(channel) => (StatusCode::CREATED, Json(ApiResponse::success(channel))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn update_channel(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.update_channel(tenant_id, id, req).await {
        Ok(Some(channel)) => (StatusCode::OK, Json(ApiResponse::success(channel))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Channel not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn delete_channel(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.delete_channel(tenant_id, id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"deleted": true})))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Channel not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

pub async fn test_channel(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match get_tenant_id(&headers) {
        Ok(id) => id,
        Err(status) => return (status, Json(ApiResponse::<()>::error("Unauthorized"))).into_response(),
    };

    match state.notifications.test_channel(tenant_id, id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"sent": true})))).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error("Channel not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(e.to_string()))).into_response(),
    }
}

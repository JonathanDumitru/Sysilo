use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use super::{CreateTemplateRequest, DeployTemplateRequest};

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub vertical: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTemplatesQuery>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    match templates_service.list_templates(
        params.vertical.as_deref(),
        params.status.as_deref(),
        params.search.as_deref(),
        limit,
        offset,
    ).await {
        Ok((templates, total)) => Json(serde_json::json!({
            "templates": templates,
            "total": total,
            "limit": limit,
            "offset": offset,
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.get_template(id).await {
        Ok(Some(template)) => Json(serde_json::json!(template)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Template not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTemplateRequest>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.create_template(req).await {
        Ok(template) => (StatusCode::CREATED, Json(serde_json::json!(template))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn publish_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.publish_template(id).await {
        Ok(template) => Json(serde_json::json!(template)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn deploy_template(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeployTemplateRequest>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    // In production, tenant_id and deployed_by come from auth context
    let tenant_id = Uuid::new_v4();
    let deployed_by = Uuid::new_v4();

    match templates_service.deploy_template(tenant_id, deployed_by, req).await {
        Ok(deployment) => (StatusCode::CREATED, Json(serde_json::json!(deployment))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_deployment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.get_deployment(id).await {
        Ok(Some(deployment)) => Json(serde_json::json!(deployment)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Deployment not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

pub async fn list_deployments(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TenantQuery>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.list_deployments(params.tenant_id).await {
        Ok(deployments) => Json(serde_json::json!({"deployments": deployments})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn rollback_deployment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.rollback_deployment(id).await {
        Ok(deployment) => Json(serde_json::json!(deployment)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_featured_templates(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.get_featured_templates(10).await {
        Ok(templates) => Json(serde_json::json!({"featured": templates})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_templates_by_vertical(
    State(state): State<Arc<AppState>>,
    Path(vertical): Path<String>,
) -> impl IntoResponse {
    let templates_service = match &state.templates {
        Some(svc) => svc,
        None => return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Templates service not available"}))).into_response(),
    };

    match templates_service.get_templates_by_vertical(&vertical).await {
        Ok(templates) => Json(serde_json::json!({"templates": templates, "vertical": vertical})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

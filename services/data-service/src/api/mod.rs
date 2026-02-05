use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::catalog::{Entity, EntityType, Schema};
use crate::lineage::{LineageEdge, LineageGraph};
use crate::quality::{QualityRule, QualityScore, QualityIssue};

// ============================================================================
// Health Endpoints
// ============================================================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "data-service".to_string(),
    })
}

pub async fn ready(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connectivity
    if state.catalog.health_check().await.is_ok() {
        Ok(Json(HealthResponse {
            status: "ready".to_string(),
            service: "data-service".to_string(),
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// ============================================================================
// Catalog Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct ListEntitiesQuery {
    pub tenant_id: Uuid,
    pub entity_type: Option<EntityType>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct ListEntitiesResponse {
    pub entities: Vec<Entity>,
    pub total: i64,
}

pub async fn list_entities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Json<ListEntitiesResponse>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match state.catalog.list_entities(query.tenant_id, query.entity_type, limit, offset).await {
        Ok((entities, total)) => Ok(Json(ListEntitiesResponse { entities, total })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CreateEntityRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub entity_type: EntityType,
    pub source_system: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn create_entity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<Json<Entity>, StatusCode> {
    match state.catalog.create_entity(
        req.tenant_id,
        req.name,
        req.entity_type,
        req.source_system,
        req.description,
        req.metadata,
    ).await {
        Ok(entity) => Ok(Json(entity)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct GetEntityQuery {
    pub tenant_id: Uuid,
}

pub async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Entity>, StatusCode> {
    match state.catalog.get_entity(query.tenant_id, id).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_entity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<StatusCode, StatusCode> {
    match state.catalog.delete_entity(query.tenant_id, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_entity_schema(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Schema>, StatusCode> {
    match state.catalog.get_entity_schema(query.tenant_id, id).await {
        Ok(Some(schema)) => Ok(Json(schema)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Lineage Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct GetLineageQuery {
    pub tenant_id: Uuid,
    pub depth: Option<i32>,
    pub direction: Option<String>,
}

pub async fn get_lineage(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<GetLineageQuery>,
) -> Result<Json<LineageGraph>, StatusCode> {
    let depth = query.depth.unwrap_or(3);
    let direction = query.direction.as_deref().unwrap_or("both");

    match state.lineage.get_lineage(query.tenant_id, entity_id, depth, direction).await {
        Ok(graph) => Ok(Json(graph)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct AddLineageEdgeRequest {
    pub tenant_id: Uuid,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub transformation_type: String,
    pub transformation_logic: Option<String>,
    pub integration_id: Option<Uuid>,
}

pub async fn add_lineage_edge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddLineageEdgeRequest>,
) -> Result<Json<LineageEdge>, StatusCode> {
    match state.lineage.add_edge(
        req.tenant_id,
        req.source_entity_id,
        req.target_entity_id,
        req.transformation_type,
        req.transformation_logic,
        req.integration_id,
    ).await {
        Ok(edge) => Ok(Json(edge)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Serialize)]
pub struct ImpactAnalysisResponse {
    pub entity_id: Uuid,
    pub downstream_entities: Vec<Entity>,
    pub affected_integrations: Vec<Uuid>,
}

pub async fn get_impact_analysis(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<ImpactAnalysisResponse>, StatusCode> {
    match state.lineage.get_impact_analysis(query.tenant_id, entity_id).await {
        Ok((downstream, integrations)) => Ok(Json(ImpactAnalysisResponse {
            entity_id,
            downstream_entities: downstream,
            affected_integrations: integrations,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Quality Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct ListQualityRulesQuery {
    pub tenant_id: Uuid,
    pub entity_id: Option<Uuid>,
}

pub async fn list_quality_rules(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQualityRulesQuery>,
) -> Result<Json<Vec<QualityRule>>, StatusCode> {
    match state.quality.list_rules(query.tenant_id, query.entity_id).await {
        Ok(rules) => Ok(Json(rules)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CreateQualityRuleRequest {
    pub tenant_id: Uuid,
    pub entity_id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub expression: String,
    pub severity: String,
    pub description: Option<String>,
}

pub async fn create_quality_rule(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateQualityRuleRequest>,
) -> Result<Json<QualityRule>, StatusCode> {
    match state.quality.create_rule(
        req.tenant_id,
        req.entity_id,
        req.name,
        req.rule_type,
        req.expression,
        req.severity,
        req.description,
    ).await {
        Ok(rule) => Ok(Json(rule)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_quality_score(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<QualityScore>, StatusCode> {
    match state.quality.get_score(query.tenant_id, entity_id).await {
        Ok(Some(score)) => Ok(Json(score)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_quality_issues(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Vec<QualityIssue>>, StatusCode> {
    match state.quality.get_issues(query.tenant_id, entity_id).await {
        Ok(issues) => Ok(Json(issues)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

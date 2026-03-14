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
use crate::lineage::{LineageEdge, LineageEdgeType, LineageGraph, LineageNode, LineageQueryParams};
use crate::quality::{
    QualityRule, QualityRuleInput, QualityScore, QualityCheckResult, QualityIssue, PiiScanResult,
};

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
    pub direction: Option<String>,
    pub depth: Option<i64>,
    pub entity_type: Option<String>,
}

/// GET /lineage/:entity_id - Get lineage graph for an entity
pub async fn get_lineage(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<String>,
    Query(query): Query<GetLineageQuery>,
) -> Result<Json<LineageGraph>, StatusCode> {
    let params = LineageQueryParams {
        entity_id,
        direction: query.direction,
        depth: query.depth,
        entity_type: query.entity_type,
    };

    match state.lineage.get_lineage(query.tenant_id, params).await {
        Ok(graph) => Ok(Json(graph)),
        Err(e) => {
            tracing::error!("Failed to get lineage: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct RecordLineageRequest {
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub edge_type: String,
    pub transformation: Option<String>,
    pub integration_id: Option<Uuid>,
}

/// POST /lineage - Record a lineage edge
pub async fn record_lineage(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordLineageRequest>,
) -> Result<Json<LineageEdge>, StatusCode> {
    let edge_type = LineageEdgeType::from_str(&req.edge_type);

    match state.lineage.record_lineage(
        req.tenant_id,
        req.source_id,
        req.target_id,
        edge_type,
        req.transformation,
        req.integration_id,
    ).await {
        Ok(edge) => Ok(Json(edge)),
        Err(e) => {
            tracing::error!("Failed to record lineage: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /lineage/:entity_id/impact - Get downstream impact analysis
pub async fn get_lineage_impact(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<String>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Vec<LineageNode>>, StatusCode> {
    match state.lineage.get_impact(query.tenant_id, &entity_id).await {
        Ok(nodes) => Ok(Json(nodes)),
        Err(e) => {
            tracing::error!("Failed to get impact analysis: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /lineage/:entity_id/sources - Get root data sources
pub async fn get_lineage_sources(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<String>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Vec<LineageNode>>, StatusCode> {
    match state.lineage.get_root_sources(query.tenant_id, &entity_id).await {
        Ok(nodes) => Ok(Json(nodes)),
        Err(e) => {
            tracing::error!("Failed to get root sources: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// DELETE /lineage/:entity_id - Remove a lineage node and its edges
pub async fn delete_lineage(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<String>,
    Query(query): Query<GetEntityQuery>,
) -> Result<StatusCode, StatusCode> {
    match state.lineage.delete_lineage(query.tenant_id, &entity_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete lineage: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Quality Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct ListQualityRulesQuery {
    pub tenant_id: Uuid,
    pub dataset_id: Option<Uuid>,
}

/// GET /quality/rules - List quality rules
pub async fn list_quality_rules(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQualityRulesQuery>,
) -> Result<Json<Vec<QualityRule>>, StatusCode> {
    match state.quality.list_rules(query.tenant_id, query.dataset_id).await {
        Ok(rules) => Ok(Json(rules)),
        Err(e) => {
            tracing::error!("Failed to list quality rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct CreateQualityRuleRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub dataset_id: Uuid,
    pub column_name: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

/// POST /quality/rules - Create a quality rule
pub async fn create_quality_rule(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateQualityRuleRequest>,
) -> Result<Json<QualityRule>, StatusCode> {
    let input = QualityRuleInput {
        name: req.name,
        description: req.description,
        rule_type: req.rule_type,
        dataset_id: req.dataset_id,
        column_name: req.column_name,
        parameters: req.parameters,
        severity: req.severity,
        enabled: req.enabled,
    };

    match state.quality.create_rule(req.tenant_id, input).await {
        Ok(rule) => Ok(Json(rule)),
        Err(e) => {
            tracing::error!("Failed to create quality rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateQualityRuleRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub dataset_id: Uuid,
    pub column_name: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

/// PUT /quality/rules/:id - Update a quality rule
pub async fn update_quality_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
    Json(req): Json<UpdateQualityRuleRequest>,
) -> Result<Json<QualityRule>, StatusCode> {
    let input = QualityRuleInput {
        name: req.name,
        description: req.description,
        rule_type: req.rule_type,
        dataset_id: req.dataset_id,
        column_name: req.column_name,
        parameters: req.parameters,
        severity: req.severity,
        enabled: req.enabled,
    };

    match state.quality.update_rule(req.tenant_id, rule_id, input).await {
        Ok(Some(rule)) => Ok(Json(rule)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update quality rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteQualityRuleQuery {
    pub tenant_id: Uuid,
}

/// DELETE /quality/rules/:id - Delete a quality rule
pub async fn delete_quality_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
    Query(query): Query<DeleteQualityRuleQuery>,
) -> Result<StatusCode, StatusCode> {
    match state.quality.delete_rule(query.tenant_id, rule_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete quality rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct EvaluateDatasetQuery {
    pub tenant_id: Uuid,
}

/// POST /quality/evaluate/:dataset_id - Evaluate all rules for a dataset
pub async fn evaluate_dataset(
    State(state): State<Arc<AppState>>,
    Path(dataset_id): Path<Uuid>,
    Query(query): Query<EvaluateDatasetQuery>,
) -> Result<Json<Vec<QualityCheckResult>>, StatusCode> {
    match state.quality.evaluate_dataset(query.tenant_id, dataset_id).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => {
            tracing::error!("Failed to evaluate dataset: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /quality/score/:dataset_id - Get quality score for a dataset
pub async fn get_quality_score(
    State(state): State<Arc<AppState>>,
    Path(dataset_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<QualityScore>, StatusCode> {
    match state.quality.get_quality_score(query.tenant_id, dataset_id).await {
        Ok(score) => Ok(Json(score)),
        Err(e) => {
            tracing::error!("Failed to get quality score: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /quality/pii-scan/:dataset_id - Run PII detection scan
pub async fn pii_scan(
    State(state): State<Arc<AppState>>,
    Path(dataset_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<PiiScanResult>, StatusCode> {
    match state.quality.detect_pii(query.tenant_id, dataset_id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Failed to run PII scan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /quality/entities/:id/issues - Get quality issues (backward compatibility)
pub async fn get_quality_issues(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Vec<QualityIssue>>, StatusCode> {
    match state.quality.get_issues(query.tenant_id, entity_id).await {
        Ok(issues) => Ok(Json(issues)),
        Err(e) => {
            tracing::error!("Failed to get quality issues: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

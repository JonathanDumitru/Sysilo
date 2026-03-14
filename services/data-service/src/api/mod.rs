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
use crate::contracts::{
    AddTermRequest, CheckUsageRequest, CheckUsageResponse, ContractHistory, ContractTerm,
    ContractValidationResult, ContractWithTerms, CreateContractRequest, ListContractsFilter,
    SemanticContract, UpdateContractRequest, UpdateTermRequest, ValidationContext,
};
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

// ============================================================================
// Contract Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct CreateContractApiRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub entity_id: Option<Uuid>,
}

/// POST /contracts - Create a new semantic contract
pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContractApiRequest>,
) -> Result<Json<SemanticContract>, StatusCode> {
    let request = CreateContractRequest {
        name: req.name,
        description: req.description,
        owner_id: req.owner_id,
        entity_id: req.entity_id,
    };

    match state.contracts.create_contract(req.tenant_id, request).await {
        Ok(contract) => Ok(Json(contract)),
        Err(e) => {
            tracing::error!("Failed to create contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct ListContractsQuery {
    pub tenant_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub status: Option<String>,
}

/// GET /contracts - List contracts with optional filters
pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListContractsQuery>,
) -> Result<Json<Vec<SemanticContract>>, StatusCode> {
    let filters = ListContractsFilter {
        entity_id: query.entity_id,
        status: query.status,
    };

    match state.contracts.list_contracts(query.tenant_id, filters).await {
        Ok(contracts) => Ok(Json(contracts)),
        Err(e) => {
            tracing::error!("Failed to list contracts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /contracts/:id - Get a contract with all its terms
pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<ContractWithTerms>, StatusCode> {
    match state.contracts.get_contract(query.tenant_id, id).await {
        Ok(Some(contract)) => Ok(Json(contract)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateContractApiRequest {
    pub tenant_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
}

/// PUT /contracts/:id - Update a contract (bumps version)
pub async fn update_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContractApiRequest>,
) -> Result<Json<SemanticContract>, StatusCode> {
    let request = UpdateContractRequest {
        name: req.name,
        description: req.description,
        owner_id: req.owner_id,
        entity_id: req.entity_id,
    };

    match state.contracts.update_contract(req.tenant_id, id, request).await {
        Ok(Some(contract)) => Ok(Json(contract)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /contracts/:id/activate - Activate a contract
pub async fn activate_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<SemanticContract>, StatusCode> {
    match state.contracts.activate_contract(query.tenant_id, id).await {
        Ok(Some(contract)) => Ok(Json(contract)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to activate contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /contracts/:id/deprecate - Deprecate a contract
pub async fn deprecate_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<SemanticContract>, StatusCode> {
    match state.contracts.deprecate_contract(query.tenant_id, id).await {
        Ok(Some(contract)) => Ok(Json(contract)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to deprecate contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct AddTermApiRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub term: AddTermRequest,
}

/// POST /contracts/:id/terms - Add a term to a contract
pub async fn add_contract_term(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    Json(req): Json<AddTermApiRequest>,
) -> Result<Json<ContractTerm>, StatusCode> {
    // Verify contract exists and belongs to tenant
    match state.contracts.get_contract(req.tenant_id, contract_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to verify contract: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match state.contracts.add_term(contract_id, req.term).await {
        Ok(term) => Ok(Json(term)),
        Err(e) => {
            tracing::error!("Failed to add contract term: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// PUT /contracts/terms/:term_id - Update a contract term
pub async fn update_contract_term(
    State(state): State<Arc<AppState>>,
    Path(term_id): Path<Uuid>,
    Json(req): Json<UpdateTermRequest>,
) -> Result<Json<ContractTerm>, StatusCode> {
    match state.contracts.update_term(term_id, req).await {
        Ok(Some(term)) => Ok(Json(term)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update contract term: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// DELETE /contracts/terms/:term_id - Remove a contract term
pub async fn remove_contract_term(
    State(state): State<Arc<AppState>>,
    Path(term_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.contracts.remove_term(term_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to remove contract term: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct ValidateContractApiRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub context: ValidationContext,
}

/// POST /contracts/:id/validate - Validate a contract at runtime
pub async fn validate_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ValidateContractApiRequest>,
) -> Result<Json<ContractValidationResult>, StatusCode> {
    match state.contracts.validate_contract(req.tenant_id, id, req.context).await {
        Ok(Some(result)) => Ok(Json(result)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to validate contract: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct CheckUsageApiRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub inner: CheckUsageRequest,
}

/// POST /contracts/:id/check-usage - Check if a use case is allowed
pub async fn check_contract_usage(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CheckUsageApiRequest>,
) -> Result<Json<CheckUsageResponse>, StatusCode> {
    match state.contracts.check_usage(req.tenant_id, id, &req.inner.use_case).await {
        Ok(Some(result)) => Ok(Json(result)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to check contract usage: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /contracts/:id/history - Get contract version history
pub async fn get_contract_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetEntityQuery>,
) -> Result<Json<Vec<ContractHistory>>, StatusCode> {
    match state.contracts.get_contract_history(query.tenant_id, id).await {
        Ok(history) => Ok(Json(history)),
        Err(e) => {
            tracing::error!("Failed to get contract history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

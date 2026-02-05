use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::assets::{Asset, CreateAssetRequest, UpdateAssetRequest};
use crate::relationships::{Relationship, CreateRelationshipRequest, RelationshipWithAsset};
use crate::graph::{SubGraph, AssetPath};
use crate::impact::{ImpactAnalysis, DownstreamImpact, UpstreamDependencies};

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
        service: "asset-service".to_string(),
    })
}

pub async fn ready(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, StatusCode> {
    if state.assets.health_check().await.is_ok() {
        Ok(Json(HealthResponse {
            status: "ready".to_string(),
            service: "asset-service".to_string(),
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// ============================================================================
// Asset Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct ListAssetsQuery {
    pub tenant_id: Uuid,
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct ListAssetsResponse {
    pub assets: Vec<Asset>,
    pub total: i64,
}

pub async fn list_assets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<ListAssetsResponse>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match state.assets.list_assets(
        query.tenant_id,
        query.asset_type,
        query.status,
        limit,
        offset,
    ).await {
        Ok((assets, total)) => Ok(Json(ListAssetsResponse { assets, total })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CreateAssetBody {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub request: CreateAssetRequest,
}

pub async fn create_asset(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateAssetBody>,
) -> Result<Json<Asset>, StatusCode> {
    match state.assets.create_asset(body.tenant_id, body.request).await {
        Ok(asset) => Ok(Json(asset)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Asset>, StatusCode> {
    match state.assets.get_asset(query.tenant_id, id).await {
        Ok(Some(asset)) => Ok(Json(asset)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct UpdateAssetBody {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub request: UpdateAssetRequest,
}

pub async fn update_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAssetBody>,
) -> Result<Json<Asset>, StatusCode> {
    match state.assets.update_asset(body.tenant_id, id, body.request).await {
        Ok(Some(asset)) => Ok(Json(asset)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<StatusCode, StatusCode> {
    match state.assets.delete_asset(query.tenant_id, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct SearchAssetsQuery {
    pub tenant_id: Uuid,
    pub q: String,
    pub limit: Option<i64>,
}

pub async fn search_assets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchAssetsQuery>,
) -> Result<Json<Vec<Asset>>, StatusCode> {
    let limit = query.limit.unwrap_or(20);

    match state.assets.search_assets(query.tenant_id, &query.q, limit).await {
        Ok(assets) => Ok(Json(assets)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Relationship Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct ListRelationshipsQuery {
    pub tenant_id: Uuid,
    pub relationship_type: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_relationships(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListRelationshipsQuery>,
) -> Result<Json<Vec<Relationship>>, StatusCode> {
    let limit = query.limit.unwrap_or(100);

    match state.relationships.list_relationships(
        query.tenant_id,
        query.relationship_type,
        limit,
    ).await {
        Ok(relationships) => Ok(Json(relationships)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CreateRelationshipBody {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub request: CreateRelationshipRequest,
}

pub async fn create_relationship(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRelationshipBody>,
) -> Result<Json<Relationship>, StatusCode> {
    match state.relationships.create_relationship(body.tenant_id, body.request).await {
        Ok(relationship) => Ok(Json(relationship)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_relationship(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<StatusCode, StatusCode> {
    match state.relationships.delete_relationship(query.tenant_id, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_asset_relationships(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<RelationshipWithAsset>>, StatusCode> {
    match state.relationships.get_asset_relationships(query.tenant_id, id).await {
        Ok(relationships) => Ok(Json(relationships)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Graph Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct GetNeighborsQuery {
    pub tenant_id: Uuid,
    pub direction: Option<String>,
}

pub async fn get_neighbors(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetNeighborsQuery>,
) -> Result<Json<SubGraph>, StatusCode> {
    match state.graph.get_neighbors(
        query.tenant_id,
        id,
        query.direction.as_deref(),
    ).await {
        Ok(subgraph) => Ok(Json(subgraph)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct FindPathQuery {
    pub tenant_id: Uuid,
    pub start_id: Uuid,
    pub end_id: Uuid,
}

pub async fn find_path(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FindPathQuery>,
) -> Result<Json<Option<AssetPath>>, StatusCode> {
    match state.graph.find_path(
        query.tenant_id,
        query.start_id,
        query.end_id,
    ).await {
        Ok(path) => Ok(Json(path)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct GetSubgraphQuery {
    pub tenant_id: Uuid,
    pub depth: Option<i32>,
}

pub async fn get_subgraph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<GetSubgraphQuery>,
) -> Result<Json<SubGraph>, StatusCode> {
    let depth = query.depth.unwrap_or(3);

    match state.graph.get_subgraph(query.tenant_id, id, depth).await {
        Ok(subgraph) => Ok(Json(subgraph)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Impact Analysis Endpoints
// ============================================================================

pub async fn get_impact_analysis(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<ImpactAnalysis>, StatusCode> {
    // This would use an ImpactService - simplified for now
    Err(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Deserialize)]
pub struct ImpactQuery {
    pub tenant_id: Uuid,
    pub depth: Option<i32>,
}

pub async fn get_downstream_impact(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ImpactQuery>,
) -> Result<Json<DownstreamImpact>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn get_upstream_dependencies(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ImpactQuery>,
) -> Result<Json<UpstreamDependencies>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

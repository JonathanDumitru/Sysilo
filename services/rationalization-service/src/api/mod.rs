use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::scoring::{ScoringService, TimeQuadrant};
use crate::scenarios::ScenariosService;
use crate::playbooks::PlaybooksService;
use crate::recommendations::RecommendationsService;
use crate::live_scoring::{LiveScoringService, ScoreEvent};

// ============================================================================
// App State
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ai_service_url: String,
    pub live_scoring: LiveScoringService,
}

impl AppState {
    pub fn new(pool: PgPool, ai_service_url: String, live_scoring: LiveScoringService) -> Self {
        Self { pool, ai_service_url, live_scoring }
    }
}

// ============================================================================
// Health Endpoints
// ============================================================================

pub async fn health() -> &'static str {
    "OK"
}

pub async fn ready(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok("Ready")
}

// ============================================================================
// Application Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Application {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub business_capability: Option<String>,
    pub business_unit: Option<String>,
    pub application_type: Option<String>,
    pub criticality: String,
    pub lifecycle_stage: String,
    pub go_live_date: Option<chrono::NaiveDate>,
    pub sunset_date: Option<chrono::NaiveDate>,
    pub business_owner_id: Option<Uuid>,
    pub technical_owner_id: Option<Uuid>,
    pub license_cost: Option<rust_decimal::Decimal>,
    pub infrastructure_cost: Option<rust_decimal::Decimal>,
    pub support_cost: Option<rust_decimal::Decimal>,
    pub development_cost: Option<rust_decimal::Decimal>,
    pub total_cost: Option<rust_decimal::Decimal>,
    pub technology_stack: Option<serde_json::Value>,
    pub hosting_model: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub business_capability: Option<String>,
    pub business_unit: Option<String>,
    pub application_type: Option<String>,
    pub criticality: Option<String>,
    pub lifecycle_stage: Option<String>,
    pub license_cost: Option<rust_decimal::Decimal>,
    pub infrastructure_cost: Option<rust_decimal::Decimal>,
    pub support_cost: Option<rust_decimal::Decimal>,
    pub development_cost: Option<rust_decimal::Decimal>,
    pub technology_stack: Option<serde_json::Value>,
    pub hosting_model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListApplicationsQuery {
    pub tenant_id: Uuid,
    pub lifecycle_stage: Option<String>,
    pub criticality: Option<String>,
    pub business_unit: Option<String>,
}

// ============================================================================
// Application Endpoints
// ============================================================================

pub async fn list_applications(
    State(state): State<AppState>,
    Query(query): Query<ListApplicationsQuery>,
) -> Result<Json<Vec<Application>>, StatusCode> {
    let apps = sqlx::query_as::<_, Application>(
        r#"
        SELECT id, tenant_id, asset_id, name, description, vendor, version,
               business_capability, business_unit, application_type, criticality,
               lifecycle_stage, go_live_date, sunset_date, business_owner_id,
               technical_owner_id, license_cost, infrastructure_cost, support_cost,
               development_cost, total_cost, technology_stack, hosting_model,
               metadata, created_at, updated_at
        FROM applications
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR lifecycle_stage = $2)
          AND ($3::text IS NULL OR criticality = $3)
          AND ($4::text IS NULL OR business_unit = $4)
        ORDER BY name
        "#
    )
    .bind(query.tenant_id)
    .bind(query.lifecycle_stage)
    .bind(query.criticality)
    .bind(query.business_unit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list applications: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(apps))
}

pub async fn create_application(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateApplicationRequest>,
) -> Result<Json<Application>, StatusCode> {
    let app = sqlx::query_as::<_, Application>(
        r#"
        INSERT INTO applications (
            tenant_id, name, description, vendor, version, business_capability,
            business_unit, application_type, criticality, lifecycle_stage,
            license_cost, infrastructure_cost, support_cost, development_cost,
            technology_stack, hosting_model
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id, tenant_id, asset_id, name, description, vendor, version,
                  business_capability, business_unit, application_type, criticality,
                  lifecycle_stage, go_live_date, sunset_date, business_owner_id,
                  technical_owner_id, license_cost, infrastructure_cost, support_cost,
                  development_cost, total_cost, technology_stack, hosting_model,
                  metadata, created_at, updated_at
        "#
    )
    .bind(tenant.tenant_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.vendor)
    .bind(&req.version)
    .bind(&req.business_capability)
    .bind(&req.business_unit)
    .bind(&req.application_type)
    .bind(req.criticality.as_deref().unwrap_or("medium"))
    .bind(req.lifecycle_stage.as_deref().unwrap_or("production"))
    .bind(req.license_cost)
    .bind(req.infrastructure_cost)
    .bind(req.support_cost)
    .bind(req.development_cost)
    .bind(&req.technology_stack)
    .bind(&req.hosting_model)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create application: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(app))
}

pub async fn get_application(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Application>, StatusCode> {
    let app = sqlx::query_as::<_, Application>(
        r#"
        SELECT id, tenant_id, asset_id, name, description, vendor, version,
               business_capability, business_unit, application_type, criticality,
               lifecycle_stage, go_live_date, sunset_date, business_owner_id,
               technical_owner_id, license_cost, infrastructure_cost, support_cost,
               development_cost, total_cost, technology_stack, hosting_model,
               metadata, created_at, updated_at
        FROM applications
        WHERE id = $1 AND tenant_id = $2
        "#
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(app))
}

pub async fn update_application(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateApplicationRequest>,
) -> Result<Json<Application>, StatusCode> {
    let app = sqlx::query_as::<_, Application>(
        r#"
        UPDATE applications SET
            name = $1, description = $2, vendor = $3, version = $4,
            business_capability = $5, business_unit = $6, application_type = $7,
            criticality = COALESCE($8, criticality), lifecycle_stage = COALESCE($9, lifecycle_stage),
            license_cost = $10, infrastructure_cost = $11, support_cost = $12,
            development_cost = $13, technology_stack = $14, hosting_model = $15,
            updated_at = NOW()
        WHERE id = $16 AND tenant_id = $17
        RETURNING id, tenant_id, asset_id, name, description, vendor, version,
                  business_capability, business_unit, application_type, criticality,
                  lifecycle_stage, go_live_date, sunset_date, business_owner_id,
                  technical_owner_id, license_cost, infrastructure_cost, support_cost,
                  development_cost, total_cost, technology_stack, hosting_model,
                  metadata, created_at, updated_at
        "#
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.vendor)
    .bind(&req.version)
    .bind(&req.business_capability)
    .bind(&req.business_unit)
    .bind(&req.application_type)
    .bind(&req.criticality)
    .bind(&req.lifecycle_stage)
    .bind(req.license_cost)
    .bind(req.infrastructure_cost)
    .bind(req.support_cost)
    .bind(req.development_cost)
    .bind(&req.technology_stack)
    .bind(&req.hosting_model)
    .bind(id)
    .bind(tenant.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(app))
}

pub async fn delete_application(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM applications WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.tenant_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Common Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

// ============================================================================
// Scoring Endpoints
// ============================================================================

pub async fn get_application_scores(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scoring::ApplicationScore>>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let scores = service.get_scores(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(scores))
}

#[derive(Debug, Deserialize)]
pub struct SetScoreRequest {
    pub dimension_id: Uuid,
    pub score: rust_decimal::Decimal,
    pub notes: Option<String>,
    pub evidence: Option<serde_json::Value>,
}

pub async fn set_application_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<SetScoreRequest>,
) -> Result<Json<crate::scoring::ApplicationScore>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let score = service.set_score(
        tenant.tenant_id,
        id,
        req.dimension_id,
        req.score,
        req.notes,
        req.evidence,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(score))
}

pub async fn list_scoring_dimensions(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scoring::ScoringDimension>>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let dimensions = service.list_dimensions(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dimensions))
}

#[derive(Debug, Deserialize)]
pub struct CreateDimensionRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub weight: rust_decimal::Decimal,
    pub scoring_criteria: serde_json::Value,
}

pub async fn create_scoring_dimension(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateDimensionRequest>,
) -> Result<Json<crate::scoring::ScoringDimension>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let dimension = service.create_dimension(
        tenant.tenant_id,
        req.name,
        req.description,
        req.category,
        req.weight,
        req.scoring_criteria,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dimension))
}

pub async fn update_scoring_dimension(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateDimensionRequest>,
) -> Result<Json<crate::scoring::ScoringDimension>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let dimension = service.update_dimension(
        tenant.tenant_id,
        id,
        req.name,
        req.description,
        req.category,
        req.weight,
        req.scoring_criteria,
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(dimension))
}

pub async fn calculate_time_quadrant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::scoring::TimeAssessment>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let assessment = service.calculate_time_assessment(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assessment))
}

pub async fn bulk_calculate_time_quadrants(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scoring::TimeAssessment>>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let assessments = service.bulk_calculate_time_assessments(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assessments))
}

pub async fn list_time_assessments(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scoring::TimeAssessment>>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let assessments = service.list_time_assessments(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assessments))
}

pub async fn get_time_assessment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::scoring::TimeAssessment>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let assessment = service.get_time_assessment(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(assessment))
}

#[derive(Debug, Deserialize)]
pub struct OverrideTimeAssessmentRequest {
    pub quadrant: String,
    pub reason: String,
}

pub async fn override_time_assessment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<OverrideTimeAssessmentRequest>,
) -> Result<Json<crate::scoring::TimeAssessment>, StatusCode> {
    let quadrant = TimeQuadrant::from_str(&req.quadrant)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let service = ScoringService::new(state.pool.clone());
    let assessment = service.override_time_assessment(tenant.tenant_id, id, quadrant, req.reason).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(assessment))
}

#[derive(Debug, Serialize)]
pub struct TimeSummary {
    pub tolerate: i64,
    pub invest: i64,
    pub migrate: i64,
    pub eliminate: i64,
    pub total_applications: i64,
}

pub async fn get_time_summary(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<TimeSummary>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let summary = service.get_time_summary(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(summary))
}

// ============================================================================
// Dependencies & Impact Analysis
// ============================================================================

pub async fn get_application_dependencies(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scoring::ApplicationDependency>>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let deps = service.get_dependencies(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(deps))
}

pub async fn get_impact_analysis(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::scoring::ImpactAnalysis>, StatusCode> {
    let service = ScoringService::new(state.pool.clone());
    let impact = service.analyze_impact(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(impact))
}

// ============================================================================
// Scenarios Endpoints
// ============================================================================

pub async fn list_scenarios(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::scenarios::Scenario>>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let scenarios = service.list(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(scenarios))
}

pub async fn create_scenario(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::scenarios::CreateScenarioRequest>,
) -> Result<Json<crate::scenarios::Scenario>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let scenario = service.create(tenant.tenant_id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(scenario))
}

pub async fn get_scenario(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::scenarios::Scenario>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let scenario = service.get(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(scenario))
}

pub async fn update_scenario(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::scenarios::UpdateScenarioRequest>,
) -> Result<Json<crate::scenarios::Scenario>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let scenario = service.update(tenant.tenant_id, id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(scenario))
}

pub async fn delete_scenario(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<StatusCode, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let deleted = service.delete(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn analyze_scenario(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::scenarios::ScenarioAnalysis>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let analysis = service.analyze(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(analysis))
}

#[derive(Debug, Deserialize)]
pub struct CompareScenarioRequest {
    pub scenario_ids: Vec<Uuid>,
}

pub async fn compare_scenarios(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CompareScenarioRequest>,
) -> Result<Json<crate::scenarios::ScenarioComparison>, StatusCode> {
    let service = ScenariosService::new(state.pool.clone());
    let comparison = service.compare(tenant.tenant_id, req.scenario_ids).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(comparison))
}

// ============================================================================
// Playbooks Endpoints
// ============================================================================

pub async fn list_playbooks(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::playbooks::Playbook>>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let playbooks = service.list(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(playbooks))
}

pub async fn create_playbook(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::playbooks::CreatePlaybookRequest>,
) -> Result<Json<crate::playbooks::Playbook>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let playbook = service.create(tenant.tenant_id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(playbook))
}

pub async fn get_playbook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::playbooks::Playbook>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let playbook = service.get(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(playbook))
}

pub async fn update_playbook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::playbooks::CreatePlaybookRequest>,
) -> Result<Json<crate::playbooks::Playbook>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let playbook = service.update(tenant.tenant_id, id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(playbook))
}

pub async fn list_playbook_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::playbooks::Playbook>>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let templates = service.list_templates().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(templates))
}

// ============================================================================
// Migration Projects Endpoints
// ============================================================================

pub async fn list_migration_projects(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::playbooks::MigrationProject>>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let projects = service.list_projects(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(projects))
}

pub async fn create_migration_project(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::playbooks::CreateProjectRequest>,
) -> Result<Json<crate::playbooks::MigrationProject>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let project = service.create_project(tenant.tenant_id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(project))
}

pub async fn get_migration_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::playbooks::MigrationProject>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let project = service.get_project(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(project))
}

pub async fn update_migration_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<crate::playbooks::UpdateProjectRequest>,
) -> Result<Json<crate::playbooks::MigrationProject>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let project = service.update_project(tenant.tenant_id, id, req).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(project))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgressRequest {
    pub current_phase: i32,
    pub progress_percent: i32,
    pub task_status: Option<serde_json::Value>,
}

pub async fn update_project_progress(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Json<crate::playbooks::MigrationProject>, StatusCode> {
    let service = PlaybooksService::new(state.pool.clone());
    let project = service.update_progress(tenant.tenant_id, id, req.current_phase, req.progress_percent, req.task_status).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(project))
}

// ============================================================================
// Recommendations Endpoints
// ============================================================================

pub async fn list_recommendations(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::recommendations::Recommendation>>, StatusCode> {
    let service = RecommendationsService::new(state.pool.clone(), state.ai_service_url.clone());
    let recommendations = service.list(tenant.tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(recommendations))
}

#[derive(Debug, Deserialize)]
pub struct GenerateRecommendationsRequest {
    pub application_id: Option<Uuid>,
    pub scenario_id: Option<Uuid>,
}

pub async fn generate_recommendations(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<GenerateRecommendationsRequest>,
) -> Result<Json<Vec<crate::recommendations::Recommendation>>, StatusCode> {
    let service = RecommendationsService::new(state.pool.clone(), state.ai_service_url.clone());
    let recommendations = service.generate(tenant.tenant_id, req.application_id, req.scenario_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(recommendations))
}

pub async fn get_recommendation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::recommendations::Recommendation>, StatusCode> {
    let service = RecommendationsService::new(state.pool.clone(), state.ai_service_url.clone());
    let recommendation = service.get(tenant.tenant_id, id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(recommendation))
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecommendationStatusRequest {
    pub status: String,
    pub feedback: Option<String>,
}

pub async fn update_recommendation_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<UpdateRecommendationStatusRequest>,
) -> Result<Json<crate::recommendations::Recommendation>, StatusCode> {
    let service = RecommendationsService::new(state.pool.clone(), state.ai_service_url.clone());
    let recommendation = service.update_status(tenant.tenant_id, id, req.status, req.feedback).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(recommendation))
}

// ============================================================================
// Analytics Endpoints
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PortfolioAnalytics {
    pub total_applications: i64,
    pub by_lifecycle: serde_json::Value,
    pub by_criticality: serde_json::Value,
    pub by_quadrant: serde_json::Value,
    pub total_cost: rust_decimal::Decimal,
    pub avg_health_score: f64,
    pub avg_value_score: f64,
}

pub async fn get_portfolio_analytics(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<PortfolioAnalytics>, StatusCode> {
    // This would aggregate data from multiple sources
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM applications WHERE tenant_id = $1"
    )
    .bind(tenant.tenant_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_cost = sqlx::query_scalar::<_, Option<rust_decimal::Decimal>>(
        "SELECT SUM(total_cost) FROM applications WHERE tenant_id = $1"
    )
    .bind(tenant.tenant_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or_default();

    Ok(Json(PortfolioAnalytics {
        total_applications: total,
        by_lifecycle: serde_json::json!({}),
        by_criticality: serde_json::json!({}),
        by_quadrant: serde_json::json!({}),
        total_cost,
        avg_health_score: 0.0,
        avg_value_score: 0.0,
    }))
}

pub async fn get_score_trends(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Would return historical score data for trend analysis
    Ok(Json(serde_json::json!({
        "periods": [],
        "scores": []
    })))
}

pub async fn get_cost_analysis(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let costs = sqlx::query_as::<_, (String, Option<rust_decimal::Decimal>)>(
        r#"
        SELECT lifecycle_stage, SUM(total_cost)
        FROM applications
        WHERE tenant_id = $1
        GROUP BY lifecycle_stage
        "#
    )
    .bind(tenant.tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: std::collections::HashMap<_, _> = costs.into_iter().collect();
    Ok(Json(serde_json::to_value(result).unwrap()))
}

// ============================================================================
// Live Scoring Endpoints
// ============================================================================

pub async fn list_live_scores(
    State(state): State<AppState>,
    Query(filters): Query<crate::live_scoring::LiveScoreFilters>,
) -> Result<Json<Vec<crate::live_scoring::LiveTimeScore>>, StatusCode> {
    let scores = state.live_scoring.get_live_scores(filters).await
        .map_err(|e| {
            tracing::error!("Failed to list live scores: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(scores))
}

pub async fn get_live_score(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::live_scoring::LiveTimeScore>, StatusCode> {
    let score = state.live_scoring.get_live_score(tenant.tenant_id, asset_id).await
        .map_err(|e| {
            tracing::error!("Failed to get live score: {}", e);
            StatusCode::NOT_FOUND
        })?;
    Ok(Json(score))
}

pub async fn get_live_score_drifts(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<Vec<crate::live_scoring::ScoreDrift>>, StatusCode> {
    let drifts = state.live_scoring.get_drift_history(tenant.tenant_id, asset_id).await
        .map_err(|e| {
            tracing::error!("Failed to get drift history: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(drifts))
}

#[derive(Debug, Deserialize)]
pub struct SubmitScoreEventRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub event: ScoreEvent,
}

pub async fn submit_score_event(
    State(state): State<AppState>,
    Json(req): Json<SubmitScoreEventRequest>,
) -> Result<Json<crate::live_scoring::LiveTimeScore>, StatusCode> {
    let score = state.live_scoring.process_event(req.tenant_id, req.event).await
        .map_err(|e| {
            tracing::error!("Failed to process score event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(score))
}

pub async fn get_score_feed(
    State(state): State<AppState>,
    Query(query): Query<crate::live_scoring::ScoreFeedQuery>,
) -> Result<Json<Vec<crate::live_scoring::ScoreFeedEntry>>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    let feed = state.live_scoring.get_score_feed(query.tenant_id, limit, query.since).await
        .map_err(|e| {
            tracing::error!("Failed to get score feed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(feed))
}

pub async fn get_live_portfolio_summary(
    State(state): State<AppState>,
    Query(tenant): Query<TenantQuery>,
) -> Result<Json<crate::live_scoring::LivePortfolioSummary>, StatusCode> {
    let summary = state.live_scoring.get_portfolio_summary(tenant.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to get portfolio summary: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(summary))
}

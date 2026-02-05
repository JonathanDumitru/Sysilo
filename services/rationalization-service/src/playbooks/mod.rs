use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Playbook {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub playbook_type: String,
    pub phases: serde_json::Value,
    pub typical_duration_weeks: Option<i32>,
    pub complexity_level: Option<String>,
    pub required_roles: Option<serde_json::Value>,
    pub is_template: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaybookRequest {
    pub name: String,
    pub description: Option<String>,
    pub playbook_type: String,
    pub phases: serde_json::Value,
    pub typical_duration_weeks: Option<i32>,
    pub complexity_level: Option<String>,
    pub required_roles: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MigrationProject {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub playbook_id: Option<Uuid>,
    pub application_id: Uuid,
    pub name: String,
    pub status: String,
    pub current_phase: i32,
    pub progress_percent: i32,
    pub task_status: Option<serde_json::Value>,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
    pub actual_start: Option<NaiveDate>,
    pub actual_end: Option<NaiveDate>,
    pub outcomes: Option<serde_json::Value>,
    pub lessons_learned: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub scenario_id: Option<Uuid>,
    pub playbook_id: Option<Uuid>,
    pub application_id: Uuid,
    pub name: String,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
    pub project_lead_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub planned_start: Option<NaiveDate>,
    pub planned_end: Option<NaiveDate>,
    pub actual_start: Option<NaiveDate>,
    pub actual_end: Option<NaiveDate>,
    pub outcomes: Option<serde_json::Value>,
    pub lessons_learned: Option<String>,
    pub project_lead_id: Option<Uuid>,
}

// ============================================================================
// Service
// ============================================================================

pub struct PlaybooksService {
    pool: PgPool,
}

impl PlaybooksService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Playbooks
    // ========================================================================

    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<Playbook>> {
        let playbooks = sqlx::query_as::<_, Playbook>(
            r#"
            SELECT id, tenant_id, name, description, playbook_type, phases,
                   typical_duration_weeks, complexity_level, required_roles,
                   is_template, created_by, created_at, updated_at
            FROM migration_playbooks
            WHERE tenant_id = $1 OR is_template = true
            ORDER BY is_template DESC, name
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(playbooks)
    }

    pub async fn list_templates(&self) -> Result<Vec<Playbook>> {
        let playbooks = sqlx::query_as::<_, Playbook>(
            r#"
            SELECT id, tenant_id, name, description, playbook_type, phases,
                   typical_duration_weeks, complexity_level, required_roles,
                   is_template, created_by, created_at, updated_at
            FROM migration_playbooks
            WHERE is_template = true
            ORDER BY playbook_type, name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(playbooks)
    }

    pub async fn create(&self, tenant_id: Uuid, req: CreatePlaybookRequest) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
            r#"
            INSERT INTO migration_playbooks (
                tenant_id, name, description, playbook_type, phases,
                typical_duration_weeks, complexity_level, required_roles, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, tenant_id, name, description, playbook_type, phases,
                      typical_duration_weeks, complexity_level, required_roles,
                      is_template, created_by, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.playbook_type)
        .bind(&req.phases)
        .bind(req.typical_duration_weeks)
        .bind(&req.complexity_level)
        .bind(&req.required_roles)
        .bind(req.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(playbook)
    }

    pub async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Playbook>> {
        let playbook = sqlx::query_as::<_, Playbook>(
            r#"
            SELECT id, tenant_id, name, description, playbook_type, phases,
                   typical_duration_weeks, complexity_level, required_roles,
                   is_template, created_by, created_at, updated_at
            FROM migration_playbooks
            WHERE id = $1 AND (tenant_id = $2 OR is_template = true)
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(playbook)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: CreatePlaybookRequest,
    ) -> Result<Option<Playbook>> {
        let playbook = sqlx::query_as::<_, Playbook>(
            r#"
            UPDATE migration_playbooks SET
                name = $1, description = $2, playbook_type = $3, phases = $4,
                typical_duration_weeks = $5, complexity_level = $6,
                required_roles = $7, updated_at = NOW()
            WHERE id = $8 AND tenant_id = $9 AND is_template = false
            RETURNING id, tenant_id, name, description, playbook_type, phases,
                      typical_duration_weeks, complexity_level, required_roles,
                      is_template, created_by, created_at, updated_at
            "#
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.playbook_type)
        .bind(&req.phases)
        .bind(req.typical_duration_weeks)
        .bind(&req.complexity_level)
        .bind(&req.required_roles)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(playbook)
    }

    // ========================================================================
    // Migration Projects
    // ========================================================================

    pub async fn list_projects(&self, tenant_id: Uuid) -> Result<Vec<MigrationProject>> {
        let projects = sqlx::query_as::<_, MigrationProject>(
            r#"
            SELECT id, tenant_id, scenario_id, playbook_id, application_id, name,
                   status, current_phase, progress_percent, task_status,
                   planned_start, planned_end, actual_start, actual_end,
                   outcomes, lessons_learned, project_lead_id, created_at, updated_at
            FROM migration_projects
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(projects)
    }

    pub async fn create_project(
        &self,
        tenant_id: Uuid,
        req: CreateProjectRequest,
    ) -> Result<MigrationProject> {
        let project = sqlx::query_as::<_, MigrationProject>(
            r#"
            INSERT INTO migration_projects (
                tenant_id, scenario_id, playbook_id, application_id, name,
                planned_start, planned_end, project_lead_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, tenant_id, scenario_id, playbook_id, application_id, name,
                      status, current_phase, progress_percent, task_status,
                      planned_start, planned_end, actual_start, actual_end,
                      outcomes, lessons_learned, project_lead_id, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(req.scenario_id)
        .bind(req.playbook_id)
        .bind(req.application_id)
        .bind(&req.name)
        .bind(req.planned_start)
        .bind(req.planned_end)
        .bind(req.project_lead_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn get_project(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<MigrationProject>> {
        let project = sqlx::query_as::<_, MigrationProject>(
            r#"
            SELECT id, tenant_id, scenario_id, playbook_id, application_id, name,
                   status, current_phase, progress_percent, task_status,
                   planned_start, planned_end, actual_start, actual_end,
                   outcomes, lessons_learned, project_lead_id, created_at, updated_at
            FROM migration_projects
            WHERE id = $1 AND tenant_id = $2
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn update_project(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateProjectRequest,
    ) -> Result<Option<MigrationProject>> {
        let project = sqlx::query_as::<_, MigrationProject>(
            r#"
            UPDATE migration_projects SET
                name = COALESCE($1, name),
                status = COALESCE($2, status),
                planned_start = COALESCE($3, planned_start),
                planned_end = COALESCE($4, planned_end),
                actual_start = COALESCE($5, actual_start),
                actual_end = COALESCE($6, actual_end),
                outcomes = COALESCE($7, outcomes),
                lessons_learned = COALESCE($8, lessons_learned),
                project_lead_id = COALESCE($9, project_lead_id),
                updated_at = NOW()
            WHERE id = $10 AND tenant_id = $11
            RETURNING id, tenant_id, scenario_id, playbook_id, application_id, name,
                      status, current_phase, progress_percent, task_status,
                      planned_start, planned_end, actual_start, actual_end,
                      outcomes, lessons_learned, project_lead_id, created_at, updated_at
            "#
        )
        .bind(&req.name)
        .bind(&req.status)
        .bind(req.planned_start)
        .bind(req.planned_end)
        .bind(req.actual_start)
        .bind(req.actual_end)
        .bind(&req.outcomes)
        .bind(&req.lessons_learned)
        .bind(req.project_lead_id)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn update_progress(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        current_phase: i32,
        progress_percent: i32,
        task_status: Option<serde_json::Value>,
    ) -> Result<Option<MigrationProject>> {
        let project = sqlx::query_as::<_, MigrationProject>(
            r#"
            UPDATE migration_projects SET
                current_phase = $1,
                progress_percent = $2,
                task_status = COALESCE($3, task_status),
                updated_at = NOW()
            WHERE id = $4 AND tenant_id = $5
            RETURNING id, tenant_id, scenario_id, playbook_id, application_id, name,
                      status, current_phase, progress_percent, task_status,
                      planned_start, planned_end, actual_start, actual_end,
                      outcomes, lessons_learned, project_lead_id, created_at, updated_at
            "#
        )
        .bind(current_phase)
        .bind(progress_percent)
        .bind(&task_status)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }
}

/// Create default playbook templates
pub fn get_default_playbook_templates() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "Cloud Rehost (Lift and Shift)",
            "description": "Move application to cloud with minimal changes",
            "playbook_type": "rehost",
            "typical_duration_weeks": 8,
            "complexity_level": "simple",
            "phases": [
                {
                    "name": "Assessment",
                    "description": "Evaluate current infrastructure and dependencies",
                    "tasks": [
                        {"name": "Inventory current infrastructure", "checklist": ["Document servers", "Document storage", "Document network"]},
                        {"name": "Assess dependencies", "checklist": ["Map application dependencies", "Identify external integrations"]},
                        {"name": "Define target architecture", "checklist": ["Select cloud services", "Design network topology"]}
                    ]
                },
                {
                    "name": "Preparation",
                    "description": "Set up target environment and migration tools",
                    "tasks": [
                        {"name": "Provision cloud infrastructure", "checklist": ["Create VPC", "Set up compute resources", "Configure storage"]},
                        {"name": "Set up migration tools", "checklist": ["Install migration agents", "Configure replication"]},
                        {"name": "Plan cutover", "checklist": ["Define cutover window", "Create rollback plan"]}
                    ]
                },
                {
                    "name": "Migration",
                    "description": "Execute the migration",
                    "tasks": [
                        {"name": "Replicate data", "checklist": ["Start replication", "Monitor progress", "Verify data integrity"]},
                        {"name": "Test in target", "checklist": ["Run smoke tests", "Validate functionality"]},
                        {"name": "Cutover", "checklist": ["Stop source", "Final sync", "Switch DNS"]}
                    ]
                },
                {
                    "name": "Optimization",
                    "description": "Optimize and validate the migrated application",
                    "tasks": [
                        {"name": "Performance tuning", "checklist": ["Right-size instances", "Optimize storage"]},
                        {"name": "Cost optimization", "checklist": ["Apply reserved instances", "Set up auto-scaling"]},
                        {"name": "Documentation", "checklist": ["Update runbooks", "Document architecture"]}
                    ]
                }
            ],
            "required_roles": [
                {"role": "Cloud Architect", "fte_percent": 50, "duration_weeks": 8},
                {"role": "DevOps Engineer", "fte_percent": 100, "duration_weeks": 6},
                {"role": "Application Owner", "fte_percent": 25, "duration_weeks": 8}
            ]
        }),
        serde_json::json!({
            "name": "Application Retirement",
            "description": "Safely decommission an application",
            "playbook_type": "retire",
            "typical_duration_weeks": 12,
            "complexity_level": "moderate",
            "phases": [
                {
                    "name": "Planning",
                    "description": "Plan the retirement and identify dependencies",
                    "tasks": [
                        {"name": "Stakeholder identification", "checklist": ["List all users", "Identify data owners"]},
                        {"name": "Dependency mapping", "checklist": ["Map integrations", "Identify data flows"]},
                        {"name": "Alternative planning", "checklist": ["Identify replacement", "Plan data migration"]}
                    ]
                },
                {
                    "name": "User Migration",
                    "description": "Move users to alternative solutions",
                    "tasks": [
                        {"name": "User communication", "checklist": ["Announce retirement", "Provide timeline"]},
                        {"name": "Training", "checklist": ["Train on alternatives", "Provide documentation"]},
                        {"name": "Access migration", "checklist": ["Provision alternative access", "Verify user migration"]}
                    ]
                },
                {
                    "name": "Data Handling",
                    "description": "Archive or migrate data",
                    "tasks": [
                        {"name": "Data classification", "checklist": ["Classify data sensitivity", "Determine retention"]},
                        {"name": "Data archival", "checklist": ["Archive required data", "Document archive location"]},
                        {"name": "Data deletion", "checklist": ["Delete non-required data", "Verify deletion"]}
                    ]
                },
                {
                    "name": "Decommissioning",
                    "description": "Remove the application",
                    "tasks": [
                        {"name": "Integration removal", "checklist": ["Disable integrations", "Update upstream systems"]},
                        {"name": "Infrastructure removal", "checklist": ["Decommission servers", "Release resources"]},
                        {"name": "Final documentation", "checklist": ["Document lessons learned", "Update asset registry"]}
                    ]
                }
            ],
            "required_roles": [
                {"role": "Project Manager", "fte_percent": 50, "duration_weeks": 12},
                {"role": "Application Owner", "fte_percent": 25, "duration_weeks": 12},
                {"role": "Data Architect", "fte_percent": 50, "duration_weeks": 4}
            ]
        })
    ]
}

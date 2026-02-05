use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Standard category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardCategory {
    Naming,
    Security,
    Architecture,
    DataManagement,
    Integration,
    Operations,
    Documentation,
}

impl StandardCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            StandardCategory::Naming => "naming",
            StandardCategory::Security => "security",
            StandardCategory::Architecture => "architecture",
            StandardCategory::DataManagement => "data_management",
            StandardCategory::Integration => "integration",
            StandardCategory::Operations => "operations",
            StandardCategory::Documentation => "documentation",
        }
    }
}

/// A standard definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Standard {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub rules: serde_json::Value,
    pub examples: Option<serde_json::Value>,
    pub version: i32,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a standard
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStandardRequest {
    pub name: String,
    pub category: StandardCategory,
    pub description: Option<String>,
    pub rules: serde_json::Value,
    pub examples: Option<serde_json::Value>,
}

/// Request to update a standard
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStandardRequest {
    pub name: Option<String>,
    pub category: Option<StandardCategory>,
    pub description: Option<String>,
    pub rules: Option<serde_json::Value>,
    pub examples: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

/// Service for managing standards
pub struct StandardsService {
    pool: PgPool,
}

impl StandardsService {
    /// Create a new standards service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List standards
    pub async fn list_standards(
        &self,
        tenant_id: Uuid,
        category: Option<String>,
        active_only: Option<String>,
    ) -> Result<Vec<Standard>> {
        let active = active_only.as_deref() == Some("true");
        let standards = sqlx::query_as::<_, Standard>(
            r#"
            SELECT id, tenant_id, name, category, description, rules, examples,
                   version, is_active, created_by, created_at, updated_at
            FROM standards
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR category = $2)
              AND ($3::bool = false OR is_active = true)
            ORDER BY category, name
            "#
        )
        .bind(tenant_id)
        .bind(&category)
        .bind(active)
        .fetch_all(&self.pool)
        .await?;

        Ok(standards)
    }

    /// Get a single standard
    pub async fn get_standard(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Standard>> {
        let standard = sqlx::query_as::<_, Standard>(
            r#"
            SELECT id, tenant_id, name, category, description, rules, examples,
                   version, is_active, created_by, created_at, updated_at
            FROM standards
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(standard)
    }

    /// Create a standard
    pub async fn create_standard(
        &self,
        tenant_id: Uuid,
        req: CreateStandardRequest,
        created_by: Option<Uuid>,
    ) -> Result<Standard> {
        let standard = sqlx::query_as::<_, Standard>(
            r#"
            INSERT INTO standards
                (tenant_id, name, category, description, rules, examples, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, name, category, description, rules, examples,
                      version, is_active, created_by, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(req.category.as_str())
        .bind(&req.description)
        .bind(&req.rules)
        .bind(&req.examples)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(standard)
    }

    /// Update a standard
    pub async fn update_standard(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateStandardRequest,
        _updated_by: Option<Uuid>,
    ) -> Result<Option<Standard>> {
        let existing = self.get_standard(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let category = req.category.map(|c| c.as_str().to_string()).unwrap_or(existing.category);
        let description = req.description.or(existing.description);
        let rules_changed = req.rules.is_some();
        let rules = req.rules.unwrap_or(existing.rules.clone());
        let examples = req.examples.or(existing.examples);
        let is_active = req.is_active.unwrap_or(existing.is_active);

        // Increment version if rules changed
        let new_version = if rules_changed {
            existing.version + 1
        } else {
            existing.version
        };

        let standard = sqlx::query_as::<_, Standard>(
            r#"
            UPDATE standards SET
                name = $1, category = $2, description = $3, rules = $4,
                examples = $5, is_active = $6, version = $7, updated_at = NOW()
            WHERE tenant_id = $8 AND id = $9
            RETURNING id, tenant_id, name, category, description, rules, examples,
                      version, is_active, created_by, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&category)
        .bind(&description)
        .bind(&rules)
        .bind(&examples)
        .bind(is_active)
        .bind(new_version)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(standard))
    }

    /// Delete a standard (soft delete via is_active)
    pub async fn delete_standard(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE standards SET is_active = false, updated_at = NOW() WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

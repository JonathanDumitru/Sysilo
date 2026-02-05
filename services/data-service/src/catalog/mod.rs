use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Types of data entities in the catalog
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entity_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Table,
    View,
    File,
    Api,
    Stream,
    Dataset,
}

/// A data entity registered in the catalog
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entity {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub entity_type: EntityType,
    pub source_system: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub schema_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Schema definition for an entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Schema {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub version: i32,
    pub fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Field definition within a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub pii: bool,
}

/// Service for managing the data catalog
pub struct CatalogService {
    pool: PgPool,
}

impl CatalogService {
    /// Create a new catalog service with database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Health check for the catalog service
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List entities with optional filtering
    pub async fn list_entities(
        &self,
        tenant_id: Uuid,
        entity_type: Option<EntityType>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Entity>, i64)> {
        let entities = if let Some(et) = entity_type {
            sqlx::query_as::<_, Entity>(
                r#"
                SELECT id, tenant_id, name, entity_type, source_system,
                       description, metadata, schema_id, created_at, updated_at
                FROM catalog_entities
                WHERE tenant_id = $1 AND entity_type = $2
                ORDER BY name
                LIMIT $3 OFFSET $4
                "#
            )
            .bind(tenant_id)
            .bind(et)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Entity>(
                r#"
                SELECT id, tenant_id, name, entity_type, source_system,
                       description, metadata, schema_id, created_at, updated_at
                FROM catalog_entities
                WHERE tenant_id = $1
                ORDER BY name
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM catalog_entities WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((entities, total.0))
    }

    /// Create a new entity in the catalog
    pub async fn create_entity(
        &self,
        tenant_id: Uuid,
        name: String,
        entity_type: EntityType,
        source_system: String,
        description: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Entity> {
        let entity = sqlx::query_as::<_, Entity>(
            r#"
            INSERT INTO catalog_entities (tenant_id, name, entity_type, source_system, description, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, entity_type, source_system,
                      description, metadata, schema_id, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(name)
        .bind(entity_type)
        .bind(source_system)
        .bind(description)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Get a single entity by ID
    pub async fn get_entity(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Entity>> {
        let entity = sqlx::query_as::<_, Entity>(
            r#"
            SELECT id, tenant_id, name, entity_type, source_system,
                   description, metadata, schema_id, created_at, updated_at
            FROM catalog_entities
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Delete an entity from the catalog
    pub async fn delete_entity(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM catalog_entities WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the schema for an entity
    pub async fn get_entity_schema(&self, tenant_id: Uuid, entity_id: Uuid) -> Result<Option<Schema>> {
        let schema = sqlx::query_as::<_, Schema>(
            r#"
            SELECT s.id, s.entity_id, s.version, s.fields, s.created_at
            FROM entity_schemas s
            JOIN catalog_entities e ON e.schema_id = s.id
            WHERE e.tenant_id = $1 AND e.id = $2
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(schema)
    }

    /// Register or update schema for an entity
    pub async fn register_schema(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        fields: Vec<SchemaField>,
    ) -> Result<Schema> {
        // Verify entity belongs to tenant
        let entity = self.get_entity(tenant_id, entity_id).await?;
        if entity.is_none() {
            anyhow::bail!("Entity not found");
        }

        let fields_json = serde_json::to_value(&fields)?;

        // Get current version
        let current_version: Option<(i32,)> = sqlx::query_as(
            "SELECT MAX(version) FROM entity_schemas WHERE entity_id = $1"
        )
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await?;

        let new_version = current_version.map(|v| v.0 + 1).unwrap_or(1);

        // Insert new schema version
        let schema = sqlx::query_as::<_, Schema>(
            r#"
            INSERT INTO entity_schemas (entity_id, version, fields)
            VALUES ($1, $2, $3)
            RETURNING id, entity_id, version, fields, created_at
            "#
        )
        .bind(entity_id)
        .bind(new_version)
        .bind(&fields_json)
        .fetch_one(&self.pool)
        .await?;

        // Update entity with new schema reference
        sqlx::query(
            "UPDATE catalog_entities SET schema_id = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(schema.id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        Ok(schema)
    }

    /// Search entities by name or description
    pub async fn search_entities(
        &self,
        tenant_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Entity>> {
        let search_pattern = format!("%{}%", query.to_lowercase());

        let entities = sqlx::query_as::<_, Entity>(
            r#"
            SELECT id, tenant_id, name, entity_type, source_system,
                   description, metadata, schema_id, created_at, updated_at
            FROM catalog_entities
            WHERE tenant_id = $1
              AND (LOWER(name) LIKE $2 OR LOWER(description) LIKE $2)
            ORDER BY name
            LIMIT $3
            "#
        )
        .bind(tenant_id)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }
}

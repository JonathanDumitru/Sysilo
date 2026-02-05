use anyhow::Result;
use neo4rs::{Graph, query};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Types of technology assets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Application,
    Service,
    Database,
    Api,
    DataStore,
    Integration,
    Infrastructure,
    Platform,
    Tool,
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Application => "Application",
            AssetType::Service => "Service",
            AssetType::Database => "Database",
            AssetType::Api => "Api",
            AssetType::DataStore => "DataStore",
            AssetType::Integration => "Integration",
            AssetType::Infrastructure => "Infrastructure",
            AssetType::Platform => "Platform",
            AssetType::Tool => "Tool",
        }
    }
}

/// Lifecycle status of an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Active,
    Deprecated,
    Sunset,
    Planned,
    UnderReview,
}

/// A technology asset in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub status: AssetStatus,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub team: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub documentation_url: Option<String>,
    pub repository_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new asset
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub asset_type: AssetType,
    pub status: Option<AssetStatus>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub team: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub documentation_url: Option<String>,
    pub repository_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

/// Request to update an asset
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub asset_type: Option<AssetType>,
    pub status: Option<AssetStatus>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub team: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub documentation_url: Option<String>,
    pub repository_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

/// Service for managing technology assets
pub struct AssetService {
    pg_pool: PgPool,
    neo4j: Graph,
}

impl AssetService {
    /// Create a new asset service
    pub async fn new(
        database_url: &str,
        neo4j_uri: &str,
        neo4j_user: &str,
        neo4j_password: &str,
    ) -> Result<Self> {
        let pg_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let neo4j = Graph::new(neo4j_uri, neo4j_user, neo4j_password).await?;

        Ok(Self { pg_pool, neo4j })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pg_pool)
            .await?;

        self.neo4j.run(query("RETURN 1")).await?;

        Ok(())
    }

    /// Create a new asset
    pub async fn create_asset(
        &self,
        tenant_id: Uuid,
        req: CreateAssetRequest,
    ) -> Result<Asset> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let status = req.status.unwrap_or(AssetStatus::Active);
        let tags = req.tags.unwrap_or_default();

        // Create in Neo4j first
        let asset_type_str = req.asset_type.as_str();
        let status_str = format!("{:?}", status).to_lowercase();
        let tags_json = serde_json::to_string(&tags)?;

        self.neo4j.run(
            query(
                r#"
                CREATE (a:Asset {
                    id: $id,
                    tenant_id: $tenant_id,
                    name: $name,
                    asset_type: $asset_type,
                    status: $status,
                    description: $description,
                    owner: $owner,
                    team: $team,
                    vendor: $vendor,
                    version: $version,
                    tags: $tags,
                    created_at: $created_at,
                    updated_at: $updated_at
                })
                "#
            )
            .param("id", id.to_string())
            .param("tenant_id", tenant_id.to_string())
            .param("name", req.name.clone())
            .param("asset_type", asset_type_str)
            .param("status", status_str)
            .param("description", req.description.clone().unwrap_or_default())
            .param("owner", req.owner.clone().unwrap_or_default())
            .param("team", req.team.clone().unwrap_or_default())
            .param("vendor", req.vendor.clone().unwrap_or_default())
            .param("version", req.version.clone().unwrap_or_default())
            .param("tags", tags_json)
            .param("created_at", now.to_rfc3339())
            .param("updated_at", now.to_rfc3339())
        ).await?;

        // Also store in PostgreSQL for tenant-aware queries
        sqlx::query(
            r#"
            INSERT INTO assets (id, tenant_id, name, asset_type, status, description,
                               owner, team, vendor, version, documentation_url,
                               repository_url, metadata, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(asset_type_str)
        .bind(&status_str)
        .bind(&req.description)
        .bind(&req.owner)
        .bind(&req.team)
        .bind(&req.vendor)
        .bind(&req.version)
        .bind(&req.documentation_url)
        .bind(&req.repository_url)
        .bind(&req.metadata)
        .bind(&tags)
        .execute(&self.pg_pool)
        .await?;

        Ok(Asset {
            id,
            tenant_id,
            name: req.name,
            asset_type: req.asset_type,
            status,
            description: req.description,
            owner: req.owner,
            team: req.team,
            vendor: req.vendor,
            version: req.version,
            documentation_url: req.documentation_url,
            repository_url: req.repository_url,
            metadata: req.metadata,
            tags,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get an asset by ID
    pub async fn get_asset(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Asset>> {
        let row = sqlx::query_as::<_, AssetRow>(
            r#"
            SELECT id, tenant_id, name, asset_type, status, description,
                   owner, team, vendor, version, documentation_url,
                   repository_url, metadata, tags, created_at, updated_at
            FROM assets
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(row.map(|r| r.into_asset()))
    }

    /// List assets with optional filtering
    pub async fn list_assets(
        &self,
        tenant_id: Uuid,
        asset_type: Option<String>,
        status: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Asset>, i64)> {
        let rows = sqlx::query_as::<_, AssetRow>(
            r#"
            SELECT id, tenant_id, name, asset_type, status, description,
                   owner, team, vendor, version, documentation_url,
                   repository_url, metadata, tags, created_at, updated_at
            FROM assets
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR asset_type = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY name
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(tenant_id)
        .bind(asset_type)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pg_pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM assets WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pg_pool)
        .await?;

        let assets = rows.into_iter().map(|r| r.into_asset()).collect();

        Ok((assets, total.0))
    }

    /// Search assets by name or description
    pub async fn search_assets(
        &self,
        tenant_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Asset>> {
        let pattern = format!("%{}%", query.to_lowercase());

        let rows = sqlx::query_as::<_, AssetRow>(
            r#"
            SELECT id, tenant_id, name, asset_type, status, description,
                   owner, team, vendor, version, documentation_url,
                   repository_url, metadata, tags, created_at, updated_at
            FROM assets
            WHERE tenant_id = $1
              AND (LOWER(name) LIKE $2 OR LOWER(description) LIKE $2)
            ORDER BY name
            LIMIT $3
            "#
        )
        .bind(tenant_id)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_asset()).collect())
    }

    /// Update an asset
    pub async fn update_asset(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateAssetRequest,
    ) -> Result<Option<Asset>> {
        // Get existing asset
        let existing = self.get_asset(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let asset_type = req.asset_type.unwrap_or(existing.asset_type);
        let status = req.status.unwrap_or(existing.status);
        let description = req.description.or(existing.description);
        let owner = req.owner.or(existing.owner);
        let team = req.team.or(existing.team);
        let vendor = req.vendor.or(existing.vendor);
        let version = req.version.or(existing.version);
        let documentation_url = req.documentation_url.or(existing.documentation_url);
        let repository_url = req.repository_url.or(existing.repository_url);
        let metadata = req.metadata.or(existing.metadata);
        let tags = req.tags.unwrap_or(existing.tags);
        let now = Utc::now();

        let asset_type_str = asset_type.as_str();
        let status_str = format!("{:?}", status).to_lowercase();

        // Update PostgreSQL
        sqlx::query(
            r#"
            UPDATE assets SET
                name = $1, asset_type = $2, status = $3, description = $4,
                owner = $5, team = $6, vendor = $7, version = $8,
                documentation_url = $9, repository_url = $10, metadata = $11,
                tags = $12, updated_at = $13
            WHERE tenant_id = $14 AND id = $15
            "#
        )
        .bind(&name)
        .bind(asset_type_str)
        .bind(&status_str)
        .bind(&description)
        .bind(&owner)
        .bind(&team)
        .bind(&vendor)
        .bind(&version)
        .bind(&documentation_url)
        .bind(&repository_url)
        .bind(&metadata)
        .bind(&tags)
        .bind(now)
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pg_pool)
        .await?;

        // Update Neo4j
        let tags_json = serde_json::to_string(&tags)?;
        self.neo4j.run(
            query(
                r#"
                MATCH (a:Asset {id: $id, tenant_id: $tenant_id})
                SET a.name = $name,
                    a.asset_type = $asset_type,
                    a.status = $status,
                    a.description = $description,
                    a.owner = $owner,
                    a.team = $team,
                    a.vendor = $vendor,
                    a.version = $version,
                    a.tags = $tags,
                    a.updated_at = $updated_at
                "#
            )
            .param("id", id.to_string())
            .param("tenant_id", tenant_id.to_string())
            .param("name", name.clone())
            .param("asset_type", asset_type_str)
            .param("status", status_str.clone())
            .param("description", description.clone().unwrap_or_default())
            .param("owner", owner.clone().unwrap_or_default())
            .param("team", team.clone().unwrap_or_default())
            .param("vendor", vendor.clone().unwrap_or_default())
            .param("version", version.clone().unwrap_or_default())
            .param("tags", tags_json)
            .param("updated_at", now.to_rfc3339())
        ).await?;

        Ok(Some(Asset {
            id,
            tenant_id,
            name,
            asset_type,
            status,
            description,
            owner,
            team,
            vendor,
            version,
            documentation_url,
            repository_url,
            metadata,
            tags,
            created_at: existing.created_at,
            updated_at: now,
        }))
    }

    /// Delete an asset
    pub async fn delete_asset(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        // Delete from Neo4j (including relationships)
        self.neo4j.run(
            query(
                r#"
                MATCH (a:Asset {id: $id, tenant_id: $tenant_id})
                DETACH DELETE a
                "#
            )
            .param("id", id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        // Delete from PostgreSQL
        let result = sqlx::query(
            "DELETE FROM assets WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pg_pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// Database row type for SQLx
#[derive(sqlx::FromRow)]
struct AssetRow {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    asset_type: String,
    status: String,
    description: Option<String>,
    owner: Option<String>,
    team: Option<String>,
    vendor: Option<String>,
    version: Option<String>,
    documentation_url: Option<String>,
    repository_url: Option<String>,
    metadata: Option<serde_json::Value>,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AssetRow {
    fn into_asset(self) -> Asset {
        Asset {
            id: self.id,
            tenant_id: self.tenant_id,
            name: self.name,
            asset_type: parse_asset_type(&self.asset_type),
            status: parse_asset_status(&self.status),
            description: self.description,
            owner: self.owner,
            team: self.team,
            vendor: self.vendor,
            version: self.version,
            documentation_url: self.documentation_url,
            repository_url: self.repository_url,
            metadata: self.metadata,
            tags: self.tags,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

fn parse_asset_type(s: &str) -> AssetType {
    match s.to_lowercase().as_str() {
        "application" => AssetType::Application,
        "service" => AssetType::Service,
        "database" => AssetType::Database,
        "api" => AssetType::Api,
        "datastore" => AssetType::DataStore,
        "integration" => AssetType::Integration,
        "infrastructure" => AssetType::Infrastructure,
        "platform" => AssetType::Platform,
        "tool" => AssetType::Tool,
        _ => AssetType::Application,
    }
}

fn parse_asset_status(s: &str) -> AssetStatus {
    match s.to_lowercase().as_str() {
        "active" => AssetStatus::Active,
        "deprecated" => AssetStatus::Deprecated,
        "sunset" => AssetStatus::Sunset,
        "planned" => AssetStatus::Planned,
        "underreview" | "under_review" => AssetStatus::UnderReview,
        _ => AssetStatus::Active,
    }
}

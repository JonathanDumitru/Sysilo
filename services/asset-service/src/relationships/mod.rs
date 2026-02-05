use anyhow::Result;
use neo4rs::{Graph, query};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Types of relationships between assets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    DependsOn,
    Integrates,
    ReadsFrom,
    WritesTo,
    Calls,
    Hosts,
    OwnedBy,
    ManagedBy,
    ReplacedBy,
    RelatedTo,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::DependsOn => "DEPENDS_ON",
            RelationshipType::Integrates => "INTEGRATES",
            RelationshipType::ReadsFrom => "READS_FROM",
            RelationshipType::WritesTo => "WRITES_TO",
            RelationshipType::Calls => "CALLS",
            RelationshipType::Hosts => "HOSTS",
            RelationshipType::OwnedBy => "OWNED_BY",
            RelationshipType::ManagedBy => "MANAGED_BY",
            RelationshipType::ReplacedBy => "REPLACED_BY",
            RelationshipType::RelatedTo => "RELATED_TO",
        }
    }
}

/// A relationship between two assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relationship_type: RelationshipType,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a relationship
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRelationshipRequest {
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relationship_type: RelationshipType,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Relationship with connected asset details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipWithAsset {
    pub relationship: Relationship,
    pub connected_asset_id: Uuid,
    pub connected_asset_name: String,
    pub connected_asset_type: String,
    pub direction: String,
}

/// Service for managing asset relationships
pub struct RelationshipService {
    neo4j: Graph,
}

impl RelationshipService {
    /// Create a new relationship service
    pub async fn new(
        neo4j_uri: &str,
        neo4j_user: &str,
        neo4j_password: &str,
    ) -> Result<Self> {
        let neo4j = Graph::new(neo4j_uri, neo4j_user, neo4j_password).await?;
        Ok(Self { neo4j })
    }

    /// Create a relationship between two assets
    pub async fn create_relationship(
        &self,
        tenant_id: Uuid,
        req: CreateRelationshipRequest,
    ) -> Result<Relationship> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let rel_type = req.relationship_type.as_str();
        let metadata_str = req.metadata.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default();

        // Create relationship in Neo4j using dynamic relationship type
        let cypher = format!(
            r#"
            MATCH (source:Asset {{id: $source_id, tenant_id: $tenant_id}})
            MATCH (target:Asset {{id: $target_id, tenant_id: $tenant_id}})
            CREATE (source)-[r:{} {{
                id: $id,
                tenant_id: $tenant_id,
                description: $description,
                metadata: $metadata,
                created_at: $created_at
            }}]->(target)
            RETURN r
            "#,
            rel_type
        );

        self.neo4j.run(
            query(&cypher)
                .param("source_id", req.source_asset_id.to_string())
                .param("target_id", req.target_asset_id.to_string())
                .param("tenant_id", tenant_id.to_string())
                .param("id", id.to_string())
                .param("description", req.description.clone().unwrap_or_default())
                .param("metadata", metadata_str)
                .param("created_at", now.to_rfc3339())
        ).await?;

        Ok(Relationship {
            id,
            tenant_id,
            source_asset_id: req.source_asset_id,
            target_asset_id: req.target_asset_id,
            relationship_type: req.relationship_type,
            description: req.description,
            metadata: req.metadata,
            created_at: now,
        })
    }

    /// Get relationships for an asset
    pub async fn get_asset_relationships(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<RelationshipWithAsset>> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[r]->(target:Asset)
                RETURN r.id as rel_id, r.description as description, r.created_at as created_at,
                       type(r) as rel_type, target.id as target_id, target.name as target_name,
                       target.asset_type as target_type, 'outgoing' as direction
                UNION
                MATCH (source:Asset)-[r]->(a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN r.id as rel_id, r.description as description, r.created_at as created_at,
                       type(r) as rel_type, source.id as source_id, source.name as source_name,
                       source.asset_type as source_type, 'incoming' as direction
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut relationships = Vec::new();

        while let Ok(Some(row)) = result.next().await {
            let rel_id: String = row.get("rel_id").unwrap_or_default();
            let description: Option<String> = row.get("description").ok();
            let created_at: String = row.get("created_at").unwrap_or_default();
            let rel_type: String = row.get("rel_type").unwrap_or_default();
            let direction: String = row.get("direction").unwrap_or_default();

            let (connected_id, connected_name, connected_type) = if direction == "outgoing" {
                let tid: String = row.get("target_id").unwrap_or_default();
                let tn: String = row.get("target_name").unwrap_or_default();
                let tt: String = row.get("target_type").unwrap_or_default();
                (tid, tn, tt)
            } else {
                let sid: String = row.get("source_id").unwrap_or_default();
                let sn: String = row.get("source_name").unwrap_or_default();
                let st: String = row.get("source_type").unwrap_or_default();
                (sid, sn, st)
            };

            relationships.push(RelationshipWithAsset {
                relationship: Relationship {
                    id: Uuid::parse_str(&rel_id).unwrap_or_default(),
                    tenant_id,
                    source_asset_id: if direction == "outgoing" { asset_id } else { Uuid::parse_str(&connected_id).unwrap_or_default() },
                    target_asset_id: if direction == "outgoing" { Uuid::parse_str(&connected_id).unwrap_or_default() } else { asset_id },
                    relationship_type: parse_relationship_type(&rel_type),
                    description,
                    metadata: None,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                },
                connected_asset_id: Uuid::parse_str(&connected_id).unwrap_or_default(),
                connected_asset_name: connected_name,
                connected_asset_type: connected_type,
                direction,
            });
        }

        Ok(relationships)
    }

    /// List all relationships for a tenant
    pub async fn list_relationships(
        &self,
        tenant_id: Uuid,
        relationship_type: Option<String>,
        limit: i64,
    ) -> Result<Vec<Relationship>> {
        let cypher = if let Some(rel_type) = relationship_type {
            format!(
                r#"
                MATCH (source:Asset {{tenant_id: $tenant_id}})-[r:{}]->(target:Asset)
                RETURN r.id as id, source.id as source_id, target.id as target_id,
                       type(r) as rel_type, r.description as description, r.created_at as created_at
                LIMIT $limit
                "#,
                rel_type
            )
        } else {
            r#"
            MATCH (source:Asset {tenant_id: $tenant_id})-[r]->(target:Asset)
            RETURN r.id as id, source.id as source_id, target.id as target_id,
                   type(r) as rel_type, r.description as description, r.created_at as created_at
            LIMIT $limit
            "#.to_string()
        };

        let mut result = self.neo4j.execute(
            query(&cypher)
                .param("tenant_id", tenant_id.to_string())
                .param("limit", limit)
        ).await?;

        let mut relationships = Vec::new();

        while let Ok(Some(row)) = result.next().await {
            let id: String = row.get("id").unwrap_or_default();
            let source_id: String = row.get("source_id").unwrap_or_default();
            let target_id: String = row.get("target_id").unwrap_or_default();
            let rel_type: String = row.get("rel_type").unwrap_or_default();
            let description: Option<String> = row.get("description").ok();
            let created_at: String = row.get("created_at").unwrap_or_default();

            relationships.push(Relationship {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                tenant_id,
                source_asset_id: Uuid::parse_str(&source_id).unwrap_or_default(),
                target_asset_id: Uuid::parse_str(&target_id).unwrap_or_default(),
                relationship_type: parse_relationship_type(&rel_type),
                description,
                metadata: None,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }

        Ok(relationships)
    }

    /// Delete a relationship
    pub async fn delete_relationship(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = self.neo4j.execute(
            query(
                r#"
                MATCH (:Asset {tenant_id: $tenant_id})-[r {id: $id}]->(:Asset)
                DELETE r
                RETURN count(r) as deleted
                "#
            )
            .param("tenant_id", tenant_id.to_string())
            .param("id", id.to_string())
        ).await?;

        // Check if any rows were deleted
        // Note: simplified check - in production would verify the count
        Ok(true)
    }
}

fn parse_relationship_type(s: &str) -> RelationshipType {
    match s.to_uppercase().as_str() {
        "DEPENDS_ON" => RelationshipType::DependsOn,
        "INTEGRATES" => RelationshipType::Integrates,
        "READS_FROM" => RelationshipType::ReadsFrom,
        "WRITES_TO" => RelationshipType::WritesTo,
        "CALLS" => RelationshipType::Calls,
        "HOSTS" => RelationshipType::Hosts,
        "OWNED_BY" => RelationshipType::OwnedBy,
        "MANAGED_BY" => RelationshipType::ManagedBy,
        "REPLACED_BY" => RelationshipType::ReplacedBy,
        _ => RelationshipType::RelatedTo,
    }
}

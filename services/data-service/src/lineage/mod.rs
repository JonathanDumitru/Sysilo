use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

use crate::catalog::Entity;

/// An edge in the lineage graph representing data flow
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LineageEdge {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub transformation_type: String,
    pub transformation_logic: Option<String>,
    pub integration_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// A node in the lineage graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub entity_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub source_system: String,
    pub depth: i32,
}

/// Complete lineage graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    pub root_entity_id: Uuid,
    pub nodes: Vec<LineageNode>,
    pub edges: Vec<LineageGraphEdge>,
}

/// Simplified edge for graph visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub transformation_type: String,
}

/// Service for managing data lineage
pub struct LineageService {
    pool: PgPool,
}

impl LineageService {
    /// Create a new lineage service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Add a lineage edge between two entities
    pub async fn add_edge(
        &self,
        tenant_id: Uuid,
        source_entity_id: Uuid,
        target_entity_id: Uuid,
        transformation_type: String,
        transformation_logic: Option<String>,
        integration_id: Option<Uuid>,
    ) -> Result<LineageEdge> {
        let edge = sqlx::query_as::<_, LineageEdge>(
            r#"
            INSERT INTO lineage_edges
                (tenant_id, source_entity_id, target_entity_id,
                 transformation_type, transformation_logic, integration_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, source_entity_id, target_entity_id,
                      transformation_type, transformation_logic, integration_id, created_at
            "#
        )
        .bind(tenant_id)
        .bind(source_entity_id)
        .bind(target_entity_id)
        .bind(transformation_type)
        .bind(transformation_logic)
        .bind(integration_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(edge)
    }

    /// Get lineage graph for an entity
    pub async fn get_lineage(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        max_depth: i32,
        direction: &str,
    ) -> Result<LineageGraph> {
        let mut nodes: HashMap<Uuid, LineageNode> = HashMap::new();
        let mut edges: Vec<LineageGraphEdge> = Vec::new();
        let mut visited: HashSet<Uuid> = HashSet::new();

        // Get root entity info
        let root: (Uuid, String, String, String) = sqlx::query_as(
            r#"
            SELECT id, name, entity_type::text, source_system
            FROM catalog_entities
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_one(&self.pool)
        .await?;

        nodes.insert(entity_id, LineageNode {
            entity_id,
            name: root.1,
            entity_type: root.2,
            source_system: root.3,
            depth: 0,
        });
        visited.insert(entity_id);

        // Traverse upstream (sources)
        if direction == "upstream" || direction == "both" {
            self.traverse_upstream(tenant_id, entity_id, 1, max_depth, &mut nodes, &mut edges, &mut visited).await?;
        }

        // Traverse downstream (targets)
        if direction == "downstream" || direction == "both" {
            self.traverse_downstream(tenant_id, entity_id, 1, max_depth, &mut nodes, &mut edges, &mut visited).await?;
        }

        Ok(LineageGraph {
            root_entity_id: entity_id,
            nodes: nodes.into_values().collect(),
            edges,
        })
    }

    /// Recursively traverse upstream lineage
    async fn traverse_upstream(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        current_depth: i32,
        max_depth: i32,
        nodes: &mut HashMap<Uuid, LineageNode>,
        edges: &mut Vec<LineageGraphEdge>,
        visited: &mut HashSet<Uuid>,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        let upstream_edges: Vec<(Uuid, Uuid, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT le.source_entity_id, le.target_entity_id, le.transformation_type,
                   ce.name, ce.entity_type::text, ce.source_system
            FROM lineage_edges le
            JOIN catalog_entities ce ON ce.id = le.source_entity_id
            WHERE le.tenant_id = $1 AND le.target_entity_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        for (source_id, target_id, transform_type, name, entity_type, source_system) in upstream_edges {
            edges.push(LineageGraphEdge {
                source: source_id,
                target: target_id,
                transformation_type: transform_type,
            });

            if !visited.contains(&source_id) {
                visited.insert(source_id);
                nodes.insert(source_id, LineageNode {
                    entity_id: source_id,
                    name,
                    entity_type,
                    source_system,
                    depth: -current_depth,
                });

                Box::pin(self.traverse_upstream(
                    tenant_id, source_id, current_depth + 1, max_depth, nodes, edges, visited
                )).await?;
            }
        }

        Ok(())
    }

    /// Recursively traverse downstream lineage
    async fn traverse_downstream(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        current_depth: i32,
        max_depth: i32,
        nodes: &mut HashMap<Uuid, LineageNode>,
        edges: &mut Vec<LineageGraphEdge>,
        visited: &mut HashSet<Uuid>,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        let downstream_edges: Vec<(Uuid, Uuid, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT le.source_entity_id, le.target_entity_id, le.transformation_type,
                   ce.name, ce.entity_type::text, ce.source_system
            FROM lineage_edges le
            JOIN catalog_entities ce ON ce.id = le.target_entity_id
            WHERE le.tenant_id = $1 AND le.source_entity_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        for (source_id, target_id, transform_type, name, entity_type, source_system) in downstream_edges {
            edges.push(LineageGraphEdge {
                source: source_id,
                target: target_id,
                transformation_type: transform_type,
            });

            if !visited.contains(&target_id) {
                visited.insert(target_id);
                nodes.insert(target_id, LineageNode {
                    entity_id: target_id,
                    name,
                    entity_type,
                    source_system,
                    depth: current_depth,
                });

                Box::pin(self.traverse_downstream(
                    tenant_id, target_id, current_depth + 1, max_depth, nodes, edges, visited
                )).await?;
            }
        }

        Ok(())
    }

    /// Analyze downstream impact of changes to an entity
    pub async fn get_impact_analysis(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
    ) -> Result<(Vec<Entity>, Vec<Uuid>)> {
        // Get all downstream entities (unlimited depth)
        let graph = self.get_lineage(tenant_id, entity_id, 100, "downstream").await?;

        let downstream_ids: Vec<Uuid> = graph.nodes
            .iter()
            .filter(|n| n.depth > 0)
            .map(|n| n.entity_id)
            .collect();

        // Fetch full entity details
        let entities = if downstream_ids.is_empty() {
            vec![]
        } else {
            sqlx::query_as::<_, Entity>(
                r#"
                SELECT id, tenant_id, name, entity_type, source_system,
                       description, metadata, schema_id, created_at, updated_at
                FROM catalog_entities
                WHERE tenant_id = $1 AND id = ANY($2)
                "#
            )
            .bind(tenant_id)
            .bind(&downstream_ids)
            .fetch_all(&self.pool)
            .await?
        };

        // Get unique integration IDs from edges
        let integration_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT integration_id
            FROM lineage_edges
            WHERE tenant_id = $1
              AND source_entity_id = $2
              AND integration_id IS NOT NULL
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok((entities, integration_ids))
    }

    /// Delete lineage edges for an entity
    pub async fn delete_entity_lineage(&self, tenant_id: Uuid, entity_id: Uuid) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM lineage_edges
            WHERE tenant_id = $1
              AND (source_entity_id = $2 OR target_entity_id = $2)
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

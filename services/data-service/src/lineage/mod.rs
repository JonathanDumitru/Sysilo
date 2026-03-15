use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use neo4rs::{Graph, query};
use tracing::{info, error};

/// Entity types that can participate in lineage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LineageEntityType {
    Dataset,
    Table,
    View,
    Api,
    File,
    Stream,
}

impl std::fmt::Display for LineageEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dataset => write!(f, "dataset"),
            Self::Table => write!(f, "table"),
            Self::View => write!(f, "view"),
            Self::Api => write!(f, "api"),
            Self::File => write!(f, "file"),
            Self::Stream => write!(f, "stream"),
        }
    }
}

impl LineageEntityType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dataset" => Self::Dataset,
            "table" => Self::Table,
            "view" => Self::View,
            "api" => Self::Api,
            "file" => Self::File,
            "stream" => Self::Stream,
            _ => Self::Dataset,
        }
    }
}

/// Types of edges in the lineage graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LineageEdgeType {
    DerivesFrom,
    TransformsTo,
    CopiesTo,
    AggregatesFrom,
    JoinsWith,
}

impl std::fmt::Display for LineageEdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DerivesFrom => write!(f, "derives_from"),
            Self::TransformsTo => write!(f, "transforms_to"),
            Self::CopiesTo => write!(f, "copies_to"),
            Self::AggregatesFrom => write!(f, "aggregates_from"),
            Self::JoinsWith => write!(f, "joins_with"),
        }
    }
}

impl LineageEdgeType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "derives_from" => Self::DerivesFrom,
            "transforms_to" => Self::TransformsTo,
            "copies_to" => Self::CopiesTo,
            "aggregates_from" => Self::AggregatesFrom,
            "joins_with" => Self::JoinsWith,
            _ => Self::DerivesFrom,
        }
    }

    /// Returns the Cypher relationship type string
    pub fn to_cypher_rel(&self) -> &str {
        match self {
            Self::DerivesFrom => "DERIVES_FROM",
            Self::TransformsTo => "TRANSFORMS_TO",
            Self::CopiesTo => "COPIES_TO",
            Self::AggregatesFrom => "AGGREGATES_FROM",
            Self::JoinsWith => "JOINS_WITH",
        }
    }
}

/// Direction for lineage traversal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LineageDirection {
    Upstream,
    Downstream,
    Both,
}

impl LineageDirection {
    pub fn from_str(s: &str) -> Self {
        match s {
            "upstream" => Self::Upstream,
            "downstream" => Self::Downstream,
            "both" => Self::Both,
            _ => Self::Both,
        }
    }
}

/// A node in the lineage graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: String,
    pub tenant_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub name: String,
    pub system: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

/// An edge in the lineage graph representing data flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub transformation: Option<String>,
    pub integration_id: Option<String>,
    pub created_at: String,
}

/// Complete lineage graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    pub nodes: Vec<LineageNode>,
    pub edges: Vec<LineageEdge>,
}

/// Query parameters for lineage traversal
#[derive(Debug, Clone, Deserialize)]
pub struct LineageQueryParams {
    pub entity_id: String,
    pub direction: Option<String>,
    pub depth: Option<i64>,
    pub entity_type: Option<String>,
}

/// Service for managing data lineage using Neo4j
pub struct LineageService {
    graph: Arc<Graph>,
}

impl LineageService {
    /// Create a new lineage service with Neo4j connection
    pub async fn new(neo4j_uri: &str, neo4j_user: &str, neo4j_password: &str) -> Result<Self> {
        let graph = Graph::new(neo4j_uri, neo4j_user, neo4j_password).await?;

        // Create indexes for performance
        let index_queries = vec![
            "CREATE INDEX IF NOT EXISTS FOR (n:LineageNode) ON (n.tenant_id, n.entity_id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:LineageNode) ON (n.tenant_id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:LineageNode) ON (n.entity_id)",
        ];

        for q in index_queries {
            if let Err(e) = graph.run(query(q)).await {
                // Indexes may already exist, log but don't fail
                info!("Index creation note: {}", e);
            }
        }

        info!("LineageService connected to Neo4j");

        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    /// Record a lineage relationship between two entities.
    /// Creates nodes if they don't exist (MERGE for idempotency) and creates the edge.
    pub async fn record_lineage(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        edge_type: LineageEdgeType,
        transformation: Option<String>,
        integration_id: Option<Uuid>,
    ) -> Result<LineageEdge> {
        let edge_id = Uuid::new_v4().to_string();
        let tenant = tenant_id.to_string();
        let source = source_id.to_string();
        let target = target_id.to_string();
        let edge_type_str = edge_type.to_string();
        let rel_type = edge_type.to_cypher_rel();
        let now = Utc::now().to_rfc3339();
        let transform = transformation.clone().unwrap_or_default();
        let int_id = integration_id.map(|id| id.to_string()).unwrap_or_default();

        // Use a single Cypher query with MERGE for idempotency
        let cypher = format!(
            r#"
            MERGE (source:LineageNode {{tenant_id: $tenant_id, entity_id: $source_id}})
            ON CREATE SET source.id = $source_node_id, source.name = $source_id,
                          source.entity_type = 'dataset', source.system = 'unknown',
                          source.created_at = $now
            MERGE (target:LineageNode {{tenant_id: $tenant_id, entity_id: $target_id}})
            ON CREATE SET target.id = $target_node_id, target.name = $target_id,
                          target.entity_type = 'dataset', target.system = 'unknown',
                          target.created_at = $now
            MERGE (source)-[r:{rel_type} {{tenant_id: $tenant_id}}]->(target)
            ON CREATE SET r.id = $edge_id, r.edge_type = $edge_type,
                          r.transformation = $transformation,
                          r.integration_id = $integration_id,
                          r.created_at = $now
            RETURN r.id AS id, $source_id AS source_id, $target_id AS target_id,
                   r.edge_type AS edge_type, r.transformation AS transformation,
                   r.integration_id AS integration_id, r.created_at AS created_at
            "#
        );

        let mut result = self.graph.execute(
            query(&cypher)
                .param("tenant_id", tenant.as_str())
                .param("source_id", source.as_str())
                .param("target_id", target.as_str())
                .param("source_node_id", Uuid::new_v4().to_string().as_str())
                .param("target_node_id", Uuid::new_v4().to_string().as_str())
                .param("edge_id", edge_id.as_str())
                .param("edge_type", edge_type_str.as_str())
                .param("transformation", transform.as_str())
                .param("integration_id", int_id.as_str())
                .param("now", now.as_str())
        ).await?;

        if let Some(row) = result.next().await? {
            Ok(LineageEdge {
                id: row.get::<String>("id").unwrap_or(edge_id),
                source_id: row.get::<String>("source_id").unwrap_or(source),
                target_id: row.get::<String>("target_id").unwrap_or(target),
                edge_type: row.get::<String>("edge_type").unwrap_or(edge_type_str),
                transformation: {
                    let t = row.get::<String>("transformation").unwrap_or_default();
                    if t.is_empty() { None } else { Some(t) }
                },
                integration_id: {
                    let i = row.get::<String>("integration_id").unwrap_or_default();
                    if i.is_empty() { None } else { Some(i) }
                },
                created_at: row.get::<String>("created_at").unwrap_or(now),
            })
        } else {
            // Fallback: return constructed edge
            Ok(LineageEdge {
                id: edge_id,
                source_id: source,
                target_id: target,
                edge_type: edge_type_str,
                transformation,
                integration_id: integration_id.map(|id| id.to_string()),
                created_at: now,
            })
        }
    }

    /// Get lineage graph for an entity, traversing upstream/downstream to specified depth
    pub async fn get_lineage(
        &self,
        tenant_id: Uuid,
        params: LineageQueryParams,
    ) -> Result<LineageGraph> {
        let tenant = tenant_id.to_string();
        let direction = LineageDirection::from_str(
            params.direction.as_deref().unwrap_or("both")
        );
        let depth = params.depth.unwrap_or(3);
        let entity_type_filter = params.entity_type.clone();

        let mut all_nodes: HashMap<String, LineageNode> = HashMap::new();
        let mut all_edges: Vec<LineageEdge> = Vec::new();

        // Build entity type filter clause
        let type_filter = if let Some(ref et) = entity_type_filter {
            format!("AND n.entity_type = '{}'", et)
        } else {
            String::new()
        };

        // Upstream traversal
        if direction == LineageDirection::Upstream || direction == LineageDirection::Both {
            let cypher = format!(
                r#"
                MATCH (start:LineageNode {{tenant_id: $tenant_id, entity_id: $entity_id}})
                MATCH path = (n:LineageNode)-[r*1..{depth}]->(start)
                WHERE ALL(node IN nodes(path) WHERE node.tenant_id = $tenant_id)
                {type_filter}
                UNWIND nodes(path) AS node
                UNWIND relationships(path) AS rel
                RETURN DISTINCT
                    node.id AS node_id, node.tenant_id AS node_tenant_id,
                    node.entity_id AS node_entity_id, node.entity_type AS node_entity_type,
                    node.name AS node_name, node.system AS node_system,
                    node.created_at AS node_created_at,
                    rel.id AS rel_id, startNode(rel).entity_id AS rel_source_id,
                    endNode(rel).entity_id AS rel_target_id,
                    rel.edge_type AS rel_edge_type, rel.transformation AS rel_transformation,
                    rel.integration_id AS rel_integration_id,
                    rel.created_at AS rel_created_at
                "#
            );

            self.collect_graph_results(
                &cypher, &tenant, &params.entity_id,
                &mut all_nodes, &mut all_edges
            ).await?;
        }

        // Downstream traversal
        if direction == LineageDirection::Downstream || direction == LineageDirection::Both {
            let cypher = format!(
                r#"
                MATCH (start:LineageNode {{tenant_id: $tenant_id, entity_id: $entity_id}})
                MATCH path = (start)-[r*1..{depth}]->(n:LineageNode)
                WHERE ALL(node IN nodes(path) WHERE node.tenant_id = $tenant_id)
                {type_filter}
                UNWIND nodes(path) AS node
                UNWIND relationships(path) AS rel
                RETURN DISTINCT
                    node.id AS node_id, node.tenant_id AS node_tenant_id,
                    node.entity_id AS node_entity_id, node.entity_type AS node_entity_type,
                    node.name AS node_name, node.system AS node_system,
                    node.created_at AS node_created_at,
                    rel.id AS rel_id, startNode(rel).entity_id AS rel_source_id,
                    endNode(rel).entity_id AS rel_target_id,
                    rel.edge_type AS rel_edge_type, rel.transformation AS rel_transformation,
                    rel.integration_id AS rel_integration_id,
                    rel.created_at AS rel_created_at
                "#
            );

            self.collect_graph_results(
                &cypher, &tenant, &params.entity_id,
                &mut all_nodes, &mut all_edges
            ).await?;
        }

        // Also include the starting node
        let start_cypher = r#"
            MATCH (n:LineageNode {tenant_id: $tenant_id, entity_id: $entity_id})
            RETURN n.id AS id, n.tenant_id AS tenant_id, n.entity_id AS entity_id,
                   n.entity_type AS entity_type, n.name AS name, n.system AS system,
                   n.created_at AS created_at
        "#;

        let mut start_result = self.graph.execute(
            query(start_cypher)
                .param("tenant_id", tenant.as_str())
                .param("entity_id", params.entity_id.as_str())
        ).await?;

        if let Some(row) = start_result.next().await? {
            let entity_id = row.get::<String>("entity_id").unwrap_or_default();
            all_nodes.entry(entity_id.clone()).or_insert(LineageNode {
                id: row.get::<String>("id").unwrap_or_default(),
                tenant_id: row.get::<String>("tenant_id").unwrap_or_default(),
                entity_id,
                entity_type: row.get::<String>("entity_type").unwrap_or_default(),
                name: row.get::<String>("name").unwrap_or_default(),
                system: row.get::<String>("system").unwrap_or_default(),
                metadata: None,
                created_at: row.get::<String>("created_at").unwrap_or_default(),
            });
        }

        Ok(LineageGraph {
            nodes: all_nodes.into_values().collect(),
            edges: all_edges,
        })
    }

    /// Helper to collect nodes and edges from a Cypher query result
    async fn collect_graph_results(
        &self,
        cypher: &str,
        tenant_id: &str,
        entity_id: &str,
        nodes: &mut HashMap<String, LineageNode>,
        edges: &mut Vec<LineageEdge>,
    ) -> Result<()> {
        let mut result = self.graph.execute(
            query(cypher)
                .param("tenant_id", tenant_id)
                .param("entity_id", entity_id)
        ).await?;

        let mut seen_edges: std::collections::HashSet<String> = std::collections::HashSet::new();

        while let Some(row) = result.next().await? {
            // Collect node
            let node_entity_id = row.get::<String>("node_entity_id").unwrap_or_default();
            if !node_entity_id.is_empty() {
                nodes.entry(node_entity_id.clone()).or_insert(LineageNode {
                    id: row.get::<String>("node_id").unwrap_or_default(),
                    tenant_id: row.get::<String>("node_tenant_id").unwrap_or_default(),
                    entity_id: node_entity_id,
                    entity_type: row.get::<String>("node_entity_type").unwrap_or_default(),
                    name: row.get::<String>("node_name").unwrap_or_default(),
                    system: row.get::<String>("node_system").unwrap_or_default(),
                    metadata: None,
                    created_at: row.get::<String>("node_created_at").unwrap_or_default(),
                });
            }

            // Collect edge
            let rel_id = row.get::<String>("rel_id").unwrap_or_default();
            if !rel_id.is_empty() && seen_edges.insert(rel_id.clone()) {
                let transformation = row.get::<String>("rel_transformation").unwrap_or_default();
                let integration_id = row.get::<String>("rel_integration_id").unwrap_or_default();

                edges.push(LineageEdge {
                    id: rel_id,
                    source_id: row.get::<String>("rel_source_id").unwrap_or_default(),
                    target_id: row.get::<String>("rel_target_id").unwrap_or_default(),
                    edge_type: row.get::<String>("rel_edge_type").unwrap_or_default(),
                    transformation: if transformation.is_empty() { None } else { Some(transformation) },
                    integration_id: if integration_id.is_empty() { None } else { Some(integration_id) },
                    created_at: row.get::<String>("rel_created_at").unwrap_or_default(),
                });
            }
        }

        Ok(())
    }

    /// Get complete upstream AND downstream lineage for an entity
    pub async fn get_full_lineage(
        &self,
        tenant_id: Uuid,
        entity_id: &str,
    ) -> Result<LineageGraph> {
        self.get_lineage(tenant_id, LineageQueryParams {
            entity_id: entity_id.to_string(),
            direction: Some("both".to_string()),
            depth: Some(100),
            entity_type: None,
        }).await
    }

    /// Delete a lineage node and all its edges
    pub async fn delete_lineage(
        &self,
        tenant_id: Uuid,
        entity_id: &str,
    ) -> Result<bool> {
        let tenant = tenant_id.to_string();

        let cypher = r#"
            MATCH (n:LineageNode {tenant_id: $tenant_id, entity_id: $entity_id})
            DETACH DELETE n
            RETURN count(n) AS deleted_count
        "#;

        let mut result = self.graph.execute(
            query(cypher)
                .param("tenant_id", tenant.as_str())
                .param("entity_id", entity_id)
        ).await?;

        if let Some(row) = result.next().await? {
            let count: i64 = row.get("deleted_count").unwrap_or(0);
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Follow upstream edges to find all root data sources (nodes with no incoming edges)
    pub async fn get_root_sources(
        &self,
        tenant_id: Uuid,
        entity_id: &str,
    ) -> Result<Vec<LineageNode>> {
        let tenant = tenant_id.to_string();

        let cypher = r#"
            MATCH (start:LineageNode {tenant_id: $tenant_id, entity_id: $entity_id})
            MATCH path = (root:LineageNode)-[*]->(start)
            WHERE root.tenant_id = $tenant_id
              AND NOT EXISTS { MATCH (other)-[]->(root) WHERE other.tenant_id = $tenant_id }
            RETURN DISTINCT root.id AS id, root.tenant_id AS tenant_id,
                   root.entity_id AS entity_id, root.entity_type AS entity_type,
                   root.name AS name, root.system AS system,
                   root.created_at AS created_at
        "#;

        let mut result = self.graph.execute(
            query(cypher)
                .param("tenant_id", tenant.as_str())
                .param("entity_id", entity_id)
        ).await?;

        let mut sources: Vec<LineageNode> = Vec::new();

        while let Some(row) = result.next().await? {
            sources.push(LineageNode {
                id: row.get::<String>("id").unwrap_or_default(),
                tenant_id: row.get::<String>("tenant_id").unwrap_or_default(),
                entity_id: row.get::<String>("entity_id").unwrap_or_default(),
                entity_type: row.get::<String>("entity_type").unwrap_or_default(),
                name: row.get::<String>("name").unwrap_or_default(),
                system: row.get::<String>("system").unwrap_or_default(),
                metadata: None,
                created_at: row.get::<String>("created_at").unwrap_or_default(),
            });
        }

        Ok(sources)
    }

    /// Follow downstream edges to find all impacted entities (downstream impact analysis)
    pub async fn get_impact(
        &self,
        tenant_id: Uuid,
        entity_id: &str,
    ) -> Result<Vec<LineageNode>> {
        let tenant = tenant_id.to_string();

        let cypher = r#"
            MATCH (start:LineageNode {tenant_id: $tenant_id, entity_id: $entity_id})
            MATCH path = (start)-[*]->(impacted:LineageNode)
            WHERE impacted.tenant_id = $tenant_id
            RETURN DISTINCT impacted.id AS id, impacted.tenant_id AS tenant_id,
                   impacted.entity_id AS entity_id, impacted.entity_type AS entity_type,
                   impacted.name AS name, impacted.system AS system,
                   impacted.created_at AS created_at
        "#;

        let mut result = self.graph.execute(
            query(cypher)
                .param("tenant_id", tenant.as_str())
                .param("entity_id", entity_id)
        ).await?;

        let mut impacted: Vec<LineageNode> = Vec::new();

        while let Some(row) = result.next().await? {
            impacted.push(LineageNode {
                id: row.get::<String>("id").unwrap_or_default(),
                tenant_id: row.get::<String>("tenant_id").unwrap_or_default(),
                entity_id: row.get::<String>("entity_id").unwrap_or_default(),
                entity_type: row.get::<String>("entity_type").unwrap_or_default(),
                name: row.get::<String>("name").unwrap_or_default(),
                system: row.get::<String>("system").unwrap_or_default(),
                metadata: None,
                created_at: row.get::<String>("created_at").unwrap_or_default(),
            });
        }

        Ok(impacted)
    }
}

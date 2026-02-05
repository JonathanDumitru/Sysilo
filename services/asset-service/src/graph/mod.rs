use anyhow::Result;
use neo4rs::{Graph, query};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A node in the graph visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub depth: i32,
}

/// An edge in the graph visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: Uuid,
    pub target: Uuid,
    pub relationship_type: String,
}

/// A subgraph containing nodes and edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGraph {
    pub root_id: Uuid,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// A path between two assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPath {
    pub start_id: Uuid,
    pub end_id: Uuid,
    pub path: Vec<PathSegment>,
    pub total_hops: i32,
}

/// A segment in a path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub relationship_to_next: Option<String>,
}

/// Service for graph operations
pub struct GraphService {
    neo4j: Graph,
}

impl GraphService {
    /// Create a new graph service
    pub async fn new(
        neo4j_uri: &str,
        neo4j_user: &str,
        neo4j_password: &str,
    ) -> Result<Self> {
        let neo4j = Graph::new(neo4j_uri, neo4j_user, neo4j_password).await?;
        Ok(Self { neo4j })
    }

    /// Get immediate neighbors of an asset
    pub async fn get_neighbors(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        direction: Option<&str>,
    ) -> Result<SubGraph> {
        let cypher = match direction {
            Some("outgoing") => {
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[r]->(neighbor:Asset)
                RETURN a, neighbor, r, type(r) as rel_type, 1 as depth
                "#
            }
            Some("incoming") => {
                r#"
                MATCH (neighbor:Asset)-[r]->(a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN a, neighbor, r, type(r) as rel_type, -1 as depth
                "#
            }
            _ => {
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[r]-(neighbor:Asset)
                RETURN a, neighbor, r, type(r) as rel_type,
                       CASE WHEN startNode(r) = a THEN 1 ELSE -1 END as depth
                "#
            }
        };

        let mut result = self.neo4j.execute(
            query(cypher)
                .param("asset_id", asset_id.to_string())
                .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut nodes = vec![];
        let mut edges = vec![];
        let mut seen_nodes = std::collections::HashSet::new();

        // Add root node
        let root_result = self.neo4j.execute(
            query(
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN a.id as id, a.name as name, a.asset_type as asset_type, a.status as status
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await;

        if let Ok(mut root_rows) = root_result {
            if let Ok(Some(row)) = root_rows.next().await {
                let id_str: String = row.get("id").unwrap_or_default();
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    seen_nodes.insert(id);
                    nodes.push(GraphNode {
                        id,
                        name: row.get("name").unwrap_or_default(),
                        asset_type: row.get("asset_type").unwrap_or_default(),
                        status: row.get("status").unwrap_or_default(),
                        depth: 0,
                    });
                }
            }
        }

        while let Ok(Some(row)) = result.next().await {
            let neighbor_id_str: String = row.get::<neo4rs::Node>("neighbor")
                .map(|n| n.get::<String>("id").unwrap_or_default())
                .unwrap_or_default();
            let neighbor_name: String = row.get::<neo4rs::Node>("neighbor")
                .map(|n| n.get::<String>("name").unwrap_or_default())
                .unwrap_or_default();
            let neighbor_type: String = row.get::<neo4rs::Node>("neighbor")
                .map(|n| n.get::<String>("asset_type").unwrap_or_default())
                .unwrap_or_default();
            let neighbor_status: String = row.get::<neo4rs::Node>("neighbor")
                .map(|n| n.get::<String>("status").unwrap_or_default())
                .unwrap_or_default();
            let rel_type: String = row.get("rel_type").unwrap_or_default();
            let depth: i64 = row.get("depth").unwrap_or(1);

            if let Ok(neighbor_id) = Uuid::parse_str(&neighbor_id_str) {
                if !seen_nodes.contains(&neighbor_id) {
                    seen_nodes.insert(neighbor_id);
                    nodes.push(GraphNode {
                        id: neighbor_id,
                        name: neighbor_name,
                        asset_type: neighbor_type,
                        status: neighbor_status,
                        depth: depth as i32,
                    });
                }

                // Determine edge direction
                let (source, target) = if depth > 0 {
                    (asset_id, neighbor_id)
                } else {
                    (neighbor_id, asset_id)
                };

                edges.push(GraphEdge {
                    source,
                    target,
                    relationship_type: rel_type,
                });
            }
        }

        Ok(SubGraph {
            root_id: asset_id,
            nodes,
            edges,
        })
    }

    /// Get a subgraph centered on an asset with specified depth
    pub async fn get_subgraph(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        max_depth: i32,
    ) -> Result<SubGraph> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH path = (root:Asset {id: $asset_id, tenant_id: $tenant_id})-[*1..$max_depth]-(connected:Asset)
                WITH root, connected, relationships(path) as rels, length(path) as depth
                UNWIND rels as r
                WITH root, connected, r, depth, startNode(r) as source, endNode(r) as target
                RETURN DISTINCT
                    connected.id as node_id,
                    connected.name as node_name,
                    connected.asset_type as node_type,
                    connected.status as node_status,
                    depth,
                    source.id as source_id,
                    target.id as target_id,
                    type(r) as rel_type
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
            .param("max_depth", max_depth as i64)
        ).await?;

        let mut nodes = vec![];
        let mut edges = vec![];
        let mut seen_nodes = std::collections::HashSet::new();
        let mut seen_edges = std::collections::HashSet::new();

        // Add root node
        let root_result = self.neo4j.execute(
            query(
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN a.name as name, a.asset_type as asset_type, a.status as status
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await;

        if let Ok(mut root_rows) = root_result {
            if let Ok(Some(row)) = root_rows.next().await {
                seen_nodes.insert(asset_id);
                nodes.push(GraphNode {
                    id: asset_id,
                    name: row.get("name").unwrap_or_default(),
                    asset_type: row.get("asset_type").unwrap_or_default(),
                    status: row.get("status").unwrap_or_default(),
                    depth: 0,
                });
            }
        }

        while let Ok(Some(row)) = result.next().await {
            let node_id_str: String = row.get("node_id").unwrap_or_default();
            let node_name: String = row.get("node_name").unwrap_or_default();
            let node_type: String = row.get("node_type").unwrap_or_default();
            let node_status: String = row.get("node_status").unwrap_or_default();
            let depth: i64 = row.get("depth").unwrap_or(1);
            let source_id_str: String = row.get("source_id").unwrap_or_default();
            let target_id_str: String = row.get("target_id").unwrap_or_default();
            let rel_type: String = row.get("rel_type").unwrap_or_default();

            if let Ok(node_id) = Uuid::parse_str(&node_id_str) {
                if !seen_nodes.contains(&node_id) {
                    seen_nodes.insert(node_id);
                    nodes.push(GraphNode {
                        id: node_id,
                        name: node_name,
                        asset_type: node_type,
                        status: node_status,
                        depth: depth as i32,
                    });
                }
            }

            if let (Ok(source_id), Ok(target_id)) = (
                Uuid::parse_str(&source_id_str),
                Uuid::parse_str(&target_id_str)
            ) {
                let edge_key = (source_id, target_id, rel_type.clone());
                if !seen_edges.contains(&edge_key) {
                    seen_edges.insert(edge_key);
                    edges.push(GraphEdge {
                        source: source_id,
                        target: target_id,
                        relationship_type: rel_type,
                    });
                }
            }
        }

        Ok(SubGraph {
            root_id: asset_id,
            nodes,
            edges,
        })
    }

    /// Find shortest path between two assets
    pub async fn find_path(
        &self,
        tenant_id: Uuid,
        start_id: Uuid,
        end_id: Uuid,
    ) -> Result<Option<AssetPath>> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH path = shortestPath(
                    (start:Asset {id: $start_id, tenant_id: $tenant_id})-[*]-(end:Asset {id: $end_id, tenant_id: $tenant_id})
                )
                WITH nodes(path) as pathNodes, relationships(path) as pathRels
                UNWIND range(0, size(pathNodes)-1) as idx
                WITH pathNodes[idx] as node,
                     CASE WHEN idx < size(pathRels) THEN type(pathRels[idx]) ELSE null END as nextRel
                RETURN node.id as id, node.name as name, node.asset_type as asset_type, nextRel
                "#
            )
            .param("start_id", start_id.to_string())
            .param("end_id", end_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut segments = vec![];

        while let Ok(Some(row)) = result.next().await {
            let id_str: String = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let asset_type: String = row.get("asset_type").unwrap_or_default();
            let next_rel: Option<String> = row.get("nextRel").ok();

            if let Ok(id) = Uuid::parse_str(&id_str) {
                segments.push(PathSegment {
                    asset_id: id,
                    asset_name: name,
                    asset_type,
                    relationship_to_next: next_rel,
                });
            }
        }

        if segments.is_empty() {
            return Ok(None);
        }

        Ok(Some(AssetPath {
            start_id,
            end_id,
            total_hops: (segments.len() - 1) as i32,
            path: segments,
        }))
    }
}

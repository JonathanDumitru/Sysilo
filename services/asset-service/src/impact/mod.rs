use anyhow::Result;
use neo4rs::{Graph, query};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub total_downstream: i32,
    pub total_upstream: i32,
    pub downstream_by_type: Vec<TypeCount>,
    pub upstream_by_type: Vec<TypeCount>,
    pub critical_paths: Vec<CriticalPath>,
    pub risk_score: f64,
}

/// Count of assets by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCount {
    pub asset_type: String,
    pub count: i32,
}

/// A critical dependency path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    pub path: Vec<PathNode>,
    pub risk_level: String,
    pub reason: String,
}

/// A node in a critical path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub status: String,
}

/// Downstream impact details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownstreamImpact {
    pub asset_id: Uuid,
    pub affected_assets: Vec<AffectedAsset>,
    pub total_affected: i32,
    pub affected_by_depth: Vec<DepthCount>,
}

/// An affected asset with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedAsset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub depth: i32,
    pub path_to_root: Vec<String>,
}

/// Count at each depth level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthCount {
    pub depth: i32,
    pub count: i32,
}

/// Upstream dependency details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamDependencies {
    pub asset_id: Uuid,
    pub dependencies: Vec<DependencyAsset>,
    pub total_dependencies: i32,
    pub critical_dependencies: Vec<DependencyAsset>,
}

/// A dependency with its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAsset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub depth: i32,
    pub relationship_type: String,
    pub is_critical: bool,
}

/// Service for impact analysis
pub struct ImpactService {
    neo4j: Graph,
}

impl ImpactService {
    /// Create a new impact service
    pub async fn new(
        neo4j_uri: &str,
        neo4j_user: &str,
        neo4j_password: &str,
    ) -> Result<Self> {
        let neo4j = Graph::new(neo4j_uri, neo4j_user, neo4j_password).await?;
        Ok(Self { neo4j })
    }

    /// Get full impact analysis for an asset
    pub async fn get_impact_analysis(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<ImpactAnalysis> {
        // Get root asset info
        let mut root_result = self.neo4j.execute(
            query(
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN a.name as name
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let asset_name = if let Ok(Some(row)) = root_result.next().await {
            row.get::<String>("name").unwrap_or_default()
        } else {
            String::new()
        };

        // Count downstream by type
        let mut downstream_result = self.neo4j.execute(
            query(
                r#"
                MATCH (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[*]->(downstream:Asset)
                RETURN downstream.asset_type as asset_type, count(DISTINCT downstream) as count
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut downstream_by_type = vec![];
        let mut total_downstream = 0;

        while let Ok(Some(row)) = downstream_result.next().await {
            let asset_type: String = row.get("asset_type").unwrap_or_default();
            let count: i64 = row.get("count").unwrap_or(0);
            total_downstream += count as i32;
            downstream_by_type.push(TypeCount {
                asset_type,
                count: count as i32,
            });
        }

        // Count upstream by type
        let mut upstream_result = self.neo4j.execute(
            query(
                r#"
                MATCH (upstream:Asset)-[*]->(a:Asset {id: $asset_id, tenant_id: $tenant_id})
                RETURN upstream.asset_type as asset_type, count(DISTINCT upstream) as count
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut upstream_by_type = vec![];
        let mut total_upstream = 0;

        while let Ok(Some(row)) = upstream_result.next().await {
            let asset_type: String = row.get("asset_type").unwrap_or_default();
            let count: i64 = row.get("count").unwrap_or(0);
            total_upstream += count as i32;
            upstream_by_type.push(TypeCount {
                asset_type,
                count: count as i32,
            });
        }

        // Calculate risk score based on dependency metrics
        let risk_score = calculate_risk_score(total_downstream, total_upstream);

        // Find critical paths (deprecated assets in dependency chain)
        let critical_paths = self.find_critical_paths(tenant_id, asset_id).await?;

        Ok(ImpactAnalysis {
            asset_id,
            asset_name,
            total_downstream,
            total_upstream,
            downstream_by_type,
            upstream_by_type,
            critical_paths,
            risk_score,
        })
    }

    /// Get detailed downstream impact
    pub async fn get_downstream_impact(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        max_depth: i32,
    ) -> Result<DownstreamImpact> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH path = (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[*1..$max_depth]->(downstream:Asset)
                WITH downstream, length(path) as depth, [n in nodes(path) | n.name] as pathNames
                RETURN DISTINCT
                    downstream.id as id,
                    downstream.name as name,
                    downstream.asset_type as asset_type,
                    downstream.status as status,
                    min(depth) as depth,
                    collect(DISTINCT pathNames)[0] as path_to_root
                ORDER BY depth
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
            .param("max_depth", max_depth as i64)
        ).await?;

        let mut affected_assets = vec![];
        let mut depth_counts: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

        while let Ok(Some(row)) = result.next().await {
            let id_str: String = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let asset_type: String = row.get("asset_type").unwrap_or_default();
            let status: String = row.get("status").unwrap_or_default();
            let depth: i64 = row.get("depth").unwrap_or(1);
            let path_to_root: Vec<String> = row.get("path_to_root").unwrap_or_default();

            if let Ok(id) = Uuid::parse_str(&id_str) {
                affected_assets.push(AffectedAsset {
                    id,
                    name,
                    asset_type,
                    status,
                    depth: depth as i32,
                    path_to_root,
                });

                *depth_counts.entry(depth as i32).or_insert(0) += 1;
            }
        }

        let affected_by_depth: Vec<DepthCount> = depth_counts
            .into_iter()
            .map(|(depth, count)| DepthCount { depth, count })
            .collect();

        Ok(DownstreamImpact {
            asset_id,
            total_affected: affected_assets.len() as i32,
            affected_assets,
            affected_by_depth,
        })
    }

    /// Get detailed upstream dependencies
    pub async fn get_upstream_dependencies(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        max_depth: i32,
    ) -> Result<UpstreamDependencies> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH path = (upstream:Asset)-[r*1..$max_depth]->(a:Asset {id: $asset_id, tenant_id: $tenant_id})
                WITH upstream, length(path) as depth, type(head(r)) as rel_type
                RETURN DISTINCT
                    upstream.id as id,
                    upstream.name as name,
                    upstream.asset_type as asset_type,
                    upstream.status as status,
                    min(depth) as depth,
                    rel_type
                ORDER BY depth
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
            .param("max_depth", max_depth as i64)
        ).await?;

        let mut dependencies = vec![];

        while let Ok(Some(row)) = result.next().await {
            let id_str: String = row.get("id").unwrap_or_default();
            let name: String = row.get("name").unwrap_or_default();
            let asset_type: String = row.get("asset_type").unwrap_or_default();
            let status: String = row.get("status").unwrap_or_default();
            let depth: i64 = row.get("depth").unwrap_or(1);
            let rel_type: String = row.get("rel_type").unwrap_or_default();

            // Mark as critical if deprecated or sunset
            let is_critical = status == "deprecated" || status == "sunset";

            if let Ok(id) = Uuid::parse_str(&id_str) {
                dependencies.push(DependencyAsset {
                    id,
                    name,
                    asset_type,
                    status,
                    depth: depth as i32,
                    relationship_type: rel_type,
                    is_critical,
                });
            }
        }

        let critical_dependencies: Vec<DependencyAsset> = dependencies
            .iter()
            .filter(|d| d.is_critical)
            .cloned()
            .collect();

        Ok(UpstreamDependencies {
            asset_id,
            total_dependencies: dependencies.len() as i32,
            critical_dependencies,
            dependencies,
        })
    }

    /// Find critical paths (paths through deprecated/problematic assets)
    async fn find_critical_paths(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<CriticalPath>> {
        let mut result = self.neo4j.execute(
            query(
                r#"
                MATCH path = (a:Asset {id: $asset_id, tenant_id: $tenant_id})-[*1..5]->(critical:Asset)
                WHERE critical.status IN ['deprecated', 'sunset']
                WITH nodes(path) as pathNodes, critical
                RETURN [n in pathNodes | {
                    id: n.id,
                    name: n.name,
                    asset_type: n.asset_type,
                    status: n.status
                }] as path, critical.status as critical_status
                LIMIT 10
                "#
            )
            .param("asset_id", asset_id.to_string())
            .param("tenant_id", tenant_id.to_string())
        ).await?;

        let mut critical_paths = vec![];

        while let Ok(Some(row)) = result.next().await {
            let critical_status: String = row.get("critical_status").unwrap_or_default();

            // Parse the path nodes
            // Note: Simplified parsing - in production would properly deserialize
            let path_nodes: Vec<PathNode> = vec![]; // Would parse from row

            let risk_level = match critical_status.as_str() {
                "sunset" => "high",
                "deprecated" => "medium",
                _ => "low",
            };

            let reason = match critical_status.as_str() {
                "sunset" => "Path leads to a sunset asset that will be removed",
                "deprecated" => "Path depends on a deprecated asset",
                _ => "Unknown risk",
            };

            critical_paths.push(CriticalPath {
                path: path_nodes,
                risk_level: risk_level.to_string(),
                reason: reason.to_string(),
            });
        }

        Ok(critical_paths)
    }
}

/// Calculate risk score based on dependency metrics
fn calculate_risk_score(downstream: i32, upstream: i32) -> f64 {
    // Higher downstream = higher blast radius
    // Higher upstream = higher fragility
    let downstream_factor = (downstream as f64).ln_1p() / 10.0;
    let upstream_factor = (upstream as f64).ln_1p() / 10.0;

    // Combine factors, cap at 1.0
    (downstream_factor * 0.6 + upstream_factor * 0.4).min(1.0)
}

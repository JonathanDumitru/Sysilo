use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regorus::Engine;
use tracing::{info, warn};

// ============================================================================
// Types
// ============================================================================

/// A governance domain -- a logical grouping of resources owned by a team
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceDomain {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_domain_id: Option<Uuid>,
    pub owner_team: String,
    pub owner_email: String,
    pub resource_patterns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain policy type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainPolicyType {
    Extension,  // Extends an enterprise policy (can only add stricter rules)
    Local,      // Domain-specific policy (no enterprise parent)
    Override,   // Requires approval to override enterprise policy (rare)
}

impl DomainPolicyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainPolicyType::Extension => "extension",
            DomainPolicyType::Local => "local",
            DomainPolicyType::Override => "override",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "extension" => Ok(DomainPolicyType::Extension),
            "local" => Ok(DomainPolicyType::Local),
            "override" => Ok(DomainPolicyType::Override),
            _ => Err(anyhow!("Invalid domain policy type: {}", s)),
        }
    }
}

/// A domain-scoped policy that inherits from enterprise policies
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DomainPolicy {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: String,
    pub rego_rule: String,
    pub enforcement_mode: String,
    pub severity: String,
    pub extends_policy_id: Option<Uuid>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Governance health score for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthScore {
    pub domain_id: Uuid,
    pub domain_name: String,
    pub overall_score: f64,
    pub dimensions: HealthDimensions,
    pub last_assessed_at: DateTime<Utc>,
    pub trend: String,
}

/// Individual health dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDimensions {
    pub policy_coverage: f64,
    pub compliance_rate: f64,
    pub violation_resolution_time_hours: f64,
    pub approval_turnaround_hours: f64,
    pub audit_completeness: f64,
    pub policy_freshness: f64,
}

/// Stored health score record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HealthScoreRecord {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub tenant_id: Uuid,
    pub overall_score: f64,
    pub policy_coverage: f64,
    pub compliance_rate: f64,
    pub violation_resolution_time_hours: f64,
    pub approval_turnaround_hours: f64,
    pub audit_completeness: f64,
    pub policy_freshness: f64,
    pub trend: String,
    pub assessed_at: DateTime<Utc>,
}

/// Policy inheritance chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInheritanceChain {
    pub enterprise_policy: Option<PolicySummary>,
    pub domain_policies: Vec<DomainPolicySummary>,
    pub effective_rules: Vec<String>,
}

/// Summary of an enterprise policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySummary {
    pub id: Uuid,
    pub name: String,
    pub scope: String,
    pub enforcement_mode: String,
}

/// Summary of a domain policy in the inheritance chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPolicySummary {
    pub id: Uuid,
    pub name: String,
    pub domain_name: String,
    pub policy_type: String,
    pub enforcement_mode: String,
}

/// Node in a domain hierarchy tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTreeNode {
    pub domain: GovernanceDomain,
    pub children: Vec<DomainTreeNode>,
}

/// Result of federated policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedEvaluationResult {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub allowed: bool,
    pub enterprise_results: Vec<PolicyEvalDetail>,
    pub domain_results: Vec<PolicyEvalDetail>,
    pub blocking_policy: Option<String>,
    pub domain_name: Option<String>,
}

/// Detail of a single policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvalDetail {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub source: String, // "enterprise" or domain name
    pub passed: bool,
    pub violations: Vec<String>,
    pub enforcement_mode: String,
    pub severity: String,
}

// ============================================================================
// Request types
// ============================================================================

/// Request to create a governance domain
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_domain_id: Option<Uuid>,
    pub owner_team: String,
    pub owner_email: String,
    pub resource_patterns: Vec<String>,
}

/// Request to update a governance domain
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDomainRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_domain_id: Option<Uuid>,
    pub owner_team: Option<String>,
    pub owner_email: Option<String>,
    pub resource_patterns: Option<Vec<String>>,
}

/// Request to create a domain policy
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDomainPolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub policy_type: DomainPolicyType,
    pub rego_rule: String,
    pub enforcement_mode: String,
    pub severity: String,
    pub extends_policy_id: Option<Uuid>,
}

/// Request to update a domain policy
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDomainPolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rego_rule: Option<String>,
    pub enforcement_mode: Option<String>,
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

/// Request to evaluate with federated inheritance
#[derive(Debug, Clone, Deserialize)]
pub struct FederatedEvaluateRequest {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub resource_data: serde_json::Value,
}

/// Query params for inheritance chain
#[derive(Debug, Clone, Deserialize)]
pub struct InheritanceQueryParams {
    pub resource_type: String,
    pub resource_id: String,
}

/// Query params for health trends
#[derive(Debug, Clone, Deserialize)]
pub struct HealthTrendsParams {
    pub days: Option<i64>,
}

// ============================================================================
// Service
// ============================================================================

/// Service for federated governance with local domain ownership
pub struct FederatedGovernanceService {
    pool: PgPool,
}

impl FederatedGovernanceService {
    /// Create a new federated governance service and initialize tables
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let service = Self { pool };
        service.create_tables().await?;

        info!("FederatedGovernanceService initialized");
        Ok(service)
    }

    /// Create the required database tables
    async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS governance_domains (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                parent_domain_id UUID REFERENCES governance_domains(id) ON DELETE SET NULL,
                owner_team TEXT NOT NULL,
                owner_email TEXT NOT NULL,
                resource_patterns TEXT[] NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (tenant_id, name)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS domain_policies (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                domain_id UUID NOT NULL REFERENCES governance_domains(id) ON DELETE CASCADE,
                tenant_id UUID NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                policy_type TEXT NOT NULL DEFAULT 'local',
                rego_rule TEXT NOT NULL,
                enforcement_mode TEXT NOT NULL DEFAULT 'audit',
                severity TEXT NOT NULL DEFAULT 'medium',
                extends_policy_id UUID,
                enabled BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (domain_id, name)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS governance_health_scores (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                domain_id UUID NOT NULL REFERENCES governance_domains(id) ON DELETE CASCADE,
                tenant_id UUID NOT NULL,
                overall_score DOUBLE PRECISION NOT NULL DEFAULT 0,
                policy_coverage DOUBLE PRECISION NOT NULL DEFAULT 0,
                compliance_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
                violation_resolution_time_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
                approval_turnaround_hours DOUBLE PRECISION NOT NULL DEFAULT 0,
                audit_completeness DOUBLE PRECISION NOT NULL DEFAULT 0,
                policy_freshness DOUBLE PRECISION NOT NULL DEFAULT 0,
                trend TEXT NOT NULL DEFAULT 'stable',
                assessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for common queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_governance_domains_tenant
                ON governance_domains(tenant_id)
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_policies_domain
                ON domain_policies(domain_id)
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_policies_tenant
                ON domain_policies(tenant_id)
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_health_scores_domain_assessed
                ON governance_health_scores(domain_id, assessed_at DESC)
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Domain Management
    // ========================================================================

    /// Create a governance domain
    pub async fn create_domain(
        &self,
        tenant_id: Uuid,
        req: CreateDomainRequest,
    ) -> Result<GovernanceDomain> {
        // If parent_domain_id is specified, verify it exists and belongs to the same tenant
        if let Some(parent_id) = req.parent_domain_id {
            let parent = self.get_domain(tenant_id, parent_id).await?;
            if parent.is_none() {
                return Err(anyhow!("Parent domain not found"));
            }
        }

        let domain = sqlx::query_as::<_, GovernanceDomain>(
            r#"
            INSERT INTO governance_domains
                (tenant_id, name, description, parent_domain_id, owner_team, owner_email, resource_patterns)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, name, description, parent_domain_id, owner_team,
                      owner_email, resource_patterns, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.parent_domain_id)
        .bind(&req.owner_team)
        .bind(&req.owner_email)
        .bind(&req.resource_patterns)
        .fetch_one(&self.pool)
        .await?;

        info!(
            domain_id = %domain.id,
            domain_name = %domain.name,
            tenant_id = %tenant_id,
            "Governance domain created"
        );

        Ok(domain)
    }

    /// Get a single governance domain
    pub async fn get_domain(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<GovernanceDomain>> {
        let domain = sqlx::query_as::<_, GovernanceDomain>(
            r#"
            SELECT id, tenant_id, name, description, parent_domain_id, owner_team,
                   owner_email, resource_patterns, created_at, updated_at
            FROM governance_domains
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(domain)
    }

    /// List all governance domains for a tenant
    pub async fn list_domains(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<GovernanceDomain>> {
        let domains = sqlx::query_as::<_, GovernanceDomain>(
            r#"
            SELECT id, tenant_id, name, description, parent_domain_id, owner_team,
                   owner_email, resource_patterns, created_at, updated_at
            FROM governance_domains
            WHERE tenant_id = $1
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(domains)
    }

    /// Update a governance domain
    pub async fn update_domain(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateDomainRequest,
    ) -> Result<Option<GovernanceDomain>> {
        let existing = self.get_domain(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        // If changing parent, verify the new parent exists
        if let Some(parent_id) = req.parent_domain_id {
            // Prevent self-referencing
            if parent_id == id {
                return Err(anyhow!("A domain cannot be its own parent"));
            }
            let parent = self.get_domain(tenant_id, parent_id).await?;
            if parent.is_none() {
                return Err(anyhow!("Parent domain not found"));
            }
        }

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let parent_domain_id = req.parent_domain_id.or(existing.parent_domain_id);
        let owner_team = req.owner_team.unwrap_or(existing.owner_team);
        let owner_email = req.owner_email.unwrap_or(existing.owner_email);
        let resource_patterns = req.resource_patterns.unwrap_or(existing.resource_patterns);

        let domain = sqlx::query_as::<_, GovernanceDomain>(
            r#"
            UPDATE governance_domains SET
                name = $1, description = $2, parent_domain_id = $3,
                owner_team = $4, owner_email = $5, resource_patterns = $6,
                updated_at = NOW()
            WHERE tenant_id = $7 AND id = $8
            RETURNING id, tenant_id, name, description, parent_domain_id, owner_team,
                      owner_email, resource_patterns, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(parent_domain_id)
        .bind(&owner_team)
        .bind(&owner_email)
        .bind(&resource_patterns)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(domain))
    }

    /// Delete a governance domain (only if no active policies)
    pub async fn delete_domain(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<bool> {
        // Check for active policies
        let active_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM domain_policies
            WHERE domain_id = $1 AND tenant_id = $2 AND enabled = true
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        if active_count.0 > 0 {
            return Err(anyhow!(
                "Cannot delete domain with {} active policies. Disable or delete them first.",
                active_count.0
            ));
        }

        // Check for child domains
        let child_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM governance_domains
            WHERE parent_domain_id = $1 AND tenant_id = $2
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        if child_count.0 > 0 {
            return Err(anyhow!(
                "Cannot delete domain with {} child domains. Reassign or delete them first.",
                child_count.0
            ));
        }

        let result = sqlx::query(
            "DELETE FROM governance_domains WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get domain hierarchy as a tree structure
    pub async fn get_domain_hierarchy(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<DomainTreeNode>> {
        let domains = self.list_domains(tenant_id).await?;

        // Build tree from flat list: collect roots, then recursively attach children
        let roots: Vec<&GovernanceDomain> = domains
            .iter()
            .filter(|d| d.parent_domain_id.is_none())
            .collect();

        let tree = roots
            .into_iter()
            .map(|root| self.build_tree_node(root, &domains))
            .collect();

        Ok(tree)
    }

    /// Recursively build a domain tree node
    fn build_tree_node(
        &self,
        domain: &GovernanceDomain,
        all_domains: &[GovernanceDomain],
    ) -> DomainTreeNode {
        let children: Vec<DomainTreeNode> = all_domains
            .iter()
            .filter(|d| d.parent_domain_id == Some(domain.id))
            .map(|child| self.build_tree_node(child, all_domains))
            .collect();

        DomainTreeNode {
            domain: domain.clone(),
            children,
        }
    }

    // ========================================================================
    // Domain Policy Management
    // ========================================================================

    /// Create a domain policy
    pub async fn create_domain_policy(
        &self,
        tenant_id: Uuid,
        domain_id: Uuid,
        req: CreateDomainPolicyRequest,
    ) -> Result<DomainPolicy> {
        // Verify domain exists
        let domain = self.get_domain(tenant_id, domain_id).await?
            .ok_or_else(|| anyhow!("Domain not found"))?;

        // Validate Rego syntax
        self.validate_rego(&req.rego_rule)?;

        // Validate enforcement mode
        self.validate_enforcement_mode(&req.enforcement_mode)?;

        // If extending an enterprise policy, validate stricter enforcement
        if let Some(parent_policy_id) = req.extends_policy_id {
            let parent_policy = sqlx::query_as::<_, crate::policies::Policy>(
                r#"
                SELECT id, tenant_id, name, description, rego_policy, scope,
                       enforcement_mode, severity, enabled, created_at, updated_at
                FROM policies
                WHERE id = $1 AND tenant_id = $2
                "#
            )
            .bind(parent_policy_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow!("Enterprise policy not found: {}", parent_policy_id))?;

            // Enforcement mode cannot be weaker than parent
            if self.enforcement_strength(&req.enforcement_mode)
                < self.enforcement_strength(&parent_policy.enforcement_mode)
            {
                return Err(anyhow!(
                    "Domain policy enforcement mode '{}' cannot be weaker than enterprise policy enforcement mode '{}'. \
                     Strength order: enforce > warn > audit",
                    req.enforcement_mode,
                    parent_policy.enforcement_mode
                ));
            }

            // For Extension type, only stricter rules are valid
            if matches!(req.policy_type, DomainPolicyType::Extension) {
                info!(
                    domain = %domain.name,
                    parent_policy = %parent_policy.name,
                    "Creating extension policy for enterprise policy"
                );
            }
        } else if matches!(req.policy_type, DomainPolicyType::Extension) {
            return Err(anyhow!(
                "Extension policies must reference an enterprise policy via extends_policy_id"
            ));
        }

        let policy = sqlx::query_as::<_, DomainPolicy>(
            r#"
            INSERT INTO domain_policies
                (domain_id, tenant_id, name, description, policy_type, rego_rule,
                 enforcement_mode, severity, extends_policy_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                      enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
            "#
        )
        .bind(domain_id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.policy_type.as_str())
        .bind(&req.rego_rule)
        .bind(&req.enforcement_mode)
        .bind(&req.severity)
        .bind(req.extends_policy_id)
        .fetch_one(&self.pool)
        .await?;

        info!(
            policy_id = %policy.id,
            domain_id = %domain_id,
            policy_type = %policy.policy_type,
            "Domain policy created"
        );

        Ok(policy)
    }

    /// Update a domain policy
    pub async fn update_domain_policy(
        &self,
        tenant_id: Uuid,
        policy_id: Uuid,
        req: UpdateDomainPolicyRequest,
    ) -> Result<Option<DomainPolicy>> {
        let existing = self.get_domain_policy(tenant_id, policy_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let rego_rule_changed = req.rego_rule.is_some();
        let rego_rule = req.rego_rule.unwrap_or(existing.rego_rule);
        let enforcement_mode = req.enforcement_mode.unwrap_or(existing.enforcement_mode.clone());
        let severity = req.severity.unwrap_or(existing.severity);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        // Validate Rego if changed
        if rego_rule_changed {
            self.validate_rego(&rego_rule)?;
        }

        // Validate enforcement mode
        self.validate_enforcement_mode(&enforcement_mode)?;

        // If this extends an enterprise policy, ensure enforcement mode stays at least as strict
        if let Some(parent_policy_id) = existing.extends_policy_id {
            let parent_policy = sqlx::query_as::<_, crate::policies::Policy>(
                r#"
                SELECT id, tenant_id, name, description, rego_policy, scope,
                       enforcement_mode, severity, enabled, created_at, updated_at
                FROM policies
                WHERE id = $1 AND tenant_id = $2
                "#
            )
            .bind(parent_policy_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(parent) = parent_policy {
                if self.enforcement_strength(&enforcement_mode)
                    < self.enforcement_strength(&parent.enforcement_mode)
                {
                    return Err(anyhow!(
                        "Domain policy enforcement mode '{}' cannot be weaker than enterprise policy enforcement mode '{}'",
                        enforcement_mode,
                        parent.enforcement_mode
                    ));
                }
            }
        }

        let policy = sqlx::query_as::<_, DomainPolicy>(
            r#"
            UPDATE domain_policies SET
                name = $1, description = $2, rego_rule = $3,
                enforcement_mode = $4, severity = $5, enabled = $6,
                updated_at = NOW()
            WHERE tenant_id = $7 AND id = $8
            RETURNING id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                      enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&rego_rule)
        .bind(&enforcement_mode)
        .bind(&severity)
        .bind(enabled)
        .bind(tenant_id)
        .bind(policy_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(policy))
    }

    /// Delete a domain policy
    pub async fn delete_domain_policy(
        &self,
        tenant_id: Uuid,
        policy_id: Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM domain_policies WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(policy_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get a single domain policy
    pub async fn get_domain_policy(
        &self,
        tenant_id: Uuid,
        policy_id: Uuid,
    ) -> Result<Option<DomainPolicy>> {
        let policy = sqlx::query_as::<_, DomainPolicy>(
            r#"
            SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                   enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
            FROM domain_policies
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(policy_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(policy)
    }

    /// List domain policies for a domain
    pub async fn list_domain_policies(
        &self,
        tenant_id: Uuid,
        domain_id: Uuid,
    ) -> Result<Vec<DomainPolicy>> {
        let policies = sqlx::query_as::<_, DomainPolicy>(
            r#"
            SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                   enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
            FROM domain_policies
            WHERE tenant_id = $1 AND domain_id = $2
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .bind(domain_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(policies)
    }

    // ========================================================================
    // Policy Inheritance
    // ========================================================================

    /// Get the full inheritance chain for a resource
    pub async fn get_inheritance_chain(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<PolicyInheritanceChain>> {
        // Find which domain(s) the resource belongs to
        let domains = self.find_domains_for_resource(tenant_id, resource_type, resource_id).await?;

        // Get enterprise policies for this resource type
        let enterprise_policies = sqlx::query_as::<_, crate::policies::Policy>(
            r#"
            SELECT id, tenant_id, name, description, rego_policy, scope,
                   enforcement_mode, severity, enabled, created_at, updated_at
            FROM policies
            WHERE tenant_id = $1 AND enabled = true
              AND (scope = $2 OR scope = 'all')
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .bind(resource_type)
        .fetch_all(&self.pool)
        .await?;

        let mut chains = Vec::new();

        for enterprise_policy in &enterprise_policies {
            let mut chain = PolicyInheritanceChain {
                enterprise_policy: Some(PolicySummary {
                    id: enterprise_policy.id,
                    name: enterprise_policy.name.clone(),
                    scope: enterprise_policy.scope.clone(),
                    enforcement_mode: enterprise_policy.enforcement_mode.clone(),
                }),
                domain_policies: Vec::new(),
                effective_rules: vec![enterprise_policy.rego_policy.clone()],
            };

            // For each domain the resource belongs to, get domain policies that extend this enterprise policy
            for domain in &domains {
                let domain_policies = self.get_extending_policies(
                    tenant_id,
                    domain.id,
                    enterprise_policy.id,
                ).await?;

                for dp in &domain_policies {
                    chain.domain_policies.push(DomainPolicySummary {
                        id: dp.id,
                        name: dp.name.clone(),
                        domain_name: domain.name.clone(),
                        policy_type: dp.policy_type.clone(),
                        enforcement_mode: dp.enforcement_mode.clone(),
                    });
                    chain.effective_rules.push(dp.rego_rule.clone());
                }

                // Also walk up the domain hierarchy for inherited domain policies
                let ancestor_policies = self.collect_ancestor_policies(
                    tenant_id,
                    domain,
                    enterprise_policy.id,
                ).await?;

                for (ancestor_domain_name, dp) in &ancestor_policies {
                    chain.domain_policies.push(DomainPolicySummary {
                        id: dp.id,
                        name: dp.name.clone(),
                        domain_name: ancestor_domain_name.clone(),
                        policy_type: dp.policy_type.clone(),
                        enforcement_mode: dp.enforcement_mode.clone(),
                    });
                    chain.effective_rules.push(dp.rego_rule.clone());
                }
            }

            chains.push(chain);
        }

        // Also add chains for local domain policies (not extending any enterprise policy)
        for domain in &domains {
            let local_policies = sqlx::query_as::<_, DomainPolicy>(
                r#"
                SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                       enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
                FROM domain_policies
                WHERE tenant_id = $1 AND domain_id = $2 AND enabled = true
                  AND extends_policy_id IS NULL AND policy_type = 'local'
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .bind(domain.id)
            .fetch_all(&self.pool)
            .await?;

            for lp in local_policies {
                chains.push(PolicyInheritanceChain {
                    enterprise_policy: None,
                    domain_policies: vec![DomainPolicySummary {
                        id: lp.id,
                        name: lp.name.clone(),
                        domain_name: domain.name.clone(),
                        policy_type: lp.policy_type.clone(),
                        enforcement_mode: lp.enforcement_mode.clone(),
                    }],
                    effective_rules: vec![lp.rego_rule.clone()],
                });
            }
        }

        Ok(chains)
    }

    /// Evaluate a resource against the full federated policy inheritance chain
    pub async fn evaluate_with_inheritance(
        &self,
        tenant_id: Uuid,
        req: FederatedEvaluateRequest,
    ) -> Result<FederatedEvaluationResult> {
        let mut result = FederatedEvaluationResult {
            resource_type: req.resource_type.clone(),
            resource_id: req.resource_id,
            allowed: true,
            enterprise_results: Vec::new(),
            domain_results: Vec::new(),
            blocking_policy: None,
            domain_name: None,
        };

        // 1. Evaluate enterprise policies first -- these cannot be overridden
        let enterprise_policies = sqlx::query_as::<_, crate::policies::Policy>(
            r#"
            SELECT id, tenant_id, name, description, rego_policy, scope,
                   enforcement_mode, severity, enabled, created_at, updated_at
            FROM policies
            WHERE tenant_id = $1 AND enabled = true
              AND (scope = $2 OR scope = 'all')
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .bind(&req.resource_type)
        .fetch_all(&self.pool)
        .await?;

        for policy in &enterprise_policies {
            let eval = self.evaluate_single_rego(&policy.rego_policy, &req.resource_data)?;
            let detail = PolicyEvalDetail {
                policy_id: policy.id,
                policy_name: policy.name.clone(),
                source: "enterprise".to_string(),
                passed: eval.0,
                violations: eval.1.clone(),
                enforcement_mode: policy.enforcement_mode.clone(),
                severity: policy.severity.clone(),
            };

            if !eval.0 && policy.enforcement_mode == "enforce" {
                result.allowed = false;
                result.blocking_policy = Some(format!("enterprise/{}", policy.name));
            }

            result.enterprise_results.push(detail);
        }

        // 2. Find domains for this resource and evaluate domain policies
        let domains = self.find_domains_for_resource(
            tenant_id,
            &req.resource_type,
            &req.resource_id.to_string(),
        ).await?;

        if let Some(domain) = domains.first() {
            result.domain_name = Some(domain.name.clone());
        }

        for domain in &domains {
            // Get all enabled domain policies for this domain
            let domain_policies = sqlx::query_as::<_, DomainPolicy>(
                r#"
                SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                       enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
                FROM domain_policies
                WHERE tenant_id = $1 AND domain_id = $2 AND enabled = true
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .bind(domain.id)
            .fetch_all(&self.pool)
            .await?;

            for dp in &domain_policies {
                let eval = self.evaluate_single_rego(&dp.rego_rule, &req.resource_data)?;
                let detail = PolicyEvalDetail {
                    policy_id: dp.id,
                    policy_name: dp.name.clone(),
                    source: domain.name.clone(),
                    passed: eval.0,
                    violations: eval.1.clone(),
                    enforcement_mode: dp.enforcement_mode.clone(),
                    severity: dp.severity.clone(),
                };

                if !eval.0 && dp.enforcement_mode == "enforce" {
                    result.allowed = false;
                    if result.blocking_policy.is_none() {
                        result.blocking_policy = Some(format!("{}/{}", domain.name, dp.name));
                    }
                }

                result.domain_results.push(detail);
            }

            // Also evaluate policies from ancestor domains
            let mut current_domain = domain.clone();
            while let Some(parent_id) = current_domain.parent_domain_id {
                if let Some(parent) = self.get_domain(tenant_id, parent_id).await? {
                    let parent_policies = sqlx::query_as::<_, DomainPolicy>(
                        r#"
                        SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                               enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
                        FROM domain_policies
                        WHERE tenant_id = $1 AND domain_id = $2 AND enabled = true
                        ORDER BY name
                        "#
                    )
                    .bind(tenant_id)
                    .bind(parent.id)
                    .fetch_all(&self.pool)
                    .await?;

                    for dp in &parent_policies {
                        let eval = self.evaluate_single_rego(&dp.rego_rule, &req.resource_data)?;
                        let detail = PolicyEvalDetail {
                            policy_id: dp.id,
                            policy_name: dp.name.clone(),
                            source: format!("{} (inherited)", parent.name),
                            passed: eval.0,
                            violations: eval.1.clone(),
                            enforcement_mode: dp.enforcement_mode.clone(),
                            severity: dp.severity.clone(),
                        };

                        if !eval.0 && dp.enforcement_mode == "enforce" {
                            result.allowed = false;
                            if result.blocking_policy.is_none() {
                                result.blocking_policy = Some(format!(
                                    "{} (inherited from {})/{}",
                                    domain.name, parent.name, dp.name
                                ));
                            }
                        }

                        result.domain_results.push(detail);
                    }

                    current_domain = parent;
                } else {
                    break;
                }
            }
        }

        Ok(result)
    }

    // ========================================================================
    // Health Scoring
    // ========================================================================

    /// Calculate the governance health score for a domain
    pub async fn calculate_health_score(
        &self,
        tenant_id: Uuid,
        domain_id: Uuid,
    ) -> Result<GovernanceHealthScore> {
        let domain = self.get_domain(tenant_id, domain_id).await?
            .ok_or_else(|| anyhow!("Domain not found"))?;

        // 1. Policy coverage: count of domain policies vs resource patterns
        let policy_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM domain_policies WHERE domain_id = $1 AND tenant_id = $2 AND enabled = true"
        )
        .bind(domain_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let pattern_count = domain.resource_patterns.len().max(1) as f64;
        let policy_coverage = ((policy_count.0 as f64 / pattern_count) * 100.0).min(100.0);

        // 2. Compliance rate: % of recent policy evaluations that pass
        // Query policy_violations for this tenant (approximation based on resources in domain)
        let total_violations: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM policy_violations
            WHERE tenant_id = $1
              AND created_at > NOW() - INTERVAL '30 days'
            "#
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let resolved_violations: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM policy_violations
            WHERE tenant_id = $1
              AND created_at > NOW() - INTERVAL '30 days'
              AND status = 'resolved'
            "#
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let compliance_rate = if total_violations.0 > 0 {
            (resolved_violations.0 as f64 / total_violations.0 as f64) * 100.0
        } else {
            100.0 // No violations means fully compliant
        };

        // 3. Violation resolution time
        let avg_resolution: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(
                AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 3600.0),
                0
            )
            FROM policy_violations
            WHERE tenant_id = $1
              AND status = 'resolved'
              AND resolved_at IS NOT NULL
              AND created_at > NOW() - INTERVAL '90 days'
            "#
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let violation_resolution_time_hours = avg_resolution.map(|r| r.0).unwrap_or(0.0);

        // 4. Approval turnaround time
        let avg_approval: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(
                AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600.0),
                0
            )
            FROM approval_requests
            WHERE tenant_id = $1
              AND status IN ('approved', 'rejected')
              AND created_at > NOW() - INTERVAL '90 days'
            "#
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let approval_turnaround_hours = avg_approval.map(|r| r.0).unwrap_or(0.0);

        // 5. Audit completeness: proportion of actions with audit entries
        let audit_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_log
            WHERE tenant_id = $1
              AND timestamp > NOW() - INTERVAL '30 days'
            "#
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        // Estimate expected entries based on domain activity
        let audit_completeness = if audit_count.0 > 0 { 100.0 } else { 0.0 };

        // 6. Policy freshness: % of domain policies updated in last 90 days
        let fresh_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM domain_policies
            WHERE domain_id = $1 AND tenant_id = $2
              AND updated_at > NOW() - INTERVAL '90 days'
            "#
        )
        .bind(domain_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let total_policy_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM domain_policies WHERE domain_id = $1 AND tenant_id = $2"
        )
        .bind(domain_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let policy_freshness = if total_policy_count.0 > 0 {
            (fresh_count.0 as f64 / total_policy_count.0 as f64) * 100.0
        } else {
            0.0
        };

        // Calculate overall score (weighted average)
        // Weights: coverage 20%, compliance 25%, resolution_time 15%, approval 10%, audit 15%, freshness 15%
        let resolution_score = if violation_resolution_time_hours <= 0.0 {
            100.0
        } else if violation_resolution_time_hours <= 24.0 {
            100.0 - (violation_resolution_time_hours / 24.0 * 30.0) // 70-100 range
        } else if violation_resolution_time_hours <= 72.0 {
            70.0 - ((violation_resolution_time_hours - 24.0) / 48.0 * 40.0) // 30-70 range
        } else {
            (30.0 - (violation_resolution_time_hours - 72.0) / 168.0 * 30.0).max(0.0) // 0-30
        };

        let approval_score = if approval_turnaround_hours <= 0.0 {
            100.0
        } else if approval_turnaround_hours <= 4.0 {
            100.0
        } else if approval_turnaround_hours <= 24.0 {
            100.0 - ((approval_turnaround_hours - 4.0) / 20.0 * 30.0)
        } else {
            (70.0 - (approval_turnaround_hours - 24.0) / 48.0 * 70.0).max(0.0)
        };

        let overall_score = (policy_coverage * 0.20)
            + (compliance_rate * 0.25)
            + (resolution_score * 0.15)
            + (approval_score * 0.10)
            + (audit_completeness * 0.15)
            + (policy_freshness * 0.15);

        // Determine trend by comparing with the previous score
        let previous_score: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT overall_score FROM governance_health_scores
            WHERE domain_id = $1 AND tenant_id = $2
            ORDER BY assessed_at DESC
            LIMIT 1
            "#
        )
        .bind(domain_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let trend = match previous_score {
            Some((prev,)) if overall_score > prev + 2.0 => "improving".to_string(),
            Some((prev,)) if overall_score < prev - 2.0 => "declining".to_string(),
            _ => "stable".to_string(),
        };

        let dimensions = HealthDimensions {
            policy_coverage,
            compliance_rate,
            violation_resolution_time_hours,
            approval_turnaround_hours,
            audit_completeness,
            policy_freshness,
        };

        // Store the score
        sqlx::query(
            r#"
            INSERT INTO governance_health_scores
                (domain_id, tenant_id, overall_score, policy_coverage, compliance_rate,
                 violation_resolution_time_hours, approval_turnaround_hours,
                 audit_completeness, policy_freshness, trend)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(domain_id)
        .bind(tenant_id)
        .bind(overall_score)
        .bind(policy_coverage)
        .bind(compliance_rate)
        .bind(violation_resolution_time_hours)
        .bind(approval_turnaround_hours)
        .bind(audit_completeness)
        .bind(policy_freshness)
        .bind(&trend)
        .execute(&self.pool)
        .await?;

        Ok(GovernanceHealthScore {
            domain_id,
            domain_name: domain.name,
            overall_score,
            dimensions,
            last_assessed_at: Utc::now(),
            trend,
        })
    }

    /// Get health scores for all domains
    pub async fn get_health_scores(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<GovernanceHealthScore>> {
        let domains = self.list_domains(tenant_id).await?;
        let mut scores = Vec::new();

        for domain in domains {
            // Try to get the most recent stored score first
            let stored: Option<HealthScoreRecord> = sqlx::query_as(
                r#"
                SELECT id, domain_id, tenant_id, overall_score, policy_coverage,
                       compliance_rate, violation_resolution_time_hours,
                       approval_turnaround_hours, audit_completeness, policy_freshness,
                       trend, assessed_at
                FROM governance_health_scores
                WHERE domain_id = $1 AND tenant_id = $2
                ORDER BY assessed_at DESC
                LIMIT 1
                "#
            )
            .bind(domain.id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

            let score = if let Some(record) = stored {
                GovernanceHealthScore {
                    domain_id: domain.id,
                    domain_name: domain.name.clone(),
                    overall_score: record.overall_score,
                    dimensions: HealthDimensions {
                        policy_coverage: record.policy_coverage,
                        compliance_rate: record.compliance_rate,
                        violation_resolution_time_hours: record.violation_resolution_time_hours,
                        approval_turnaround_hours: record.approval_turnaround_hours,
                        audit_completeness: record.audit_completeness,
                        policy_freshness: record.policy_freshness,
                    },
                    last_assessed_at: record.assessed_at,
                    trend: record.trend,
                }
            } else {
                // Calculate fresh if no stored score
                self.calculate_health_score(tenant_id, domain.id).await?
            };

            scores.push(score);
        }

        Ok(scores)
    }

    /// Get health trends for a domain over time
    pub async fn get_health_trends(
        &self,
        tenant_id: Uuid,
        domain_id: Uuid,
        days: i64,
    ) -> Result<Vec<HealthScoreRecord>> {
        let records = sqlx::query_as::<_, HealthScoreRecord>(
            r#"
            SELECT id, domain_id, tenant_id, overall_score, policy_coverage,
                   compliance_rate, violation_resolution_time_hours,
                   approval_turnaround_hours, audit_completeness, policy_freshness,
                   trend, assessed_at
            FROM governance_health_scores
            WHERE domain_id = $1 AND tenant_id = $2
              AND assessed_at > NOW() - make_interval(days => $3)
            ORDER BY assessed_at ASC
            "#
        )
        .bind(domain_id)
        .bind(tenant_id)
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    // ========================================================================
    // Private Helpers
    // ========================================================================

    /// Validate Rego policy syntax
    fn validate_rego(&self, rego: &str) -> Result<()> {
        let mut engine = Engine::new();
        engine.add_policy(
            "validation.rego".to_string(),
            rego.to_string(),
        ).map_err(|e| anyhow!("Invalid Rego policy: {}", e))?;
        Ok(())
    }

    /// Validate enforcement mode string
    fn validate_enforcement_mode(&self, mode: &str) -> Result<()> {
        match mode {
            "enforce" | "warn" | "audit" => Ok(()),
            _ => Err(anyhow!(
                "Invalid enforcement mode '{}'. Must be one of: enforce, warn, audit",
                mode
            )),
        }
    }

    /// Get enforcement strength as a numeric value for comparison
    fn enforcement_strength(&self, mode: &str) -> u8 {
        match mode {
            "enforce" => 3,
            "warn" => 2,
            "audit" => 1,
            _ => 0,
        }
    }

    /// Find domains whose resource patterns match a given resource
    async fn find_domains_for_resource(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<GovernanceDomain>> {
        let all_domains = self.list_domains(tenant_id).await?;
        let resource_string = format!("{}/{}", resource_type, resource_id);

        let matching: Vec<GovernanceDomain> = all_domains
            .into_iter()
            .filter(|d| {
                d.resource_patterns.iter().any(|pattern| {
                    glob_match(pattern, &resource_string)
                })
            })
            .collect();

        Ok(matching)
    }

    /// Get domain policies that extend a specific enterprise policy
    async fn get_extending_policies(
        &self,
        tenant_id: Uuid,
        domain_id: Uuid,
        enterprise_policy_id: Uuid,
    ) -> Result<Vec<DomainPolicy>> {
        let policies = sqlx::query_as::<_, DomainPolicy>(
            r#"
            SELECT id, domain_id, tenant_id, name, description, policy_type, rego_rule,
                   enforcement_mode, severity, extends_policy_id, enabled, created_at, updated_at
            FROM domain_policies
            WHERE tenant_id = $1 AND domain_id = $2 AND extends_policy_id = $3 AND enabled = true
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .bind(domain_id)
        .bind(enterprise_policy_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(policies)
    }

    /// Walk up the domain hierarchy and collect policies that extend an enterprise policy
    async fn collect_ancestor_policies(
        &self,
        tenant_id: Uuid,
        domain: &GovernanceDomain,
        enterprise_policy_id: Uuid,
    ) -> Result<Vec<(String, DomainPolicy)>> {
        let mut result = Vec::new();
        let mut current_parent_id = domain.parent_domain_id;

        while let Some(parent_id) = current_parent_id {
            if let Some(parent) = self.get_domain(tenant_id, parent_id).await? {
                let policies = self.get_extending_policies(
                    tenant_id,
                    parent.id,
                    enterprise_policy_id,
                ).await?;

                for p in policies {
                    result.push((parent.name.clone(), p));
                }

                current_parent_id = parent.parent_domain_id;
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// Evaluate a single Rego policy against resource data
    fn evaluate_single_rego(
        &self,
        rego_rule: &str,
        resource_data: &serde_json::Value,
    ) -> Result<(bool, Vec<String>)> {
        let mut engine = Engine::new();

        engine.add_policy(
            "policy.rego".to_string(),
            rego_rule.to_string(),
        )?;

        let input = regorus::Value::from_json_str(&resource_data.to_string())?;
        engine.set_input(input);

        let query_result = engine.eval_query("data.policy.deny".to_string(), false)?;

        let mut violations = Vec::new();
        let mut passed = true;

        if let Some(bindings) = query_result.result.first() {
            for expr in &bindings.expressions {
                if let regorus::Value::Set(set) = &expr.value {
                    for item in set.iter() {
                        if let regorus::Value::String(msg) = item {
                            violations.push(msg.to_string());
                            passed = false;
                        }
                    }
                }
            }
        }

        Ok((passed, violations))
    }
}

// ============================================================================
// Glob Matching Utility
// ============================================================================

/// Simple glob pattern matching supporting * and ** wildcards
fn glob_match(pattern: &str, input: &str) -> bool {
    // Handle exact match
    if pattern == input {
        return true;
    }

    // Handle ** (matches everything)
    if pattern == "**" {
        return true;
    }

    // Split pattern and input by /
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let input_parts: Vec<&str> = input.split('/').collect();

    glob_match_parts(&pattern_parts, &input_parts)
}

fn glob_match_parts(pattern: &[&str], input: &[&str]) -> bool {
    if pattern.is_empty() && input.is_empty() {
        return true;
    }
    if pattern.is_empty() {
        return false;
    }

    let pat = pattern[0];

    // ** matches zero or more path segments
    if pat == "**" {
        // Try matching remaining pattern against current and subsequent input positions
        for i in 0..=input.len() {
            if glob_match_parts(&pattern[1..], &input[i..]) {
                return true;
            }
        }
        return false;
    }

    if input.is_empty() {
        return false;
    }

    // * matches any single segment
    if pat == "*" {
        return glob_match_parts(&pattern[1..], &input[1..]);
    }

    // Check if segment matches (with * within segment)
    if segment_matches(pat, input[0]) {
        return glob_match_parts(&pattern[1..], &input[1..]);
    }

    false
}

/// Match a single segment with possible * wildcards within it
fn segment_matches(pattern: &str, input: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == input;
    }

    // Simple wildcard matching within a segment
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.is_empty() {
        return true;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = input[pos..].find(part) {
            if i == 0 && found != 0 {
                return false; // First part must match at start
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    // If pattern doesn't end with *, the match must reach the end
    if !pattern.ends_with('*') {
        return pos == input.len();
    }

    true
}

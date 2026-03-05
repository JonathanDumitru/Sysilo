use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regorus::Engine;

/// Policy scope - what resource type the policy applies to
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScope {
    Integration,
    Connection,
    Agent,
    Asset,
    DataEntity,
    All,
}

impl PolicyScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicyScope::Integration => "integration",
            PolicyScope::Connection => "connection",
            PolicyScope::Agent => "agent",
            PolicyScope::Asset => "asset",
            PolicyScope::DataEntity => "data_entity",
            PolicyScope::All => "all",
        }
    }
}

/// Policy enforcement mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    Enforce,  // Block violating actions
    Warn,     // Allow but warn
    Audit,    // Log only, no action
}

impl EnforcementMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnforcementMode::Enforce => "enforce",
            EnforcementMode::Warn => "warn",
            EnforcementMode::Audit => "audit",
        }
    }
}

/// Policy severity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl PolicySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicySeverity::Critical => "critical",
            PolicySeverity::High => "high",
            PolicySeverity::Medium => "medium",
            PolicySeverity::Low => "low",
            PolicySeverity::Info => "info",
        }
    }
}

/// A policy definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Policy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rego_policy: String,
    pub scope: String,
    pub enforcement_mode: String,
    pub severity: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a policy
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub rego_policy: String,
    pub scope: PolicyScope,
    pub enforcement_mode: EnforcementMode,
    pub severity: PolicySeverity,
}

/// Request to update a policy
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rego_policy: Option<String>,
    pub scope: Option<PolicyScope>,
    pub enforcement_mode: Option<EnforcementMode>,
    pub severity: Option<PolicySeverity>,
    pub enabled: Option<bool>,
}

/// Policy violation record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PolicyViolation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub policy_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub details: serde_json::Value,
    pub status: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Result of policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub passed: bool,
    pub violations: Vec<String>,
    pub enforcement_mode: String,
    pub severity: String,
}

/// Request to evaluate policies
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluatePoliciesRequest {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub resource_data: serde_json::Value,
}

/// Service for managing policies
pub struct PoliciesService {
    pool: PgPool,
}

impl PoliciesService {
    /// Create a new policies service
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

    /// List policies
    pub async fn list_policies(
        &self,
        tenant_id: Uuid,
        scope: Option<String>,
        enabled_only: bool,
    ) -> Result<Vec<Policy>> {
        let policies = if enabled_only {
            sqlx::query_as::<_, Policy>(
                r#"
                SELECT id, tenant_id, name, description, rego_policy, scope,
                       enforcement_mode, severity, enabled, created_at, updated_at
                FROM policies
                WHERE tenant_id = $1
                  AND enabled = true
                  AND ($2::text IS NULL OR scope = $2 OR scope = 'all')
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .bind(&scope)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Policy>(
                r#"
                SELECT id, tenant_id, name, description, rego_policy, scope,
                       enforcement_mode, severity, enabled, created_at, updated_at
                FROM policies
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR scope = $2 OR scope = 'all')
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .bind(&scope)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(policies)
    }

    /// Get a single policy
    pub async fn get_policy(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Policy>> {
        let policy = sqlx::query_as::<_, Policy>(
            r#"
            SELECT id, tenant_id, name, description, rego_policy, scope,
                   enforcement_mode, severity, enabled, created_at, updated_at
            FROM policies
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(policy)
    }

    /// Create a policy
    pub async fn create_policy(
        &self,
        tenant_id: Uuid,
        req: CreatePolicyRequest,
    ) -> Result<Policy> {
        // Validate the Rego policy syntax
        self.validate_rego(&req.rego_policy)?;

        let policy = sqlx::query_as::<_, Policy>(
            r#"
            INSERT INTO policies
                (tenant_id, name, description, rego_policy, scope, enforcement_mode, severity)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, name, description, rego_policy, scope,
                      enforcement_mode, severity, enabled, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.rego_policy)
        .bind(req.scope.as_str())
        .bind(req.enforcement_mode.as_str())
        .bind(req.severity.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(policy)
    }

    /// Update a policy
    pub async fn update_policy(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdatePolicyRequest,
    ) -> Result<Option<Policy>> {
        let existing = self.get_policy(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let rego_policy_changed = req.rego_policy.is_some();
        let rego_policy = req.rego_policy.unwrap_or(existing.rego_policy);
        let scope = req.scope.map(|s| s.as_str().to_string()).unwrap_or(existing.scope);
        let enforcement_mode = req.enforcement_mode.map(|e| e.as_str().to_string()).unwrap_or(existing.enforcement_mode);
        let severity = req.severity.map(|s| s.as_str().to_string()).unwrap_or(existing.severity);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        // Validate the Rego policy if it changed
        if rego_policy_changed {
            self.validate_rego(&rego_policy)?;
        }

        let policy = sqlx::query_as::<_, Policy>(
            r#"
            UPDATE policies SET
                name = $1, description = $2, rego_policy = $3, scope = $4,
                enforcement_mode = $5, severity = $6, enabled = $7, updated_at = NOW()
            WHERE tenant_id = $8 AND id = $9
            RETURNING id, tenant_id, name, description, rego_policy, scope,
                      enforcement_mode, severity, enabled, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&rego_policy)
        .bind(&scope)
        .bind(&enforcement_mode)
        .bind(&severity)
        .bind(enabled)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(policy))
    }

    /// Delete a policy
    pub async fn delete_policy(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM policies WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Validate Rego policy syntax
    fn validate_rego(&self, rego: &str) -> Result<()> {
        let mut engine = Engine::new();
        engine.add_policy(
            "validation.rego".to_string(),
            rego.to_string(),
        ).map_err(|e| anyhow!("Invalid Rego policy: {}", e))?;
        Ok(())
    }

    /// Evaluate policies against a resource
    pub async fn evaluate_policies(
        &self,
        tenant_id: Uuid,
        req: EvaluatePoliciesRequest,
    ) -> Result<Vec<PolicyEvaluationResult>> {
        // Get applicable policies
        let policies = self.list_policies(
            tenant_id,
            Some(req.resource_type.clone()),
            true,
        ).await?;

        let mut results = Vec::new();

        for policy in policies {
            let result = self.evaluate_single_policy(&policy, &req.resource_data)?;

            // Record violation if policy failed
            if !result.passed && policy.enforcement_mode != "audit" {
                self.record_violation(
                    tenant_id,
                    policy.id,
                    &req.resource_type,
                    req.resource_id,
                    &result.violations,
                ).await?;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Evaluate a single policy
    pub fn evaluate_single_policy(
        &self,
        policy: &Policy,
        resource_data: &serde_json::Value,
    ) -> Result<PolicyEvaluationResult> {
        let mut engine = Engine::new();

        // Add the policy
        engine.add_policy(
            format!("{}.rego", policy.name),
            policy.rego_policy.clone(),
        )?;

        // Set input data
        let input = regorus::Value::from_json_str(&resource_data.to_string())?;
        engine.set_input(input);

        // Evaluate the policy
        // The policy should define a "deny" rule that returns violation messages
        let query_result = engine.eval_query("data.policy.deny".to_string(), false)?;

        let mut violations = Vec::new();
        let mut passed = true;

        // Extract violations from the result
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

        Ok(PolicyEvaluationResult {
            policy_id: policy.id,
            policy_name: policy.name.clone(),
            passed,
            violations,
            enforcement_mode: policy.enforcement_mode.clone(),
            severity: policy.severity.clone(),
        })
    }

    /// Record a policy violation
    async fn record_violation(
        &self,
        tenant_id: Uuid,
        policy_id: Uuid,
        resource_type: &str,
        resource_id: Uuid,
        violations: &[String],
    ) -> Result<PolicyViolation> {
        let details = serde_json::json!({
            "violations": violations
        });

        let violation = sqlx::query_as::<_, PolicyViolation>(
            r#"
            INSERT INTO policy_violations
                (tenant_id, policy_id, resource_type, resource_id, details)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, tenant_id, policy_id, resource_type, resource_id, details,
                      status, resolved_at, resolved_by, resolution_note, created_at
            "#
        )
        .bind(tenant_id)
        .bind(policy_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(&details)
        .fetch_one(&self.pool)
        .await?;

        Ok(violation)
    }

    /// List policy violations
    pub async fn list_violations(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        policy_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PolicyViolation>, i64)> {
        let violations = sqlx::query_as::<_, PolicyViolation>(
            r#"
            SELECT id, tenant_id, policy_id, resource_type, resource_id, details,
                   status, resolved_at, resolved_by, resolution_note, created_at
            FROM policy_violations
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR policy_id = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(tenant_id)
        .bind(&status)
        .bind(policy_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM policy_violations
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR policy_id = $3)
            "#
        )
        .bind(tenant_id)
        .bind(&status)
        .bind(policy_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((violations, total.0))
    }

    /// Resolve a violation
    pub async fn resolve_violation(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        resolved_by: Option<Uuid>,
        resolution_note: Option<String>,
    ) -> Result<Option<PolicyViolation>> {
        let violation = sqlx::query_as::<_, PolicyViolation>(
            r#"
            UPDATE policy_violations SET
                status = 'resolved',
                resolved_at = NOW(),
                resolved_by = $1,
                resolution_note = $2
            WHERE tenant_id = $3 AND id = $4 AND status = 'open'
            RETURNING id, tenant_id, policy_id, resource_type, resource_id, details,
                      status, resolved_at, resolved_by, resolution_note, created_at
            "#
        )
        .bind(resolved_by)
        .bind(&resolution_note)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(violation)
    }
}

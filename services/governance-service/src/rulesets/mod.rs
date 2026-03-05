use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::policies::{PolicyEvaluationResult, PoliciesService};

/// A ruleset groups related policies together
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ruleset {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub enabled: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A ruleset with its associated policy IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesetWithPolicies {
    #[serde(flatten)]
    pub ruleset: Ruleset,
    pub policy_ids: Vec<Uuid>,
}

/// Request to create a ruleset
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRulesetRequest {
    pub name: String,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub policy_ids: Option<Vec<Uuid>>,
}

/// Request to update a ruleset
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRulesetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub enabled: Option<bool>,
    pub policy_ids: Option<Vec<Uuid>>,
}

/// Service for managing rulesets
pub struct RulesetsService {
    pool: PgPool,
}

impl RulesetsService {
    /// Create a new rulesets service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// List rulesets for a tenant
    pub async fn list_rulesets(
        &self,
        tenant_id: Uuid,
        scope: Option<String>,
        enabled_only: bool,
    ) -> Result<Vec<Ruleset>> {
        let rulesets = if enabled_only {
            sqlx::query_as::<_, Ruleset>(
                r#"
                SELECT id, tenant_id, name, description, scope, enabled,
                       created_by, created_at, updated_at
                FROM rulesets
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
            sqlx::query_as::<_, Ruleset>(
                r#"
                SELECT id, tenant_id, name, description, scope, enabled,
                       created_by, created_at, updated_at
                FROM rulesets
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

        Ok(rulesets)
    }

    /// Get a single ruleset
    pub async fn get_ruleset(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Ruleset>> {
        let ruleset = sqlx::query_as::<_, Ruleset>(
            r#"
            SELECT id, tenant_id, name, description, scope, enabled,
                   created_by, created_at, updated_at
            FROM rulesets
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ruleset)
    }

    /// Get a ruleset with its associated policy IDs
    pub async fn get_ruleset_with_policies(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RulesetWithPolicies>> {
        let ruleset = self.get_ruleset(tenant_id, id).await?;
        match ruleset {
            Some(ruleset) => {
                let policy_ids = self.get_policy_ids(id).await?;
                Ok(Some(RulesetWithPolicies { ruleset, policy_ids }))
            }
            None => Ok(None),
        }
    }

    /// Get policy IDs for a ruleset, ordered by position
    async fn get_policy_ids(&self, ruleset_id: Uuid) -> Result<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT policy_id
            FROM ruleset_policies
            WHERE ruleset_id = $1
            ORDER BY position, added_at
            "#
        )
        .bind(ruleset_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Create a ruleset
    pub async fn create_ruleset(
        &self,
        tenant_id: Uuid,
        req: CreateRulesetRequest,
        created_by: Option<Uuid>,
    ) -> Result<RulesetWithPolicies> {
        let scope = req.scope.unwrap_or_else(|| "all".to_string());

        let ruleset = sqlx::query_as::<_, Ruleset>(
            r#"
            INSERT INTO rulesets (tenant_id, name, description, scope, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, tenant_id, name, description, scope, enabled,
                      created_by, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&scope)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        // Add policies if provided
        let policy_ids = if let Some(policy_ids) = req.policy_ids {
            self.set_policies(ruleset.id, &policy_ids).await?;
            policy_ids
        } else {
            Vec::new()
        };

        Ok(RulesetWithPolicies { ruleset, policy_ids })
    }

    /// Update a ruleset
    pub async fn update_ruleset(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateRulesetRequest,
    ) -> Result<Option<RulesetWithPolicies>> {
        let existing = self.get_ruleset(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let scope = req.scope.unwrap_or(existing.scope);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        let ruleset = sqlx::query_as::<_, Ruleset>(
            r#"
            UPDATE rulesets SET
                name = $1, description = $2, scope = $3, enabled = $4, updated_at = NOW()
            WHERE tenant_id = $5 AND id = $6
            RETURNING id, tenant_id, name, description, scope, enabled,
                      created_by, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&scope)
        .bind(enabled)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        // Update policies if provided
        let policy_ids = if let Some(policy_ids) = req.policy_ids {
            self.set_policies(id, &policy_ids).await?;
            policy_ids
        } else {
            self.get_policy_ids(id).await?
        };

        Ok(Some(RulesetWithPolicies { ruleset, policy_ids }))
    }

    /// Delete a ruleset
    pub async fn delete_ruleset(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM rulesets WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Set the policies for a ruleset (replaces existing associations)
    async fn set_policies(&self, ruleset_id: Uuid, policy_ids: &[Uuid]) -> Result<()> {
        // Remove existing associations
        sqlx::query("DELETE FROM ruleset_policies WHERE ruleset_id = $1")
            .bind(ruleset_id)
            .execute(&self.pool)
            .await?;

        // Insert new associations
        for (position, policy_id) in policy_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO ruleset_policies (ruleset_id, policy_id, position)
                VALUES ($1, $2, $3)
                ON CONFLICT (ruleset_id, policy_id) DO NOTHING
                "#
            )
            .bind(ruleset_id)
            .bind(policy_id)
            .bind(position as i32)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Evaluate all policies in a ruleset against a resource
    pub async fn evaluate_ruleset(
        &self,
        tenant_id: Uuid,
        ruleset_id: Uuid,
        resource_data: &serde_json::Value,
        policies_service: &PoliciesService,
    ) -> Result<Vec<PolicyEvaluationResult>> {
        let policy_ids = self.get_policy_ids(ruleset_id).await?;

        let mut results = Vec::new();
        for policy_id in policy_ids {
            if let Some(policy) = policies_service.get_policy(tenant_id, policy_id).await? {
                if policy.enabled {
                    let result = policies_service.evaluate_single_policy(&policy, resource_data)?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }
}

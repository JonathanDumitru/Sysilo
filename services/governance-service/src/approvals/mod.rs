use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Approval workflow definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalWorkflow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_conditions: serde_json::Value,
    pub stages: serde_json::Value,
    pub auto_approve_conditions: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single stage in an approval workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStage {
    pub name: String,
    pub approvers: Vec<Uuid>,        // User or group IDs
    pub approver_roles: Vec<String>, // Role names that can approve
    pub required_count: i32,         // How many approvals needed
    pub timeout_hours: Option<i32>,  // Auto-escalate after timeout
    pub escalation_to: Option<Uuid>, // Escalate to this user/group
}

/// Request to create a workflow
#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_conditions: serde_json::Value,
    pub stages: Vec<WorkflowStage>,
    pub auto_approve_conditions: Option<serde_json::Value>,
}

/// Request to update a workflow
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_conditions: Option<serde_json::Value>,
    pub stages: Option<Vec<WorkflowStage>>,
    pub auto_approve_conditions: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

/// Approval request status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
    Expired,
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            ApprovalStatus::Cancelled => "cancelled",
            ApprovalStatus::Expired => "expired",
        }
    }
}

/// An approval request
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub workflow_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub resource_snapshot: Option<serde_json::Value>,
    pub requester_id: Uuid,
    pub current_stage: i32,
    pub status: String,
    pub auto_approved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Request to create an approval request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApprovalRequestInput {
    pub workflow_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub resource_snapshot: Option<serde_json::Value>,
}

/// An approval decision
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalDecision {
    pub id: Uuid,
    pub request_id: Uuid,
    pub stage: i32,
    pub approver_id: Uuid,
    pub decision: String,
    pub comment: Option<String>,
    pub decided_at: DateTime<Utc>,
}

/// Decision type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Approved,
    Rejected,
}

impl Decision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Decision::Approved => "approved",
            Decision::Rejected => "rejected",
        }
    }
}

/// Request to make a decision on an approval
#[derive(Debug, Clone, Deserialize)]
pub struct DecideRequest {
    pub decision: Decision,
    pub comment: Option<String>,
}

/// Approval request with workflow details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequestWithWorkflow {
    pub request: ApprovalRequest,
    pub workflow_name: String,
    pub current_stage_name: String,
    pub decisions: Vec<ApprovalDecision>,
}

/// Service for managing approvals
pub struct ApprovalsService {
    pool: PgPool,
}

impl ApprovalsService {
    /// Create a new approvals service
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

    // ========================================================================
    // Workflows
    // ========================================================================

    /// List workflows
    pub async fn list_workflows(&self, tenant_id: Uuid) -> Result<Vec<ApprovalWorkflow>> {
        let workflows = sqlx::query_as::<_, ApprovalWorkflow>(
            r#"
            SELECT id, tenant_id, name, description, trigger_conditions, stages,
                   auto_approve_conditions, enabled, created_by, created_at, updated_at
            FROM approval_workflows
            WHERE tenant_id = $1
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(workflows)
    }

    /// Get a single workflow
    pub async fn get_workflow(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<ApprovalWorkflow>> {
        let workflow = sqlx::query_as::<_, ApprovalWorkflow>(
            r#"
            SELECT id, tenant_id, name, description, trigger_conditions, stages,
                   auto_approve_conditions, enabled, created_by, created_at, updated_at
            FROM approval_workflows
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workflow)
    }

    /// Create a workflow
    pub async fn create_workflow(
        &self,
        tenant_id: Uuid,
        req: CreateWorkflowRequest,
        created_by: Option<Uuid>,
    ) -> Result<ApprovalWorkflow> {
        let stages = serde_json::to_value(&req.stages)?;

        let workflow = sqlx::query_as::<_, ApprovalWorkflow>(
            r#"
            INSERT INTO approval_workflows
                (tenant_id, name, description, trigger_conditions, stages, auto_approve_conditions, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, name, description, trigger_conditions, stages,
                      auto_approve_conditions, enabled, created_by, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.trigger_conditions)
        .bind(&stages)
        .bind(&req.auto_approve_conditions)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(workflow)
    }

    /// Update a workflow
    pub async fn update_workflow(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateWorkflowRequest,
    ) -> Result<Option<ApprovalWorkflow>> {
        let existing = self.get_workflow(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let trigger_conditions = req.trigger_conditions.unwrap_or(existing.trigger_conditions);
        let stages = req.stages
            .map(|s| serde_json::to_value(&s).unwrap())
            .unwrap_or(existing.stages);
        let auto_approve_conditions = req.auto_approve_conditions.or(existing.auto_approve_conditions);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        let workflow = sqlx::query_as::<_, ApprovalWorkflow>(
            r#"
            UPDATE approval_workflows SET
                name = $1, description = $2, trigger_conditions = $3, stages = $4,
                auto_approve_conditions = $5, enabled = $6, updated_at = NOW()
            WHERE tenant_id = $7 AND id = $8
            RETURNING id, tenant_id, name, description, trigger_conditions, stages,
                      auto_approve_conditions, enabled, created_by, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&trigger_conditions)
        .bind(&stages)
        .bind(&auto_approve_conditions)
        .bind(enabled)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(workflow))
    }

    /// Delete a workflow
    pub async fn delete_workflow(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM approval_workflows WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Approval Requests
    // ========================================================================

    /// List approval requests
    pub async fn list_requests(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        requester_id: Option<Uuid>,
        approver_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ApprovalRequest>, i64)> {
        // If approver_id is specified, filter to requests pending their approval
        let requests = if let Some(approver) = approver_id {
            sqlx::query_as::<_, ApprovalRequest>(
                r#"
                SELECT ar.id, ar.tenant_id, ar.workflow_id, ar.resource_type,
                       ar.resource_id, ar.resource_snapshot, ar.requester_id,
                       ar.current_stage, ar.status, ar.auto_approved,
                       ar.created_at, ar.updated_at, ar.completed_at
                FROM approval_requests ar
                JOIN approval_workflows aw ON aw.id = ar.workflow_id
                WHERE ar.tenant_id = $1
                  AND ar.status = 'pending'
                  AND ($2::text IS NULL OR ar.status = $2)
                  AND (
                      -- Check if approver is in the current stage
                      EXISTS (
                          SELECT 1 FROM jsonb_array_elements(aw.stages) WITH ORDINALITY AS stage(s, idx)
                          WHERE (idx - 1) = ar.current_stage
                          AND $5::uuid = ANY(
                              SELECT jsonb_array_elements_text(s->'approvers')::uuid
                          )
                      )
                  )
                ORDER BY ar.created_at DESC
                LIMIT $3 OFFSET $4
                "#
            )
            .bind(tenant_id)
            .bind(&status)
            .bind(limit)
            .bind(offset)
            .bind(approver)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ApprovalRequest>(
                r#"
                SELECT id, tenant_id, workflow_id, resource_type, resource_id,
                       resource_snapshot, requester_id, current_stage, status,
                       auto_approved, created_at, updated_at, completed_at
                FROM approval_requests
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::uuid IS NULL OR requester_id = $3)
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
                "#
            )
            .bind(tenant_id)
            .bind(&status)
            .bind(requester_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM approval_requests
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR requester_id = $3)
            "#
        )
        .bind(tenant_id)
        .bind(&status)
        .bind(requester_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((requests, total.0))
    }

    /// Get a single approval request with details
    pub async fn get_request(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ApprovalRequestWithWorkflow>> {
        let request = sqlx::query_as::<_, ApprovalRequest>(
            r#"
            SELECT id, tenant_id, workflow_id, resource_type, resource_id,
                   resource_snapshot, requester_id, current_stage, status,
                   auto_approved, created_at, updated_at, completed_at
            FROM approval_requests
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if request.is_none() {
            return Ok(None);
        }
        let request = request.unwrap();

        // Get workflow
        let workflow = self.get_workflow(tenant_id, request.workflow_id).await?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        // Get decisions
        let decisions = self.get_decisions(request.id).await?;

        // Parse stages to get current stage name
        let stages: Vec<WorkflowStage> = serde_json::from_value(workflow.stages)?;
        let current_stage_name = stages
            .get(request.current_stage as usize)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(Some(ApprovalRequestWithWorkflow {
            request,
            workflow_name: workflow.name,
            current_stage_name,
            decisions,
        }))
    }

    /// Create an approval request
    pub async fn create_request(
        &self,
        tenant_id: Uuid,
        requester_id: Uuid,
        req: CreateApprovalRequestInput,
    ) -> Result<ApprovalRequest> {
        // Get workflow to validate
        let workflow = self.get_workflow(tenant_id, req.workflow_id).await?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        if !workflow.enabled {
            return Err(anyhow::anyhow!("Workflow is disabled"));
        }

        // Check for auto-approval conditions
        let auto_approved = self.check_auto_approve(&workflow, &req.resource_snapshot);

        let request = sqlx::query_as::<_, ApprovalRequest>(
            r#"
            INSERT INTO approval_requests
                (tenant_id, workflow_id, resource_type, resource_id, resource_snapshot,
                 requester_id, auto_approved, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, tenant_id, workflow_id, resource_type, resource_id,
                      resource_snapshot, requester_id, current_stage, status,
                      auto_approved, created_at, updated_at, completed_at
            "#
        )
        .bind(tenant_id)
        .bind(req.workflow_id)
        .bind(&req.resource_type)
        .bind(req.resource_id)
        .bind(&req.resource_snapshot)
        .bind(requester_id)
        .bind(auto_approved)
        .bind(if auto_approved { "approved" } else { "pending" })
        .fetch_one(&self.pool)
        .await?;

        Ok(request)
    }

    /// Check if auto-approval conditions are met
    ///
    /// Evaluates conditions against the resource snapshot.
    /// Supported condition formats:
    /// - `{"field": "value"}` - exact match
    /// - `{"field": {"$eq": "value"}}` - exact match
    /// - `{"field": {"$ne": "value"}}` - not equal
    /// - `{"field": {"$gt": 100}}` - greater than (numeric)
    /// - `{"field": {"$lt": 100}}` - less than (numeric)
    /// - `{"field": {"$in": ["a", "b"]}}` - value in array
    /// - `{"$and": [{...}, {...}]}` - all conditions must match
    /// - `{"$or": [{...}, {...}]}` - any condition must match
    fn check_auto_approve(
        &self,
        workflow: &ApprovalWorkflow,
        resource_snapshot: &Option<serde_json::Value>,
    ) -> bool {
        let conditions = match &workflow.auto_approve_conditions {
            Some(c) => c,
            None => return false,
        };

        let snapshot = match resource_snapshot {
            Some(s) => s,
            None => return false, // Can't auto-approve without resource data
        };

        self.evaluate_condition(conditions, snapshot)
    }

    /// Recursively evaluate a condition against a resource snapshot
    fn evaluate_condition(
        &self,
        condition: &serde_json::Value,
        snapshot: &serde_json::Value,
    ) -> bool {
        let obj = match condition.as_object() {
            Some(o) => o,
            None => return false,
        };

        for (key, value) in obj {
            match key.as_str() {
                // Logical operators
                "$and" => {
                    if let Some(arr) = value.as_array() {
                        if !arr.iter().all(|c| self.evaluate_condition(c, snapshot)) {
                            return false;
                        }
                    }
                }
                "$or" => {
                    if let Some(arr) = value.as_array() {
                        if !arr.iter().any(|c| self.evaluate_condition(c, snapshot)) {
                            return false;
                        }
                    }
                }
                // Field conditions
                field => {
                    let field_value = self.get_field_value(snapshot, field);
                    if !self.evaluate_field_condition(value, &field_value) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Get a field value from the snapshot (supports nested paths like "risk.level")
    fn get_field_value(&self, snapshot: &serde_json::Value, field: &str) -> serde_json::Value {
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = snapshot;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return serde_json::Value::Null,
            }
        }

        current.clone()
    }

    /// Evaluate a field condition (value can be direct value or operator object)
    fn evaluate_field_condition(
        &self,
        condition: &serde_json::Value,
        field_value: &serde_json::Value,
    ) -> bool {
        // If condition is an object with operators
        if let Some(obj) = condition.as_object() {
            for (op, expected) in obj {
                let matches = match op.as_str() {
                    "$eq" => field_value == expected,
                    "$ne" => field_value != expected,
                    "$gt" => {
                        match (field_value.as_f64(), expected.as_f64()) {
                            (Some(fv), Some(ev)) => fv > ev,
                            _ => false,
                        }
                    }
                    "$gte" => {
                        match (field_value.as_f64(), expected.as_f64()) {
                            (Some(fv), Some(ev)) => fv >= ev,
                            _ => false,
                        }
                    }
                    "$lt" => {
                        match (field_value.as_f64(), expected.as_f64()) {
                            (Some(fv), Some(ev)) => fv < ev,
                            _ => false,
                        }
                    }
                    "$lte" => {
                        match (field_value.as_f64(), expected.as_f64()) {
                            (Some(fv), Some(ev)) => fv <= ev,
                            _ => false,
                        }
                    }
                    "$in" => {
                        if let Some(arr) = expected.as_array() {
                            arr.contains(field_value)
                        } else {
                            false
                        }
                    }
                    "$nin" => {
                        if let Some(arr) = expected.as_array() {
                            !arr.contains(field_value)
                        } else {
                            false
                        }
                    }
                    "$exists" => {
                        let should_exist = expected.as_bool().unwrap_or(true);
                        let exists = !field_value.is_null();
                        exists == should_exist
                    }
                    "$regex" => {
                        if let (Some(pattern), Some(text)) = (expected.as_str(), field_value.as_str()) {
                            // Simple substring match (full regex would require regex crate)
                            text.contains(pattern)
                        } else {
                            false
                        }
                    }
                    _ => false, // Unknown operator
                };

                if !matches {
                    return false;
                }
            }
            true
        } else {
            // Direct value comparison (implicit $eq)
            field_value == condition
        }
    }

    /// Make a decision on an approval request
    pub async fn decide(
        &self,
        tenant_id: Uuid,
        request_id: Uuid,
        approver_id: Uuid,
        req: DecideRequest,
    ) -> Result<ApprovalRequest> {
        // Get the request
        let request = sqlx::query_as::<_, ApprovalRequest>(
            r#"
            SELECT id, tenant_id, workflow_id, resource_type, resource_id,
                   resource_snapshot, requester_id, current_stage, status,
                   auto_approved, created_at, updated_at, completed_at
            FROM approval_requests
            WHERE tenant_id = $1 AND id = $2 AND status = 'pending'
            "#
        )
        .bind(tenant_id)
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Request not found or not pending"))?;

        // Get workflow
        let workflow = self.get_workflow(tenant_id, request.workflow_id).await?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        let stages: Vec<WorkflowStage> = serde_json::from_value(workflow.stages)?;
        let current_stage = stages.get(request.current_stage as usize)
            .ok_or_else(|| anyhow::anyhow!("Invalid stage"))?;

        // Record the decision
        sqlx::query(
            r#"
            INSERT INTO approval_decisions
                (request_id, stage, approver_id, decision, comment)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(request_id)
        .bind(request.current_stage)
        .bind(approver_id)
        .bind(req.decision.as_str())
        .bind(&req.comment)
        .execute(&self.pool)
        .await?;

        // Handle rejection - immediately reject the whole request
        if matches!(req.decision, Decision::Rejected) {
            let updated = sqlx::query_as::<_, ApprovalRequest>(
                r#"
                UPDATE approval_requests SET
                    status = 'rejected',
                    completed_at = NOW(),
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, tenant_id, workflow_id, resource_type, resource_id,
                          resource_snapshot, requester_id, current_stage, status,
                          auto_approved, created_at, updated_at, completed_at
                "#
            )
            .bind(request_id)
            .fetch_one(&self.pool)
            .await?;

            return Ok(updated);
        }

        // Count approvals for current stage
        let approval_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM approval_decisions
            WHERE request_id = $1 AND stage = $2 AND decision = 'approved'
            "#
        )
        .bind(request_id)
        .bind(request.current_stage)
        .fetch_one(&self.pool)
        .await?;

        // Check if we have enough approvals to advance
        if approval_count.0 >= current_stage.required_count as i64 {
            // Check if there are more stages
            if (request.current_stage as usize) < stages.len() - 1 {
                // Advance to next stage
                let next_stage = request.current_stage + 1;

                let updated = sqlx::query_as::<_, ApprovalRequest>(
                    r#"
                    UPDATE approval_requests SET
                        current_stage = $1,
                        updated_at = NOW()
                    WHERE id = $2
                    RETURNING id, tenant_id, workflow_id, resource_type, resource_id,
                              resource_snapshot, requester_id, current_stage, status,
                              auto_approved, created_at, updated_at, completed_at
                    "#
                )
                .bind(next_stage)
                .bind(request_id)
                .fetch_one(&self.pool)
                .await?;

                return Ok(updated);
            } else {
                // All stages complete - approve the request
                let updated = sqlx::query_as::<_, ApprovalRequest>(
                    r#"
                    UPDATE approval_requests SET
                        status = 'approved',
                        completed_at = NOW(),
                        updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, tenant_id, workflow_id, resource_type, resource_id,
                              resource_snapshot, requester_id, current_stage, status,
                              auto_approved, created_at, updated_at, completed_at
                    "#
                )
                .bind(request_id)
                .fetch_one(&self.pool)
                .await?;

                return Ok(updated);
            }
        }

        // Not enough approvals yet, return current state
        Ok(request)
    }

    /// Get decisions for a request
    async fn get_decisions(&self, request_id: Uuid) -> Result<Vec<ApprovalDecision>> {
        let decisions = sqlx::query_as::<_, ApprovalDecision>(
            r#"
            SELECT id, request_id, stage, approver_id, decision, comment, decided_at
            FROM approval_decisions
            WHERE request_id = $1
            ORDER BY decided_at ASC
            "#
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(decisions)
    }

    /// Cancel an approval request (by requester)
    pub async fn cancel_request(
        &self,
        tenant_id: Uuid,
        request_id: Uuid,
        requester_id: Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE approval_requests SET
                status = 'cancelled',
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND requester_id = $3 AND status = 'pending'
            "#
        )
        .bind(tenant_id)
        .bind(request_id)
        .bind(requester_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Expire timed-out requests (called by background job)
    /// Note: Since the schema doesn't have expires_at, this method uses created_at + workflow timeout
    pub async fn expire_requests(&self, default_timeout_hours: i64) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE approval_requests SET
                status = 'expired',
                completed_at = NOW(),
                updated_at = NOW()
            WHERE status = 'pending'
              AND created_at < NOW() - make_interval(hours => $1)
            "#
        )
        .bind(default_timeout_hours)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

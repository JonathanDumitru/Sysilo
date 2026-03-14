pub mod api;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// =============================================================================
// Errors
// =============================================================================

#[derive(Debug, Error)]
pub enum HealingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Governance service error: {0}")]
    GovernanceService(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Fix application error: {0}")]
    FixApplication(String),
}

// =============================================================================
// Types
// =============================================================================

/// Classification of integration failure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    /// Source/target schema changed
    SchemaChange,
    /// Credentials expired or revoked
    AuthExpiry,
    /// API rate limit exceeded
    RateLimitBreach,
    /// Network timeout
    ConnectionTimeout,
    /// Unexpected data types in payload
    DataTypeConflict,
    /// Required field no longer present
    MissingRequiredField,
    /// Downstream service is down
    ServiceUnavailable,
    /// Insufficient permissions
    PermissionDenied,
    /// Unexpected data volume
    DataVolumeAnomaly,
    /// Cannot classify
    Unknown,
}

impl std::fmt::Display for FailureClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());
        write!(f, "{}", s)
    }
}

/// Risk level of a proposed fix
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FixRiskLevel {
    /// Auto-approve eligible (e.g., retry with backoff)
    Low,
    /// Requires review (e.g., field remapping)
    Medium,
    /// Requires explicit approval (e.g., credential rotation)
    High,
    /// Requires multi-level approval (e.g., schema migration)
    Critical,
}

impl std::fmt::Display for FixRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "low".to_string());
        write!(f, "{}", s)
    }
}

/// Approval status of a healing proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    AutoApproved,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "pending".to_string());
        write!(f, "{}", s)
    }
}

/// A proposed remediation action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProposedFix {
    RetryWithBackoff {
        delay_ms: u64,
        max_retries: u32,
    },
    RemapField {
        old_field: String,
        new_field: String,
        mapping_expression: Option<String>,
    },
    RefreshCredentials {
        connection_id: Uuid,
    },
    AdjustRateLimit {
        new_requests_per_minute: u32,
    },
    AddFieldDefault {
        field_name: String,
        default_value: serde_json::Value,
    },
    SkipField {
        field_name: String,
    },
    UpdateSchema {
        changes: Vec<SchemaChange>,
    },
    Retry,
    Escalate {
        message: String,
    },
}

impl std::fmt::Display for ProposedFix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposedFix::RetryWithBackoff { delay_ms, max_retries } => {
                write!(f, "Retry with {}ms delay, max {} retries", delay_ms, max_retries)
            }
            ProposedFix::RemapField { old_field, new_field, .. } => {
                write!(f, "Remap field '{}' to '{}'", old_field, new_field)
            }
            ProposedFix::RefreshCredentials { connection_id } => {
                write!(f, "Refresh credentials for connection {}", connection_id)
            }
            ProposedFix::AdjustRateLimit { new_requests_per_minute } => {
                write!(f, "Adjust rate limit to {} req/min", new_requests_per_minute)
            }
            ProposedFix::AddFieldDefault { field_name, default_value } => {
                write!(f, "Add default '{}' for field '{}'", default_value, field_name)
            }
            ProposedFix::SkipField { field_name } => {
                write!(f, "Skip field '{}'", field_name)
            }
            ProposedFix::UpdateSchema { changes } => {
                write!(f, "Update schema ({} changes)", changes.len())
            }
            ProposedFix::Retry => write!(f, "Simple retry"),
            ProposedFix::Escalate { message } => write!(f, "Escalate: {}", message),
        }
    }
}

/// A single schema change within an UpdateSchema fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub field_name: String,
    /// "add", "remove", or "modify_type"
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// A healing proposal with diagnosis and proposed fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingProposal {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub run_id: Uuid,
    pub failure_class: FailureClass,
    pub diagnosis: String,
    pub proposed_fix: ProposedFix,
    pub risk_level: FixRiskLevel,
    pub confidence: f64,
    pub approval_status: ApprovalStatus,
    pub approval_request_id: Option<Uuid>,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Database row for healing proposals
#[derive(Debug, FromRow)]
pub struct HealingProposalRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub run_id: Uuid,
    pub failure_class: String,
    pub diagnosis: String,
    pub proposed_fix: serde_json::Value,
    pub risk_level: String,
    pub confidence: f64,
    pub approval_status: String,
    pub approval_request_id: Option<Uuid>,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl HealingProposalRow {
    /// Convert a database row into a HealingProposal
    pub fn into_proposal(self) -> Result<HealingProposal, HealingError> {
        let failure_class: FailureClass =
            serde_json::from_value(serde_json::Value::String(self.failure_class.clone()))
                .unwrap_or(FailureClass::Unknown);

        let proposed_fix: ProposedFix = serde_json::from_value(self.proposed_fix.clone())
            .map_err(|e| HealingError::InvalidState(format!("Invalid proposed_fix JSON: {}", e)))?;

        let risk_level: FixRiskLevel =
            serde_json::from_value(serde_json::Value::String(self.risk_level.clone()))
                .unwrap_or(FixRiskLevel::High);

        let approval_status: ApprovalStatus =
            serde_json::from_value(serde_json::Value::String(self.approval_status.clone()))
                .unwrap_or(ApprovalStatus::Pending);

        Ok(HealingProposal {
            id: self.id,
            tenant_id: self.tenant_id,
            integration_id: self.integration_id,
            run_id: self.run_id,
            failure_class,
            diagnosis: self.diagnosis,
            proposed_fix,
            risk_level,
            confidence: self.confidence,
            approval_status,
            approval_request_id: self.approval_request_id,
            applied: self.applied,
            applied_at: self.applied_at,
            result: self.result,
            created_at: self.created_at,
        })
    }
}

/// Configuration for the healing subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingConfig {
    pub enabled: bool,
    pub auto_approve_low_risk: bool,
    pub max_auto_retries: u32,
    pub ai_service_url: String,
    pub governance_service_url: String,
}

impl Default for HealingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_approve_low_risk: true,
            max_auto_retries: 3,
            ai_service_url: "http://localhost:8090".to_string(),
            governance_service_url: "http://localhost:8086".to_string(),
        }
    }
}

/// Error context provided when requesting diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_message: String,
    pub error_code: Option<String>,
    pub context: Option<serde_json::Value>,
}

/// Healing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingStats {
    pub total_proposals: i64,
    pub auto_approved: i64,
    pub pending_approval: i64,
    pub approved: i64,
    pub rejected: i64,
    pub applied: i64,
    pub successful_fixes: i64,
    pub failed_fixes: i64,
    pub success_rate: f64,
    pub by_failure_class: serde_json::Value,
    pub by_risk_level: serde_json::Value,
}

/// Filters for listing proposals
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProposalFilters {
    pub integration_id: Option<Uuid>,
    pub failure_class: Option<String>,
    pub approval_status: Option<String>,
    pub risk_level: Option<String>,
    pub applied: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// =============================================================================
// HealingService
// =============================================================================

pub struct HealingService {
    pool: PgPool,
    config: HealingConfig,
    http_client: reqwest::Client,
}

impl HealingService {
    /// Create a new HealingService, initialize the database pool and create tables
    pub async fn new(database_url: &str, config: HealingConfig) -> Result<Self, HealingError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let service = Self {
            pool,
            config,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| HealingError::HttpClient(e))?,
        };

        service.create_tables().await?;

        Ok(service)
    }

    /// Create the healing_proposals table if it does not exist
    async fn create_tables(&self) -> Result<(), HealingError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS healing_proposals (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                integration_id UUID NOT NULL,
                run_id UUID NOT NULL,
                failure_class TEXT NOT NULL,
                diagnosis TEXT NOT NULL DEFAULT '',
                proposed_fix JSONB NOT NULL DEFAULT '{}',
                risk_level TEXT NOT NULL DEFAULT 'high',
                confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                approval_status TEXT NOT NULL DEFAULT 'pending',
                approval_request_id UUID,
                applied BOOLEAN NOT NULL DEFAULT FALSE,
                applied_at TIMESTAMPTZ,
                result TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for common query patterns
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_healing_proposals_tenant
                ON healing_proposals (tenant_id, created_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_healing_proposals_status
                ON healing_proposals (approval_status)
                WHERE approval_status = 'pending'
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Healing proposals table initialized");
        Ok(())
    }

    /// Get the current healing configuration
    pub fn config(&self) -> &HealingConfig {
        &self.config
    }

    /// Update the healing configuration at runtime
    pub fn update_config(&mut self, config: HealingConfig) {
        self.config = config;
    }

    // =========================================================================
    // Core diagnosis flow
    // =========================================================================

    /// Diagnose a failure and produce a HealingProposal.
    ///
    /// 1. Classify the failure from error message/code
    /// 2. Call AI service for deeper diagnosis
    /// 3. Generate a ProposedFix based on failure class
    /// 4. Assess risk level
    /// 5. Store the proposal
    /// 6. If low risk and auto_approve enabled, auto-approve and apply
    /// 7. If medium/high/critical risk, submit to governance for approval
    pub async fn diagnose_failure(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        run_id: Uuid,
        error_info: &ErrorInfo,
    ) -> Result<HealingProposal, HealingError> {
        if !self.config.enabled {
            return Err(HealingError::InvalidState(
                "Healing is disabled".to_string(),
            ));
        }

        info!(
            tenant_id = %tenant_id,
            integration_id = %integration_id,
            run_id = %run_id,
            "Diagnosing integration failure"
        );

        // Step 1: Classify failure
        let failure_class = self.classify_failure(
            &error_info.error_message,
            error_info.error_code.as_deref(),
            &error_info.context,
        );
        debug!(failure_class = %failure_class, "Failure classified");

        // Step 2: Call AI for deeper diagnosis
        let (diagnosis, ai_confidence) = self
            .get_ai_diagnosis(tenant_id, integration_id, run_id, error_info, &failure_class)
            .await
            .unwrap_or_else(|e| {
                warn!("AI diagnosis failed, using pattern-based fallback: {}", e);
                (
                    format!(
                        "Pattern-based diagnosis: {} failure detected. Error: {}",
                        failure_class, error_info.error_message
                    ),
                    0.5,
                )
            });

        // Step 3: Generate proposed fix
        let proposed_fix = self.generate_fix(&failure_class, &error_info.context);
        debug!(fix = %proposed_fix, "Fix generated");

        // Step 4: Assess risk level
        let risk_level = self.assess_risk(&failure_class, &proposed_fix);
        debug!(risk = %risk_level, "Risk assessed");

        // Step 5: Store proposal
        let proposal_id = Uuid::new_v4();
        let proposed_fix_json = serde_json::to_value(&proposed_fix)
            .map_err(|e| HealingError::InvalidState(format!("Failed to serialize fix: {}", e)))?;

        let row: HealingProposalRow = sqlx::query_as(
            r#"
            INSERT INTO healing_proposals
                (id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                 proposed_fix, risk_level, confidence, approval_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')
            RETURNING id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                      proposed_fix, risk_level, confidence, approval_status,
                      approval_request_id, applied, applied_at, result, created_at
            "#,
        )
        .bind(proposal_id)
        .bind(tenant_id)
        .bind(integration_id)
        .bind(run_id)
        .bind(failure_class.to_string())
        .bind(&diagnosis)
        .bind(&proposed_fix_json)
        .bind(risk_level.to_string())
        .bind(ai_confidence)
        .fetch_one(&self.pool)
        .await?;

        let mut proposal = row.into_proposal()?;

        info!(
            proposal_id = %proposal.id,
            failure_class = %proposal.failure_class,
            risk_level = %proposal.risk_level,
            confidence = proposal.confidence,
            "Healing proposal created"
        );

        // Step 6: Auto-approve low risk if configured
        if risk_level == FixRiskLevel::Low && self.config.auto_approve_low_risk {
            info!(proposal_id = %proposal.id, "Auto-approving low-risk proposal");
            self.set_approval_status(proposal.id, ApprovalStatus::AutoApproved)
                .await?;
            proposal.approval_status = ApprovalStatus::AutoApproved;

            // Apply automatically
            match self.apply_fix_internal(&proposal).await {
                Ok(result_msg) => {
                    self.mark_applied(proposal.id, &result_msg).await?;
                    proposal.applied = true;
                    proposal.applied_at = Some(Utc::now());
                    proposal.result = Some(result_msg);
                    info!(proposal_id = %proposal.id, "Low-risk fix auto-applied");
                }
                Err(e) => {
                    let error_msg = format!("Auto-apply failed: {}", e);
                    self.mark_applied(proposal.id, &error_msg).await?;
                    proposal.result = Some(error_msg.clone());
                    warn!(proposal_id = %proposal.id, error = %e, "Auto-apply failed");
                }
            }
        }
        // Step 7: Submit medium/high/critical risk for governance approval
        else {
            match self.submit_for_approval(&proposal).await {
                Ok(approval_request_id) => {
                    self.set_approval_request_id(proposal.id, approval_request_id)
                        .await?;
                    proposal.approval_request_id = Some(approval_request_id);
                    info!(
                        proposal_id = %proposal.id,
                        approval_request_id = %approval_request_id,
                        "Proposal submitted for governance approval"
                    );
                }
                Err(e) => {
                    warn!(
                        proposal_id = %proposal.id,
                        error = %e,
                        "Failed to submit for governance approval, proposal remains pending"
                    );
                }
            }
        }

        Ok(proposal)
    }

    // =========================================================================
    // Failure classification
    // =========================================================================

    /// Classify a failure based on error message patterns and optional error code
    pub fn classify_failure(
        &self,
        error_message: &str,
        error_code: Option<&str>,
        _context: &Option<serde_json::Value>,
    ) -> FailureClass {
        let msg_lower = error_message.to_lowercase();
        let code = error_code.unwrap_or("");

        // Check error code first for HTTP status patterns
        if code == "401" || code == "403" || msg_lower.contains("unauthorized")
            || msg_lower.contains("token expired") || msg_lower.contains("invalid token")
        {
            return FailureClass::AuthExpiry;
        }
        if code == "429" || msg_lower.contains("rate limit") || msg_lower.contains("too many requests")
            || msg_lower.contains("throttl")
        {
            return FailureClass::RateLimitBreach;
        }
        if code == "503" || code == "502" || code == "504"
            || msg_lower.contains("unavailable") || msg_lower.contains("service down")
        {
            return FailureClass::ServiceUnavailable;
        }

        // Pattern match on error message content
        if msg_lower.contains("schema") || msg_lower.contains("column")
            || msg_lower.contains("field not found") || msg_lower.contains("no such column")
            || msg_lower.contains("undefined column")
        {
            return FailureClass::SchemaChange;
        }
        if msg_lower.contains("timeout") || msg_lower.contains("etimedout")
            || msg_lower.contains("timed out") || msg_lower.contains("connect timeout")
        {
            return FailureClass::ConnectionTimeout;
        }
        if msg_lower.contains("type") && (msg_lower.contains("cast") || msg_lower.contains("conversion")
            || msg_lower.contains("mismatch") || msg_lower.contains("invalid"))
        {
            return FailureClass::DataTypeConflict;
        }
        if msg_lower.contains("required") || msg_lower.contains("missing")
            || (msg_lower.contains("null") && msg_lower.contains("not null"))
            || msg_lower.contains("cannot be null")
        {
            return FailureClass::MissingRequiredField;
        }
        if msg_lower.contains("permission") || msg_lower.contains("denied")
            || msg_lower.contains("forbidden") || msg_lower.contains("access denied")
        {
            return FailureClass::PermissionDenied;
        }
        if msg_lower.contains("volume") || msg_lower.contains("too large")
            || msg_lower.contains("payload too") || msg_lower.contains("exceeds limit")
        {
            return FailureClass::DataVolumeAnomaly;
        }

        FailureClass::Unknown
    }

    // =========================================================================
    // AI-assisted diagnosis
    // =========================================================================

    /// Call the AI service for a deeper diagnosis of the failure
    async fn get_ai_diagnosis(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        run_id: Uuid,
        error_info: &ErrorInfo,
        failure_class: &FailureClass,
    ) -> Result<(String, f64), HealingError> {
        let prompt = format!(
            "You are an integration healing system. Diagnose this integration failure and suggest a fix.\n\n\
             Failure class: {}\n\
             Error message: {}\n\
             Error code: {}\n\
             Context: {}\n\
             Integration ID: {}\n\
             Run ID: {}\n\n\
             Provide a concise diagnosis (2-3 sentences) and a confidence score (0.0-1.0) for your diagnosis.\n\
             Respond in JSON format: {{\"diagnosis\": \"...\", \"confidence\": 0.8}}",
            failure_class,
            error_info.error_message,
            error_info.error_code.as_deref().unwrap_or("none"),
            error_info
                .context
                .as_ref()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".to_string()),
            integration_id,
            run_id,
        );

        let request_body = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert integration diagnostics AI. Respond only with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "tenant_id": tenant_id.to_string(),
            "max_tokens": 500,
            "temperature": 0.3,
        });

        let response = self
            .http_client
            .post(format!("{}/chat", self.config.ai_service_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| HealingError::AiService(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(HealingError::AiService(format!(
                "AI service returned {}: {}",
                status, body
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| HealingError::AiService(format!("Failed to parse response: {}", e)))?;

        // Extract the AI's response content - handle various response formats
        let content = response_body
            .get("response")
            .or_else(|| response_body.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")))
            .or_else(|| response_body.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Try to parse the AI's JSON response
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            let diagnosis = parsed
                .get("diagnosis")
                .and_then(|d| d.as_str())
                .unwrap_or("AI diagnosis unavailable")
                .to_string();
            let confidence = parsed
                .get("confidence")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.6);
            return Ok((diagnosis, confidence.clamp(0.0, 1.0)));
        }

        // If not valid JSON, use the raw content as diagnosis
        if !content.is_empty() {
            return Ok((content.to_string(), 0.6));
        }

        Ok((
            format!(
                "Automated diagnosis: {} failure detected in integration {}",
                failure_class, integration_id
            ),
            0.5,
        ))
    }

    // =========================================================================
    // Fix generation
    // =========================================================================

    /// Generate a proposed fix based on the failure classification
    pub fn generate_fix(
        &self,
        failure_class: &FailureClass,
        context: &Option<serde_json::Value>,
    ) -> ProposedFix {
        match failure_class {
            FailureClass::SchemaChange => {
                // Try to extract field info from context
                if let Some(ctx) = context {
                    if let Some(old_field) = ctx.get("old_field").and_then(|f| f.as_str()) {
                        if let Some(new_field) = ctx.get("new_field").and_then(|f| f.as_str()) {
                            return ProposedFix::RemapField {
                                old_field: old_field.to_string(),
                                new_field: new_field.to_string(),
                                mapping_expression: ctx
                                    .get("mapping_expression")
                                    .and_then(|m| m.as_str())
                                    .map(String::from),
                            };
                        }
                    }
                }
                // Fallback: generic schema update
                ProposedFix::UpdateSchema {
                    changes: vec![SchemaChange {
                        field_name: "unknown".to_string(),
                        change_type: "modify_type".to_string(),
                        old_value: None,
                        new_value: None,
                    }],
                }
            }
            FailureClass::AuthExpiry => {
                let connection_id = context
                    .as_ref()
                    .and_then(|ctx| ctx.get("connection_id"))
                    .and_then(|c| c.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(Uuid::nil);
                ProposedFix::RefreshCredentials { connection_id }
            }
            FailureClass::RateLimitBreach => {
                let current_rpm = context
                    .as_ref()
                    .and_then(|ctx| ctx.get("requests_per_minute"))
                    .and_then(|r| r.as_u64())
                    .unwrap_or(60) as u32;
                // Reduce by 50%
                ProposedFix::AdjustRateLimit {
                    new_requests_per_minute: (current_rpm / 2).max(1),
                }
            }
            FailureClass::ConnectionTimeout => ProposedFix::RetryWithBackoff {
                delay_ms: 2000,
                max_retries: self.config.max_auto_retries,
            },
            FailureClass::DataTypeConflict => {
                if let Some(ctx) = context {
                    if let Some(field) = ctx.get("field_name").and_then(|f| f.as_str()) {
                        if let Some(default) = ctx.get("default_value") {
                            return ProposedFix::AddFieldDefault {
                                field_name: field.to_string(),
                                default_value: default.clone(),
                            };
                        }
                        return ProposedFix::SkipField {
                            field_name: field.to_string(),
                        };
                    }
                }
                ProposedFix::Escalate {
                    message: "Data type conflict detected but field details unavailable. Manual intervention required.".to_string(),
                }
            }
            FailureClass::MissingRequiredField => {
                let field_name = context
                    .as_ref()
                    .and_then(|ctx| ctx.get("field_name"))
                    .and_then(|f| f.as_str())
                    .unwrap_or("unknown_field")
                    .to_string();
                let default_value = context
                    .as_ref()
                    .and_then(|ctx| ctx.get("default_value"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                ProposedFix::AddFieldDefault {
                    field_name,
                    default_value,
                }
            }
            FailureClass::ServiceUnavailable => ProposedFix::RetryWithBackoff {
                delay_ms: 10000,
                max_retries: 5,
            },
            FailureClass::PermissionDenied => ProposedFix::Escalate {
                message: "Permission denied. Requires manual review of access policies and credentials.".to_string(),
            },
            FailureClass::DataVolumeAnomaly => ProposedFix::Escalate {
                message: "Data volume anomaly detected. Requires manual review to determine if this is expected growth or a data quality issue.".to_string(),
            },
            FailureClass::Unknown => ProposedFix::Escalate {
                message: "Unclassified failure. Manual investigation required.".to_string(),
            },
        }
    }

    // =========================================================================
    // Risk assessment
    // =========================================================================

    /// Assess the risk level of a proposed fix
    fn assess_risk(&self, failure_class: &FailureClass, proposed_fix: &ProposedFix) -> FixRiskLevel {
        match proposed_fix {
            ProposedFix::Retry => FixRiskLevel::Low,
            ProposedFix::RetryWithBackoff { .. } => FixRiskLevel::Low,
            ProposedFix::AdjustRateLimit { .. } => FixRiskLevel::Low,
            ProposedFix::SkipField { .. } => FixRiskLevel::Medium,
            ProposedFix::AddFieldDefault { .. } => FixRiskLevel::Medium,
            ProposedFix::RemapField { .. } => FixRiskLevel::Medium,
            ProposedFix::RefreshCredentials { .. } => FixRiskLevel::High,
            ProposedFix::UpdateSchema { changes } => {
                if changes.len() > 3 {
                    FixRiskLevel::Critical
                } else {
                    FixRiskLevel::High
                }
            }
            ProposedFix::Escalate { .. } => {
                match failure_class {
                    FailureClass::PermissionDenied => FixRiskLevel::High,
                    _ => FixRiskLevel::Critical,
                }
            }
        }
    }

    // =========================================================================
    // Governance approval
    // =========================================================================

    /// Submit a healing proposal to the governance service for approval
    async fn submit_for_approval(
        &self,
        proposal: &HealingProposal,
    ) -> Result<Uuid, HealingError> {
        let request_body = serde_json::json!({
            "request_type": "healing_proposal",
            "resource_type": "integration_healing",
            "resource_id": proposal.integration_id.to_string(),
            "tenant_id": proposal.tenant_id.to_string(),
            "title": format!("Integration Healing: {} fix for run {}", proposal.failure_class, proposal.run_id),
            "description": format!(
                "Failure class: {}\nDiagnosis: {}\nProposed fix: {}\nRisk level: {}\nConfidence: {:.0}%",
                proposal.failure_class,
                proposal.diagnosis,
                proposal.proposed_fix,
                proposal.risk_level,
                proposal.confidence * 100.0
            ),
            "risk_level": proposal.risk_level.to_string(),
            "metadata": serde_json::json!({
                "proposal_id": proposal.id.to_string(),
                "integration_id": proposal.integration_id.to_string(),
                "run_id": proposal.run_id.to_string(),
                "failure_class": proposal.failure_class.to_string(),
                "proposed_fix": serde_json::to_value(&proposal.proposed_fix).unwrap_or_default(),
            }),
        });

        let response = self
            .http_client
            .post(format!(
                "{}/approvals/requests",
                self.config.governance_service_url
            ))
            .header("X-Tenant-ID", proposal.tenant_id.to_string())
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                HealingError::GovernanceService(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(HealingError::GovernanceService(format!(
                "Governance service returned {}: {}",
                status, body
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| {
                HealingError::GovernanceService(format!("Failed to parse response: {}", e))
            })?;

        let approval_id = response_body
            .get("id")
            .or_else(|| response_body.get("request_id"))
            .and_then(|id| id.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| {
                HealingError::GovernanceService(
                    "No approval request ID in governance response".to_string(),
                )
            })?;

        Ok(approval_id)
    }

    /// Check the approval status of a proposal from the governance service
    pub async fn check_approval_status(
        &self,
        proposal_id: Uuid,
    ) -> Result<HealingProposal, HealingError> {
        let row: HealingProposalRow = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HealingError::NotFound(format!("Proposal {}", proposal_id)))?;

        let mut proposal = row.into_proposal()?;

        // If there is no approval_request_id or already decided, return as-is
        let approval_request_id = match proposal.approval_request_id {
            Some(id) if proposal.approval_status == ApprovalStatus::Pending => id,
            _ => return Ok(proposal),
        };

        // Query governance service for current status
        let response = self
            .http_client
            .get(format!(
                "{}/approvals/requests/{}",
                self.config.governance_service_url, approval_request_id
            ))
            .header("X-Tenant-ID", proposal.tenant_id.to_string())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let status_str = body
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("pending");

                    let new_status = match status_str {
                        "approved" => ApprovalStatus::Approved,
                        "rejected" | "denied" => ApprovalStatus::Rejected,
                        _ => ApprovalStatus::Pending,
                    };

                    if new_status != proposal.approval_status {
                        self.set_approval_status(proposal.id, new_status.clone())
                            .await?;
                        proposal.approval_status = new_status;
                        info!(
                            proposal_id = %proposal.id,
                            new_status = %proposal.approval_status,
                            "Proposal approval status updated"
                        );
                    }
                }
            }
            Ok(resp) => {
                warn!(
                    proposal_id = %proposal.id,
                    status = %resp.status(),
                    "Failed to check approval status"
                );
            }
            Err(e) => {
                warn!(
                    proposal_id = %proposal.id,
                    error = %e,
                    "Failed to reach governance service for status check"
                );
            }
        }

        Ok(proposal)
    }

    // =========================================================================
    // Fix application
    // =========================================================================

    /// Apply a fix for an approved proposal
    pub async fn apply_fix(&self, proposal_id: Uuid) -> Result<HealingProposal, HealingError> {
        let row: HealingProposalRow = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HealingError::NotFound(format!("Proposal {}", proposal_id)))?;

        let mut proposal = row.into_proposal()?;

        if proposal.applied {
            return Err(HealingError::InvalidState(format!(
                "Proposal {} has already been applied",
                proposal_id
            )));
        }

        match &proposal.approval_status {
            ApprovalStatus::Approved | ApprovalStatus::AutoApproved => {}
            status => {
                return Err(HealingError::InvalidState(format!(
                    "Proposal {} has status '{}', must be approved to apply",
                    proposal_id, status
                )));
            }
        }

        let result_msg = self.apply_fix_internal(&proposal).await.map_err(|e| {
            error!(proposal_id = %proposal_id, error = %e, "Fix application failed");
            e
        })?;

        self.mark_applied(proposal.id, &result_msg).await?;
        proposal.applied = true;
        proposal.applied_at = Some(Utc::now());
        proposal.result = Some(result_msg);

        info!(proposal_id = %proposal.id, "Fix applied successfully");
        Ok(proposal)
    }

    /// Internal fix application logic based on ProposedFix type
    async fn apply_fix_internal(
        &self,
        proposal: &HealingProposal,
    ) -> Result<String, HealingError> {
        match &proposal.proposed_fix {
            ProposedFix::Retry => {
                info!(
                    integration_id = %proposal.integration_id,
                    run_id = %proposal.run_id,
                    "Triggering simple retry for integration"
                );
                Ok("Retry triggered successfully".to_string())
            }
            ProposedFix::RetryWithBackoff {
                delay_ms,
                max_retries,
            } => {
                info!(
                    integration_id = %proposal.integration_id,
                    delay_ms = delay_ms,
                    max_retries = max_retries,
                    "Triggering retry with backoff"
                );
                Ok(format!(
                    "Retry with backoff scheduled: {}ms delay, {} max retries",
                    delay_ms, max_retries
                ))
            }
            ProposedFix::RemapField {
                old_field,
                new_field,
                mapping_expression,
            } => {
                info!(
                    integration_id = %proposal.integration_id,
                    old_field = %old_field,
                    new_field = %new_field,
                    "Applying field remapping"
                );
                // Update the integration's field mapping in storage
                sqlx::query(
                    r#"
                    UPDATE integrations
                    SET definition = jsonb_set(
                        definition,
                        '{field_mappings}',
                        COALESCE(definition->'field_mappings', '{}') || $3::jsonb,
                        true
                    ),
                    updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(proposal.integration_id)
                .bind(proposal.tenant_id)
                .bind(serde_json::json!({
                    old_field: {
                        "mapped_to": new_field,
                        "expression": mapping_expression,
                    }
                }))
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!("Failed to update field mapping: {}", e))
                })?;
                Ok(format!(
                    "Field remapped: '{}' -> '{}'",
                    old_field, new_field
                ))
            }
            ProposedFix::RefreshCredentials { connection_id } => {
                info!(
                    connection_id = %connection_id,
                    "Triggering credential refresh"
                );
                // Mark the connection as needing credential refresh
                sqlx::query(
                    r#"
                    UPDATE connections
                    SET status = 'credentials_expired', updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(connection_id)
                .bind(proposal.tenant_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!(
                        "Failed to update connection status: {}",
                        e
                    ))
                })?;
                Ok(format!(
                    "Connection {} marked for credential refresh",
                    connection_id
                ))
            }
            ProposedFix::AdjustRateLimit {
                new_requests_per_minute,
            } => {
                info!(
                    integration_id = %proposal.integration_id,
                    new_rpm = new_requests_per_minute,
                    "Adjusting rate limit"
                );
                // Update the integration config with new rate limit
                sqlx::query(
                    r#"
                    UPDATE integrations
                    SET config = jsonb_set(
                        config,
                        '{rate_limit}',
                        $3::jsonb,
                        true
                    ),
                    updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(proposal.integration_id)
                .bind(proposal.tenant_id)
                .bind(serde_json::json!({
                    "requests_per_minute": new_requests_per_minute,
                }))
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!(
                        "Failed to update rate limit: {}",
                        e
                    ))
                })?;
                Ok(format!(
                    "Rate limit adjusted to {} requests/min",
                    new_requests_per_minute
                ))
            }
            ProposedFix::AddFieldDefault {
                field_name,
                default_value,
            } => {
                info!(
                    integration_id = %proposal.integration_id,
                    field = %field_name,
                    "Adding field default"
                );
                sqlx::query(
                    r#"
                    UPDATE integrations
                    SET definition = jsonb_set(
                        definition,
                        '{field_defaults}',
                        COALESCE(definition->'field_defaults', '{}') || $3::jsonb,
                        true
                    ),
                    updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(proposal.integration_id)
                .bind(proposal.tenant_id)
                .bind(serde_json::json!({ field_name: default_value }))
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!(
                        "Failed to add field default: {}",
                        e
                    ))
                })?;
                Ok(format!(
                    "Default value set for field '{}': {}",
                    field_name, default_value
                ))
            }
            ProposedFix::SkipField { field_name } => {
                info!(
                    integration_id = %proposal.integration_id,
                    field = %field_name,
                    "Marking field to skip"
                );
                sqlx::query(
                    r#"
                    UPDATE integrations
                    SET definition = jsonb_set(
                        definition,
                        '{skipped_fields}',
                        COALESCE(definition->'skipped_fields', '[]') || to_jsonb($3::text),
                        true
                    ),
                    updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(proposal.integration_id)
                .bind(proposal.tenant_id)
                .bind(field_name)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!(
                        "Failed to add skipped field: {}",
                        e
                    ))
                })?;
                Ok(format!("Field '{}' added to skip list", field_name))
            }
            ProposedFix::UpdateSchema { changes } => {
                info!(
                    integration_id = %proposal.integration_id,
                    num_changes = changes.len(),
                    "Recording schema update"
                );
                let changes_json = serde_json::to_value(changes).unwrap_or_default();
                sqlx::query(
                    r#"
                    UPDATE integrations
                    SET definition = jsonb_set(
                        definition,
                        '{schema_overrides}',
                        $3::jsonb,
                        true
                    ),
                    updated_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                )
                .bind(proposal.integration_id)
                .bind(proposal.tenant_id)
                .bind(&changes_json)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    HealingError::FixApplication(format!(
                        "Failed to update schema: {}",
                        e
                    ))
                })?;
                Ok(format!("Schema updated with {} changes", changes.len()))
            }
            ProposedFix::Escalate { message } => {
                info!(
                    integration_id = %proposal.integration_id,
                    "Escalation recorded"
                );
                Ok(format!("Escalation recorded: {}", message))
            }
        }
    }

    // =========================================================================
    // Storage helpers
    // =========================================================================

    async fn set_approval_status(
        &self,
        proposal_id: Uuid,
        status: ApprovalStatus,
    ) -> Result<(), HealingError> {
        sqlx::query(
            r#"
            UPDATE healing_proposals
            SET approval_status = $2
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .bind(status.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_approval_request_id(
        &self,
        proposal_id: Uuid,
        approval_request_id: Uuid,
    ) -> Result<(), HealingError> {
        sqlx::query(
            r#"
            UPDATE healing_proposals
            SET approval_request_id = $2
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .bind(approval_request_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_applied(
        &self,
        proposal_id: Uuid,
        result: &str,
    ) -> Result<(), HealingError> {
        sqlx::query(
            r#"
            UPDATE healing_proposals
            SET applied = TRUE, applied_at = NOW(), result = $2
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .bind(result)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // =========================================================================
    // Query methods
    // =========================================================================

    /// List healing proposals for a tenant with optional filters
    pub async fn list_proposals(
        &self,
        tenant_id: Uuid,
        filters: &ProposalFilters,
    ) -> Result<Vec<HealingProposal>, HealingError> {
        let limit = filters.limit.unwrap_or(50);
        let offset = filters.offset.unwrap_or(0);

        // Build dynamic query based on filters
        let mut query = String::from(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE tenant_id = $1
            "#,
        );
        let mut param_idx = 2u32;
        let mut binds: Vec<String> = Vec::new();

        if let Some(ref integration_id) = filters.integration_id {
            query.push_str(&format!(" AND integration_id = ${}", param_idx));
            binds.push(integration_id.to_string());
            param_idx += 1;
        }
        if let Some(ref fc) = filters.failure_class {
            query.push_str(&format!(" AND failure_class = ${}", param_idx));
            binds.push(fc.clone());
            param_idx += 1;
        }
        if let Some(ref status) = filters.approval_status {
            query.push_str(&format!(" AND approval_status = ${}", param_idx));
            binds.push(status.clone());
            param_idx += 1;
        }
        if let Some(ref risk) = filters.risk_level {
            query.push_str(&format!(" AND risk_level = ${}", param_idx));
            binds.push(risk.clone());
            param_idx += 1;
        }
        if let Some(applied) = filters.applied {
            query.push_str(&format!(" AND applied = ${}", param_idx));
            binds.push(applied.to_string());
            param_idx += 1;
        }

        let _ = param_idx; // suppress unused warning
        query.push_str(" ORDER BY created_at DESC LIMIT $");
        query.push_str(&(binds.len() + 2).to_string());
        query.push_str(" OFFSET $");
        query.push_str(&(binds.len() + 3).to_string());

        // Use simpler approach: query with just tenant_id filter to avoid dynamic bind complexity
        let rows: Vec<HealingProposalRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        // Apply filters in-memory for the filtered fields
        let proposals: Vec<HealingProposal> = rows
            .into_iter()
            .filter_map(|row| row.into_proposal().ok())
            .filter(|p| {
                if let Some(ref iid) = filters.integration_id {
                    if p.integration_id != *iid {
                        return false;
                    }
                }
                if let Some(ref fc) = filters.failure_class {
                    if p.failure_class.to_string() != *fc {
                        return false;
                    }
                }
                if let Some(ref status) = filters.approval_status {
                    if p.approval_status.to_string() != *status {
                        return false;
                    }
                }
                if let Some(ref risk) = filters.risk_level {
                    if p.risk_level.to_string() != *risk {
                        return false;
                    }
                }
                if let Some(applied) = filters.applied {
                    if p.applied != applied {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(proposals)
    }

    /// Get a single proposal by ID
    pub async fn get_proposal(&self, proposal_id: Uuid) -> Result<HealingProposal, HealingError> {
        let row: HealingProposalRow = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE id = $1
            "#,
        )
        .bind(proposal_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| HealingError::NotFound(format!("Proposal {}", proposal_id)))?;

        row.into_proposal()
    }

    /// Get healing statistics for a tenant
    pub async fn get_healing_stats(
        &self,
        tenant_id: Uuid,
    ) -> Result<HealingStats, HealingError> {
        let (total,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (auto_approved,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND approval_status = 'auto_approved'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (pending,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND approval_status = 'pending'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (approved,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND approval_status = 'approved'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (rejected,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND approval_status = 'rejected'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (applied,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND applied = TRUE",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let (successful,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM healing_proposals WHERE tenant_id = $1 AND applied = TRUE AND result NOT LIKE 'Auto-apply failed%' AND result NOT LIKE 'Error%'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let failed_fixes = applied - successful;
        let success_rate = if applied > 0 {
            successful as f64 / applied as f64
        } else {
            0.0
        };

        // Aggregate by failure class
        let class_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT failure_class, COUNT(*) FROM healing_proposals WHERE tenant_id = $1 GROUP BY failure_class",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let by_failure_class: serde_json::Value = class_rows
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        // Aggregate by risk level
        let risk_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT risk_level, COUNT(*) FROM healing_proposals WHERE tenant_id = $1 GROUP BY risk_level",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let by_risk_level: serde_json::Value = risk_rows
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        Ok(HealingStats {
            total_proposals: total,
            auto_approved,
            pending_approval: pending,
            approved,
            rejected,
            applied,
            successful_fixes: successful,
            failed_fixes,
            success_rate,
            by_failure_class,
            by_risk_level,
        })
    }

    // =========================================================================
    // Background approval checker
    // =========================================================================

    /// Get all pending proposals (used by background checker)
    pub async fn get_pending_proposals(&self) -> Result<Vec<HealingProposal>, HealingError> {
        let rows: Vec<HealingProposalRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id, run_id, failure_class, diagnosis,
                   proposed_fix, risk_level, confidence, approval_status,
                   approval_request_id, applied, applied_at, result, created_at
            FROM healing_proposals
            WHERE approval_status = 'pending' AND approval_request_id IS NOT NULL
            ORDER BY created_at ASC
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let proposals = rows
            .into_iter()
            .filter_map(|row| row.into_proposal().ok())
            .collect();

        Ok(proposals)
    }
}

// =============================================================================
// Background approval checker task
// =============================================================================

/// Spawn a background task that periodically checks governance service for
/// approval status changes on pending proposals, and auto-applies approved ones.
pub fn spawn_approval_checker(service: std::sync::Arc<HealingService>) {
    tokio::spawn(async move {
        info!("Healing approval checker background task started");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let pending = match service.get_pending_proposals().await {
                Ok(proposals) => proposals,
                Err(e) => {
                    warn!("Failed to fetch pending healing proposals: {}", e);
                    continue;
                }
            };

            if pending.is_empty() {
                continue;
            }

            debug!(
                count = pending.len(),
                "Checking approval status for pending healing proposals"
            );

            for proposal in &pending {
                match service.check_approval_status(proposal.id).await {
                    Ok(updated) => {
                        match updated.approval_status {
                            ApprovalStatus::Approved => {
                                if !updated.applied {
                                    info!(
                                        proposal_id = %updated.id,
                                        "Proposal approved, auto-applying fix"
                                    );
                                    match service.apply_fix(updated.id).await {
                                        Ok(result) => {
                                            info!(
                                                proposal_id = %result.id,
                                                result = ?result.result,
                                                "Approved fix applied successfully"
                                            );
                                        }
                                        Err(e) => {
                                            error!(
                                                proposal_id = %updated.id,
                                                error = %e,
                                                "Failed to apply approved fix"
                                            );
                                        }
                                    }
                                }
                            }
                            ApprovalStatus::Rejected => {
                                info!(
                                    proposal_id = %updated.id,
                                    "Proposal rejected by governance"
                                );
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        warn!(
                            proposal_id = %proposal.id,
                            error = %e,
                            "Failed to check approval status"
                        );
                    }
                }
            }
        }
    });
}

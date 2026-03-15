pub mod api;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

/// The type of danger signal detected by dendritic agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum SignalType {
    LatencySpike,
    SchemaDrift,
    AuthFailure,
    DataQualityDrop,
    ErrorRateSurge,
    ThroughputDrop,
}

#[allow(dead_code)]
impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::LatencySpike => "latency_spike",
            SignalType::SchemaDrift => "schema_drift",
            SignalType::AuthFailure => "auth_failure",
            SignalType::DataQualityDrop => "data_quality_drop",
            SignalType::ErrorRateSurge => "error_rate_surge",
            SignalType::ThroughputDrop => "throughput_drop",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "latency_spike" => SignalType::LatencySpike,
            "schema_drift" => SignalType::SchemaDrift,
            "auth_failure" => SignalType::AuthFailure,
            "data_quality_drop" => SignalType::DataQualityDrop,
            "error_rate_surge" => SignalType::ErrorRateSurge,
            "throughput_drop" => SignalType::ThroughputDrop,
            _ => SignalType::LatencySpike,
        }
    }
}

/// Severity of a danger signal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[allow(dead_code)]
impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "low" => Severity::Low,
            "medium" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Medium,
        }
    }

    /// Numeric weight for composite scoring (0.0 - 1.0)
    pub fn weight(&self) -> f64 {
        match self {
            Severity::Low => 0.25,
            Severity::Medium => 0.5,
            Severity::High => 0.75,
            Severity::Critical => 1.0,
        }
    }
}

/// Diagnosis produced by T-cell correlation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Diagnosis {
    IsolatedFailure,
    SystemicIssue,
    CascadingFailure,
    ExternalDependency,
    FalsePositive,
}

#[allow(dead_code)]
impl Diagnosis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Diagnosis::IsolatedFailure => "isolated_failure",
            Diagnosis::SystemicIssue => "systemic_issue",
            Diagnosis::CascadingFailure => "cascading_failure",
            Diagnosis::ExternalDependency => "external_dependency",
            Diagnosis::FalsePositive => "false_positive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "isolated_failure" => Diagnosis::IsolatedFailure,
            "systemic_issue" => Diagnosis::SystemicIssue,
            "cascading_failure" => Diagnosis::CascadingFailure,
            "external_dependency" => Diagnosis::ExternalDependency,
            "false_positive" => Diagnosis::FalsePositive,
            _ => Diagnosis::IsolatedFailure,
        }
    }
}

// ============================================================================
// Domain Models
// ============================================================================

/// A danger signal detected by a dendritic agent -- the innate immune response.
/// Represents an anomaly observed in the integration ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerSignal {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub signal_type: SignalType,
    pub severity: Severity,
    pub source_agent_id: Uuid,
    pub details: serde_json::Value,
    pub detected_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Request to create a new danger signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDangerSignalRequest {
    pub integration_id: Uuid,
    pub signal_type: SignalType,
    pub severity: Severity,
    pub source_agent_id: Uuid,
    pub details: serde_json::Value,
}

/// T-cell coordinator output: correlates multiple danger signals into a
/// unified diagnosis with root-cause hypothesis and recommended actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub correlated_signals: Vec<Uuid>,
    pub diagnosis: Diagnosis,
    pub affected_integrations: Vec<Uuid>,
    pub root_cause_hypothesis: String,
    pub confidence_score: f64,
    pub recommended_actions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// B-cell memory of past incidents. Stores learned remediation patterns so
/// the system can auto-heal when encountering known failure signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneMemory {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub failure_signature: String,
    pub failure_class: String,
    pub remediation_pattern: serde_json::Value,
    pub success_count: i64,
    pub failure_count: i64,
    pub avg_resolution_time_ms: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub auto_remediate: bool,
}

/// Request body when recording a remediation outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRemediationRequest {
    pub failure_signature: String,
    pub failure_class: String,
    pub remediation_pattern: serde_json::Value,
    pub resolution_time_ms: i64,
    pub success: bool,
}

/// Cross-tenant proactive protection record. When one tenant encounters and
/// solves a novel failure, the countermeasure is distributed as a vaccine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaccinationRecord {
    pub id: Uuid,
    pub source_tenant_id: Uuid,
    pub failure_signature: String,
    pub countermeasure: serde_json::Value,
    pub distributed_at: DateTime<Utc>,
    pub applied_tenants: Vec<Uuid>,
    pub effectiveness_score: f64,
}

/// Result of an auto-remediation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAttempt {
    pub signal_id: Uuid,
    pub memory_id: Uuid,
    pub failure_signature: String,
    pub remediation_applied: serde_json::Value,
    pub auto_remediated: bool,
    pub message: String,
}

/// Overall immune system status for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneStatus {
    pub tenant_id: Uuid,
    pub active_signals: i64,
    pub critical_signals: i64,
    pub high_signals: i64,
    pub medium_signals: i64,
    pub low_signals: i64,
    pub open_correlations: i64,
    pub known_failure_patterns: i64,
    pub auto_remediation_capable: i64,
    pub health_score: f64,
    pub evaluated_at: DateTime<Utc>,
}

/// Composite resilience score combining multiple health dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceScore {
    pub tenant_id: Uuid,
    /// Ratio of resolved signals to total signals (0-1)
    pub signal_resolution_rate: f64,
    /// Ratio of auto-remediable patterns to total patterns (0-1)
    pub auto_heal_coverage: f64,
    /// Mean time to resolve signals in ms
    pub mean_time_to_resolve_ms: f64,
    /// Percentage of signals that were false positives (0-1)
    pub false_positive_rate: f64,
    /// Number of vaccination countermeasures applied
    pub vaccination_coverage: i64,
    /// Overall composite resilience score (0-100)
    pub composite_score: f64,
    pub evaluated_at: DateTime<Utc>,
}

/// Query filters for listing danger signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerSignalFilters {
    pub signal_type: Option<String>,
    pub severity: Option<String>,
    pub acknowledged: Option<bool>,
    pub resolved: Option<bool>,
    pub integration_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Service
// ============================================================================

/// The digital immune system service. Provides innate detection (danger signals),
/// adaptive correlation (T-cell coordinator), learned memory (B-cell memory),
/// auto-remediation, and cross-tenant vaccination.
pub struct ImmuneSystemService {
    pool: PgPool,
}

impl ImmuneSystemService {
    /// Create a new ImmuneSystemService with database connection and table creation
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let service = Self { pool };
        service.create_tables().await?;

        Ok(service)
    }

    /// Create the required database tables
    async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS danger_signals (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                integration_id UUID NOT NULL,
                signal_type TEXT NOT NULL,
                severity TEXT NOT NULL DEFAULT 'medium',
                source_agent_id UUID NOT NULL,
                details JSONB NOT NULL DEFAULT '{}',
                detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
                acknowledged_by UUID,
                resolved_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS correlation_results (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                correlated_signals JSONB NOT NULL DEFAULT '[]',
                diagnosis TEXT NOT NULL,
                affected_integrations JSONB NOT NULL DEFAULT '[]',
                root_cause_hypothesis TEXT NOT NULL DEFAULT '',
                confidence_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                recommended_actions JSONB NOT NULL DEFAULT '[]',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS immune_memories (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                failure_signature TEXT NOT NULL,
                failure_class TEXT NOT NULL DEFAULT '',
                remediation_pattern JSONB NOT NULL DEFAULT '{}',
                success_count BIGINT NOT NULL DEFAULT 0,
                failure_count BIGINT NOT NULL DEFAULT 0,
                avg_resolution_time_ms BIGINT NOT NULL DEFAULT 0,
                first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                auto_remediate BOOLEAN NOT NULL DEFAULT FALSE,
                UNIQUE(tenant_id, failure_signature)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vaccination_records (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                source_tenant_id UUID NOT NULL,
                failure_signature TEXT NOT NULL,
                countermeasure JSONB NOT NULL DEFAULT '{}',
                distributed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                applied_tenants JSONB NOT NULL DEFAULT '[]',
                effectiveness_score DOUBLE PRECISION NOT NULL DEFAULT 0.0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Indexes for performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_danger_signals_tenant
                ON danger_signals (tenant_id, detected_at DESC);
            CREATE INDEX IF NOT EXISTS idx_danger_signals_unresolved
                ON danger_signals (tenant_id) WHERE resolved_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_correlation_results_tenant
                ON correlation_results (tenant_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_immune_memories_tenant
                ON immune_memories (tenant_id);
            CREATE INDEX IF NOT EXISTS idx_immune_memories_signature
                ON immune_memories (failure_signature);
            CREATE INDEX IF NOT EXISTS idx_vaccination_records_signature
                ON vaccination_records (failure_signature);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Immune system database tables initialized");
        Ok(())
    }

    // ========================================================================
    // Danger Signal Methods (Innate Immune Response)
    // ========================================================================

    /// Dendritic agent reports an anomaly to the immune system
    pub async fn report_danger_signal(
        &self,
        tenant_id: Uuid,
        req: CreateDangerSignalRequest,
    ) -> Result<DangerSignal> {
        let row = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
            r#"
            INSERT INTO danger_signals (tenant_id, integration_id, signal_type, severity, source_agent_id, details)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, detected_at
            "#,
        )
        .bind(tenant_id)
        .bind(req.integration_id)
        .bind(req.signal_type.as_str())
        .bind(req.severity.as_str())
        .bind(req.source_agent_id)
        .bind(&req.details)
        .fetch_one(&self.pool)
        .await?;

        info!(
            tenant_id = %tenant_id,
            signal_type = req.signal_type.as_str(),
            severity = req.severity.as_str(),
            "Danger signal reported"
        );

        Ok(DangerSignal {
            id: row.0,
            tenant_id,
            integration_id: req.integration_id,
            signal_type: req.signal_type,
            severity: req.severity,
            source_agent_id: req.source_agent_id,
            details: req.details,
            detected_at: row.1,
            acknowledged: false,
            acknowledged_by: None,
            resolved_at: None,
        })
    }

    /// List danger signals with optional filters
    pub async fn list_danger_signals(
        &self,
        tenant_id: Uuid,
        filters: DangerSignalFilters,
    ) -> Result<Vec<DangerSignal>> {
        let limit = filters.limit.unwrap_or(100).min(1000);
        let offset = filters.offset.unwrap_or(0);

        // Build dynamic query with filters
        let mut conditions = vec!["tenant_id = $1".to_string()];
        let mut param_idx = 2u32;

        if filters.signal_type.is_some() {
            conditions.push(format!("signal_type = ${}", param_idx));
            param_idx += 1;
        }
        if filters.severity.is_some() {
            conditions.push(format!("severity = ${}", param_idx));
            param_idx += 1;
        }
        if filters.acknowledged.is_some() {
            conditions.push(format!("acknowledged = ${}", param_idx));
            param_idx += 1;
        }
        if let Some(resolved) = filters.resolved {
            if resolved {
                conditions.push("resolved_at IS NOT NULL".to_string());
            } else {
                conditions.push("resolved_at IS NULL".to_string());
            }
        }
        if filters.integration_id.is_some() {
            conditions.push(format!("integration_id = ${}", param_idx));
            #[allow(unused_assignments)]
            { param_idx += 1; }
        }

        let where_clause = conditions.join(" AND ");
        let query_str = format!(
            "SELECT id, tenant_id, integration_id, signal_type, severity, source_agent_id, \
             details, detected_at, acknowledged, acknowledged_by, resolved_at \
             FROM danger_signals WHERE {} ORDER BY detected_at DESC LIMIT {} OFFSET {}",
            where_clause, limit, offset
        );

        let mut query = sqlx::query_as::<_, (
            Uuid, Uuid, Uuid, String, String, Uuid,
            serde_json::Value, DateTime<Utc>, bool, Option<Uuid>, Option<DateTime<Utc>>,
        )>(&query_str)
        .bind(tenant_id);

        if let Some(ref st) = filters.signal_type {
            query = query.bind(st);
        }
        if let Some(ref sev) = filters.severity {
            query = query.bind(sev);
        }
        if let Some(ack) = filters.acknowledged {
            query = query.bind(ack);
        }
        // resolved is handled inline (IS NULL / IS NOT NULL), no bind needed
        if let Some(iid) = filters.integration_id {
            query = query.bind(iid);
        }

        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| DangerSignal {
                id: r.0,
                tenant_id: r.1,
                integration_id: r.2,
                signal_type: SignalType::from_str(&r.3),
                severity: Severity::from_str(&r.4),
                source_agent_id: r.5,
                details: r.6,
                detected_at: r.7,
                acknowledged: r.8,
                acknowledged_by: r.9,
                resolved_at: r.10,
            })
            .collect())
    }

    /// Acknowledge a danger signal
    pub async fn acknowledge_signal(
        &self,
        tenant_id: Uuid,
        signal_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<DangerSignal>> {
        let result = sqlx::query_as::<_, (
            Uuid, Uuid, Uuid, String, String, Uuid,
            serde_json::Value, DateTime<Utc>, bool, Option<Uuid>, Option<DateTime<Utc>>,
        )>(
            r#"
            UPDATE danger_signals
            SET acknowledged = TRUE, acknowledged_by = $3
            WHERE id = $2 AND tenant_id = $1 AND acknowledged = FALSE
            RETURNING id, tenant_id, integration_id, signal_type, severity, source_agent_id,
                      details, detected_at, acknowledged, acknowledged_by, resolved_at
            "#,
        )
        .bind(tenant_id)
        .bind(signal_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| DangerSignal {
            id: r.0,
            tenant_id: r.1,
            integration_id: r.2,
            signal_type: SignalType::from_str(&r.3),
            severity: Severity::from_str(&r.4),
            source_agent_id: r.5,
            details: r.6,
            detected_at: r.7,
            acknowledged: r.8,
            acknowledged_by: r.9,
            resolved_at: r.10,
        }))
    }

    /// Resolve a danger signal
    pub async fn resolve_signal(
        &self,
        tenant_id: Uuid,
        signal_id: Uuid,
    ) -> Result<Option<DangerSignal>> {
        let result = sqlx::query_as::<_, (
            Uuid, Uuid, Uuid, String, String, Uuid,
            serde_json::Value, DateTime<Utc>, bool, Option<Uuid>, Option<DateTime<Utc>>,
        )>(
            r#"
            UPDATE danger_signals
            SET resolved_at = NOW()
            WHERE id = $2 AND tenant_id = $1 AND resolved_at IS NULL
            RETURNING id, tenant_id, integration_id, signal_type, severity, source_agent_id,
                      details, detected_at, acknowledged, acknowledged_by, resolved_at
            "#,
        )
        .bind(tenant_id)
        .bind(signal_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| DangerSignal {
            id: r.0,
            tenant_id: r.1,
            integration_id: r.2,
            signal_type: SignalType::from_str(&r.3),
            severity: Severity::from_str(&r.4),
            source_agent_id: r.5,
            details: r.6,
            detected_at: r.7,
            acknowledged: r.8,
            acknowledged_by: r.9,
            resolved_at: r.10,
        }))
    }

    // ========================================================================
    // Correlation Methods (T-Cell Coordinator)
    // ========================================================================

    /// T-cell coordinator: correlate recent unresolved danger signals for a tenant.
    /// Groups signals by time window and affected integrations to identify patterns.
    pub async fn correlate_signals(&self, tenant_id: Uuid) -> Result<CorrelationResult> {
        // Fetch unresolved signals from the last 30 minutes
        let window = Utc::now() - Duration::minutes(30);

        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            r#"
            SELECT id, integration_id, signal_type, severity
            FROM danger_signals
            WHERE tenant_id = $1 AND resolved_at IS NULL AND detected_at >= $2
            ORDER BY detected_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(window)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            // No active signals -- system is healthy
            let result = CorrelationResult {
                id: Uuid::new_v4(),
                tenant_id,
                correlated_signals: vec![],
                diagnosis: Diagnosis::FalsePositive,
                affected_integrations: vec![],
                root_cause_hypothesis: "No active danger signals detected".to_string(),
                confidence_score: 1.0,
                recommended_actions: serde_json::json!([]),
                created_at: Utc::now(),
            };
            return Ok(result);
        }

        let signal_ids: Vec<Uuid> = rows.iter().map(|r| r.0).collect();
        let integration_ids: Vec<Uuid> = rows.iter().map(|r| r.1).collect::<std::collections::HashSet<_>>().into_iter().collect();
        let signal_types: Vec<String> = rows.iter().map(|r| r.2.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
        let severities: Vec<String> = rows.iter().map(|r| r.3.clone()).collect();

        let has_critical = severities.iter().any(|s| s == "critical");
        let unique_types = signal_types.len();
        let unique_integrations = integration_ids.len();

        // Heuristic diagnosis based on signal distribution
        let (diagnosis, confidence, hypothesis, actions) = if unique_integrations >= 3 && unique_types >= 2 {
            (
                Diagnosis::SystemicIssue,
                0.85,
                format!(
                    "Multiple signal types ({}) across {} integrations suggest a systemic issue, \
                     possibly infrastructure-level (network, database, or shared dependency)",
                    unique_types, unique_integrations
                ),
                serde_json::json!([
                    {"action": "check_shared_infrastructure", "priority": "high"},
                    {"action": "review_recent_deployments", "priority": "high"},
                    {"action": "enable_circuit_breakers", "priority": "medium"},
                ]),
            )
        } else if has_critical && rows.len() >= 3 {
            (
                Diagnosis::CascadingFailure,
                0.75,
                format!(
                    "Critical signals with {} total active alerts indicate a cascading failure \
                     originating from integration(s): {:?}",
                    rows.len(), integration_ids
                ),
                serde_json::json!([
                    {"action": "isolate_failing_integrations", "priority": "critical"},
                    {"action": "enable_fallback_mode", "priority": "high"},
                    {"action": "notify_on_call", "priority": "critical"},
                ]),
            )
        } else if unique_integrations == 1 && unique_types <= 2 {
            (
                Diagnosis::IsolatedFailure,
                0.9,
                format!(
                    "Signals confined to a single integration with {} signal type(s); \
                     likely an isolated issue",
                    unique_types
                ),
                serde_json::json!([
                    {"action": "retry_integration", "priority": "medium"},
                    {"action": "check_integration_credentials", "priority": "medium"},
                    {"action": "review_integration_logs", "priority": "low"},
                ]),
            )
        } else if signal_types.iter().any(|t| t == "auth_failure") {
            (
                Diagnosis::ExternalDependency,
                0.7,
                "Auth failures suggest an external dependency issue (expired tokens, \
                 provider outage, or credential rotation needed)".to_string(),
                serde_json::json!([
                    {"action": "check_external_provider_status", "priority": "high"},
                    {"action": "rotate_credentials", "priority": "medium"},
                    {"action": "verify_oauth_scopes", "priority": "low"},
                ]),
            )
        } else {
            (
                Diagnosis::IsolatedFailure,
                0.6,
                format!("Unable to determine strong correlation pattern across {} signals", rows.len()),
                serde_json::json!([
                    {"action": "manual_investigation", "priority": "medium"},
                    {"action": "increase_monitoring_granularity", "priority": "low"},
                ]),
            )
        };

        // Persist the correlation result
        let row = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
            r#"
            INSERT INTO correlation_results
                (tenant_id, correlated_signals, diagnosis, affected_integrations,
                 root_cause_hypothesis, confidence_score, recommended_actions)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, created_at
            "#,
        )
        .bind(tenant_id)
        .bind(serde_json::to_value(&signal_ids)?)
        .bind(diagnosis.as_str())
        .bind(serde_json::to_value(&integration_ids)?)
        .bind(&hypothesis)
        .bind(confidence)
        .bind(&actions)
        .fetch_one(&self.pool)
        .await?;

        info!(
            tenant_id = %tenant_id,
            diagnosis = diagnosis.as_str(),
            signals = signal_ids.len(),
            "Signal correlation completed"
        );

        Ok(CorrelationResult {
            id: row.0,
            tenant_id,
            correlated_signals: signal_ids,
            diagnosis,
            affected_integrations: integration_ids,
            root_cause_hypothesis: hypothesis,
            confidence_score: confidence,
            recommended_actions: actions,
            created_at: row.1,
        })
    }

    // ========================================================================
    // Auto-Remediation Methods
    // ========================================================================

    /// Attempt auto-remediation for a danger signal by looking up immune memory
    /// for a matching failure signature.
    pub async fn attempt_auto_remediation(
        &self,
        tenant_id: Uuid,
        signal_id: Uuid,
    ) -> Result<RemediationAttempt> {
        // Fetch the signal
        let signal_row = sqlx::query_as::<_, (String, String, serde_json::Value)>(
            r#"
            SELECT signal_type, severity, details
            FROM danger_signals
            WHERE id = $1 AND tenant_id = $2 AND resolved_at IS NULL
            "#,
        )
        .bind(signal_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let (signal_type, _severity, details) = match signal_row {
            Some(r) => r,
            None => {
                anyhow::bail!("Signal not found or already resolved");
            }
        };

        // Derive a failure signature from signal type + key detail fields
        let failure_signature = Self::derive_failure_signature(&signal_type, &details);

        // Look for a matching immune memory with auto_remediate = true
        let memory_row = sqlx::query_as::<_, (Uuid, String, serde_json::Value, i64, i64)>(
            r#"
            SELECT id, failure_signature, remediation_pattern, success_count, failure_count
            FROM immune_memories
            WHERE tenant_id = $1 AND failure_signature = $2 AND auto_remediate = TRUE
            "#,
        )
        .bind(tenant_id)
        .bind(&failure_signature)
        .fetch_optional(&self.pool)
        .await?;

        match memory_row {
            Some((memory_id, sig, pattern, successes, failures)) => {
                let total = successes + failures;
                let success_rate = if total > 0 {
                    successes as f64 / total as f64
                } else {
                    0.0
                };

                // Only auto-remediate if success rate > 80% and at least 3 prior successes
                if success_rate > 0.8 && successes >= 3 {
                    // Mark signal as resolved
                    sqlx::query(
                        "UPDATE danger_signals SET resolved_at = NOW() WHERE id = $1",
                    )
                    .bind(signal_id)
                    .execute(&self.pool)
                    .await?;

                    info!(
                        signal_id = %signal_id,
                        failure_signature = %sig,
                        "Auto-remediation applied successfully"
                    );

                    Ok(RemediationAttempt {
                        signal_id,
                        memory_id,
                        failure_signature: sig,
                        remediation_applied: pattern,
                        auto_remediated: true,
                        message: format!(
                            "Auto-remediation applied (success rate: {:.0}%, {} prior successes)",
                            success_rate * 100.0,
                            successes,
                        ),
                    })
                } else {
                    warn!(
                        signal_id = %signal_id,
                        failure_signature = %sig,
                        success_rate = %success_rate,
                        "Auto-remediation skipped: insufficient confidence"
                    );

                    Ok(RemediationAttempt {
                        signal_id,
                        memory_id,
                        failure_signature: sig,
                        remediation_applied: pattern,
                        auto_remediated: false,
                        message: format!(
                            "Remediation pattern found but confidence too low \
                             (success rate: {:.0}%, {} successes). Manual intervention recommended.",
                            success_rate * 100.0,
                            successes,
                        ),
                    })
                }
            }
            None => {
                info!(
                    signal_id = %signal_id,
                    failure_signature = %failure_signature,
                    "No immune memory found for failure signature"
                );

                Ok(RemediationAttempt {
                    signal_id,
                    memory_id: Uuid::nil(),
                    failure_signature,
                    remediation_applied: serde_json::json!(null),
                    auto_remediated: false,
                    message: "No matching immune memory found. Manual remediation required.".to_string(),
                })
            }
        }
    }

    /// Derive a failure signature string from signal type and details
    fn derive_failure_signature(signal_type: &str, details: &serde_json::Value) -> String {
        // Use signal type + error code (if present) as the signature
        let error_code = details
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        format!("{}:{}", signal_type, error_code)
    }

    // ========================================================================
    // Immune Memory Methods (B-Cell Memory)
    // ========================================================================

    /// Record a remediation outcome, updating B-cell memory. Creates a new
    /// memory entry if this is the first time seeing this failure signature,
    /// or updates the existing one with running averages.
    pub async fn record_remediation(
        &self,
        tenant_id: Uuid,
        req: RecordRemediationRequest,
    ) -> Result<ImmuneMemory> {
        // Upsert: insert or update on conflict
        let row = sqlx::query_as::<_, (
            Uuid, Uuid, String, String, serde_json::Value,
            i64, i64, i64, DateTime<Utc>, DateTime<Utc>, bool,
        )>(
            r#"
            INSERT INTO immune_memories
                (tenant_id, failure_signature, failure_class, remediation_pattern,
                 success_count, failure_count, avg_resolution_time_ms, first_seen, last_seen, auto_remediate)
            VALUES ($1, $2, $3, $4,
                    CASE WHEN $5 THEN 1 ELSE 0 END,
                    CASE WHEN $5 THEN 0 ELSE 1 END,
                    $6, NOW(), NOW(),
                    FALSE)
            ON CONFLICT (tenant_id, failure_signature) DO UPDATE SET
                remediation_pattern = CASE WHEN $5 THEN $4 ELSE immune_memories.remediation_pattern END,
                success_count = immune_memories.success_count + CASE WHEN $5 THEN 1 ELSE 0 END,
                failure_count = immune_memories.failure_count + CASE WHEN $5 THEN 0 ELSE 1 END,
                avg_resolution_time_ms = (
                    (immune_memories.avg_resolution_time_ms *
                     (immune_memories.success_count + immune_memories.failure_count) + $6)
                    / (immune_memories.success_count + immune_memories.failure_count + 1)
                ),
                last_seen = NOW(),
                auto_remediate = CASE
                    WHEN (immune_memories.success_count + CASE WHEN $5 THEN 1 ELSE 0 END) >= 5
                         AND (immune_memories.success_count + CASE WHEN $5 THEN 1 ELSE 0 END)::FLOAT
                             / GREATEST((immune_memories.success_count + immune_memories.failure_count + 1)::FLOAT, 1.0) > 0.9
                    THEN TRUE
                    ELSE immune_memories.auto_remediate
                END
            RETURNING id, tenant_id, failure_signature, failure_class, remediation_pattern,
                      success_count, failure_count, avg_resolution_time_ms,
                      first_seen, last_seen, auto_remediate
            "#,
        )
        .bind(tenant_id)
        .bind(&req.failure_signature)
        .bind(&req.failure_class)
        .bind(&req.remediation_pattern)
        .bind(req.success)
        .bind(req.resolution_time_ms)
        .fetch_one(&self.pool)
        .await?;

        info!(
            tenant_id = %tenant_id,
            failure_signature = %req.failure_signature,
            success = %req.success,
            auto_remediate = %row.10,
            "Remediation recorded in immune memory"
        );

        Ok(ImmuneMemory {
            id: row.0,
            tenant_id: row.1,
            failure_signature: row.2,
            failure_class: row.3,
            remediation_pattern: row.4,
            success_count: row.5,
            failure_count: row.6,
            avg_resolution_time_ms: row.7,
            first_seen: row.8,
            last_seen: row.9,
            auto_remediate: row.10,
        })
    }

    /// List all immune memories for a tenant
    pub async fn list_immune_memories(&self, tenant_id: Uuid) -> Result<Vec<ImmuneMemory>> {
        let rows = sqlx::query_as::<_, (
            Uuid, Uuid, String, String, serde_json::Value,
            i64, i64, i64, DateTime<Utc>, DateTime<Utc>, bool,
        )>(
            r#"
            SELECT id, tenant_id, failure_signature, failure_class, remediation_pattern,
                   success_count, failure_count, avg_resolution_time_ms,
                   first_seen, last_seen, auto_remediate
            FROM immune_memories
            WHERE tenant_id = $1
            ORDER BY last_seen DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ImmuneMemory {
                id: r.0,
                tenant_id: r.1,
                failure_signature: r.2,
                failure_class: r.3,
                remediation_pattern: r.4,
                success_count: r.5,
                failure_count: r.6,
                avg_resolution_time_ms: r.7,
                first_seen: r.8,
                last_seen: r.9,
                auto_remediate: r.10,
            })
            .collect())
    }

    // ========================================================================
    // Vaccination Methods (Cross-Tenant Protection)
    // ========================================================================

    /// Distribute a vaccination: take the known countermeasure for a failure signature
    /// from the source tenant and make it available to all tenants.
    pub async fn distribute_vaccination(
        &self,
        tenant_id: Uuid,
        failure_signature: String,
    ) -> Result<VaccinationRecord> {
        // Look up the immune memory from the source tenant
        let memory = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT remediation_pattern
            FROM immune_memories
            WHERE tenant_id = $1 AND failure_signature = $2 AND auto_remediate = TRUE
            "#,
        )
        .bind(tenant_id)
        .bind(&failure_signature)
        .fetch_optional(&self.pool)
        .await?;

        let countermeasure = match memory {
            Some((pattern,)) => pattern,
            None => {
                anyhow::bail!(
                    "No proven remediation pattern found for signature '{}' in tenant {}",
                    failure_signature, tenant_id
                );
            }
        };

        // Find all other tenants that have seen the same failure signature
        let applied_rows = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT DISTINCT tenant_id
            FROM immune_memories
            WHERE failure_signature = $1 AND tenant_id != $2
            "#,
        )
        .bind(&failure_signature)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let applied_tenants: Vec<Uuid> = applied_rows.into_iter().map(|r| r.0).collect();

        // Create the vaccination record
        let row = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
            r#"
            INSERT INTO vaccination_records
                (source_tenant_id, failure_signature, countermeasure, applied_tenants, effectiveness_score)
            VALUES ($1, $2, $3, $4, 0.0)
            RETURNING id, distributed_at
            "#,
        )
        .bind(tenant_id)
        .bind(&failure_signature)
        .bind(&countermeasure)
        .bind(serde_json::to_value(&applied_tenants)?)
        .fetch_one(&self.pool)
        .await?;

        // Update immune memories for applied tenants: insert the countermeasure if
        // they don't already have one, or update if theirs has lower success rate
        for applied_tenant in &applied_tenants {
            let _ = sqlx::query(
                r#"
                INSERT INTO immune_memories
                    (tenant_id, failure_signature, failure_class, remediation_pattern,
                     success_count, failure_count, avg_resolution_time_ms, auto_remediate)
                VALUES ($1, $2, 'vaccinated', $3, 0, 0, 0, FALSE)
                ON CONFLICT (tenant_id, failure_signature) DO UPDATE SET
                    remediation_pattern = CASE
                        WHEN immune_memories.success_count = 0 THEN $3
                        ELSE immune_memories.remediation_pattern
                    END
                "#,
            )
            .bind(applied_tenant)
            .bind(&failure_signature)
            .bind(&countermeasure)
            .execute(&self.pool)
            .await;
        }

        info!(
            source_tenant = %tenant_id,
            failure_signature = %failure_signature,
            applied_count = applied_tenants.len(),
            "Vaccination distributed"
        );

        Ok(VaccinationRecord {
            id: row.0,
            source_tenant_id: tenant_id,
            failure_signature,
            countermeasure,
            distributed_at: row.1,
            applied_tenants,
            effectiveness_score: 0.0,
        })
    }

    /// Get vaccination history across all tenants
    pub async fn get_vaccination_history(&self) -> Result<Vec<VaccinationRecord>> {
        let rows = sqlx::query_as::<_, (
            Uuid, Uuid, String, serde_json::Value, DateTime<Utc>, serde_json::Value, f64,
        )>(
            r#"
            SELECT id, source_tenant_id, failure_signature, countermeasure,
                   distributed_at, applied_tenants, effectiveness_score
            FROM vaccination_records
            ORDER BY distributed_at DESC
            LIMIT 200
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let applied: Vec<Uuid> = serde_json::from_value(r.5.clone()).unwrap_or_default();
                VaccinationRecord {
                    id: r.0,
                    source_tenant_id: r.1,
                    failure_signature: r.2,
                    countermeasure: r.3,
                    distributed_at: r.4,
                    applied_tenants: applied,
                    effectiveness_score: r.6,
                }
            })
            .collect())
    }

    // ========================================================================
    // Status & Scoring Methods
    // ========================================================================

    /// Get overall immune system status for a tenant
    pub async fn get_immune_status(&self, tenant_id: Uuid) -> Result<ImmuneStatus> {
        // Count active (unresolved) signals by severity
        let severity_counts = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT severity, COUNT(*)
            FROM danger_signals
            WHERE tenant_id = $1 AND resolved_at IS NULL
            GROUP BY severity
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut critical: i64 = 0;
        let mut high: i64 = 0;
        let mut medium: i64 = 0;
        let mut low: i64 = 0;
        for (sev, count) in &severity_counts {
            match sev.as_str() {
                "critical" => critical = *count,
                "high" => high = *count,
                "medium" => medium = *count,
                "low" => low = *count,
                _ => {}
            }
        }
        let active_signals = critical + high + medium + low;

        // Count open correlations (from last 24 hours)
        let open_correlations = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM correlation_results
            WHERE tenant_id = $1 AND created_at >= NOW() - INTERVAL '24 hours'
                  AND diagnosis != 'false_positive'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?
        .0;

        // Count immune memories
        let memory_counts = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE auto_remediate = TRUE)
            FROM immune_memories
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        // Compute health score: 100 - penalty for active signals
        let penalty = (critical as f64 * 25.0)
            + (high as f64 * 10.0)
            + (medium as f64 * 4.0)
            + (low as f64 * 1.0);
        let health_score = (100.0 - penalty).max(0.0).min(100.0);

        Ok(ImmuneStatus {
            tenant_id,
            active_signals,
            critical_signals: critical,
            high_signals: high,
            medium_signals: medium,
            low_signals: low,
            open_correlations,
            known_failure_patterns: memory_counts.0,
            auto_remediation_capable: memory_counts.1,
            health_score,
            evaluated_at: Utc::now(),
        })
    }

    /// Compute a composite system resilience score for a tenant
    pub async fn get_system_resilience_score(&self, tenant_id: Uuid) -> Result<ResilienceScore> {
        // Signal resolution rate: resolved / total (last 30 days)
        let resolution = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE resolved_at IS NOT NULL)
            FROM danger_signals
            WHERE tenant_id = $1 AND detected_at >= NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let signal_resolution_rate = if resolution.0 > 0 {
            resolution.1 as f64 / resolution.0 as f64
        } else {
            1.0 // No signals = perfect resolution
        };

        // Auto-heal coverage
        let memory = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE auto_remediate = TRUE)
            FROM immune_memories
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let auto_heal_coverage = if memory.0 > 0 {
            memory.1 as f64 / memory.0 as f64
        } else {
            0.0
        };

        // Mean time to resolve (only resolved signals)
        let mttr = sqlx::query_as::<_, (Option<f64>,)>(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (resolved_at - detected_at)) * 1000)
            FROM danger_signals
            WHERE tenant_id = $1 AND resolved_at IS NOT NULL
                  AND detected_at >= NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let mean_time_to_resolve_ms = mttr.0.unwrap_or(0.0);

        // False positive rate
        let fp = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*),
                COUNT(*) FILTER (WHERE diagnosis = 'false_positive')
            FROM correlation_results
            WHERE tenant_id = $1 AND created_at >= NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let false_positive_rate = if fp.0 > 0 {
            fp.1 as f64 / fp.0 as f64
        } else {
            0.0
        };

        // Vaccination coverage
        let vax_count = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM vaccination_records
            WHERE applied_tenants @> to_jsonb($1::TEXT)
               OR source_tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        // Composite score (0-100)
        let composite_score = (
            signal_resolution_rate * 30.0       // 30% weight
            + auto_heal_coverage * 25.0          // 25% weight
            + (1.0 - false_positive_rate) * 15.0 // 15% weight (inverted: fewer FPs = better)
            + (if mean_time_to_resolve_ms > 0.0 && mean_time_to_resolve_ms < 300_000.0 {
                // Fast resolution bonus (under 5 min = full marks)
                (1.0 - (mean_time_to_resolve_ms / 300_000.0).min(1.0)) * 20.0
            } else if mean_time_to_resolve_ms == 0.0 {
                20.0 // No signals to resolve = full marks
            } else {
                0.0
            })
            + (vax_count.0 as f64).min(10.0) // 10% weight (cap at 10 vaccinations)
        )
        .max(0.0)
        .min(100.0);

        Ok(ResilienceScore {
            tenant_id,
            signal_resolution_rate,
            auto_heal_coverage,
            mean_time_to_resolve_ms,
            false_positive_rate,
            vaccination_coverage: vax_count.0,
            composite_score,
            evaluated_at: Utc::now(),
        })
    }
}

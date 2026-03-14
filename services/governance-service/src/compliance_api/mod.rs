pub mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// =============================================================================
// Types — Governance-as-a-Service API
// =============================================================================

/// External policy decision request (from downstream systems)
#[derive(Debug, Clone, Deserialize)]
pub struct PolicyDecisionRequest {
    pub requesting_system: String,
    pub subject_type: String,
    pub subject_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub context: serde_json::Value,
}

/// Policy decision response
#[derive(Debug, Clone, Serialize)]
pub struct PolicyDecisionResponse {
    pub decision_id: Uuid,
    pub allowed: bool,
    pub reason: String,
    pub matched_policies: Vec<String>,
    pub conditions: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
    pub evaluation_time_ms: i64,
}

/// Policy decision audit record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyDecisionRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub requesting_system: String,
    pub subject_type: String,
    pub subject_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub context: serde_json::Value,
    pub decision: String,
    pub reason: String,
    pub matched_policies: serde_json::Value,
    pub evaluation_time_ms: i64,
    pub created_at: DateTime<Utc>,
}

/// Compliance score for an entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceScore {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub overall_score: f64,
    pub policy_adherence_score: f64,
    pub data_quality_score: f64,
    pub access_control_score: f64,
    pub audit_completeness_score: f64,
    pub risk_level: String,
    pub issues_count: i32,
    pub last_evaluated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance report export format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Json,
    Pdf,
    Csv,
    Html,
}

/// Auto-generated compliance report
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceReportRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub framework: String,
    pub report_type: String,
    pub format: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub overall_status: String,
    pub compliance_score: f64,
    pub total_controls: i32,
    pub passing_controls: i32,
    pub failing_controls: i32,
    pub report_data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<Uuid>,
}

/// Regulatory change tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegulatoryChange {
    pub id: Uuid,
    pub regulation: String,
    pub change_type: String,
    pub title: String,
    pub description: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub impact_assessment: Option<serde_json::Value>,
    pub proposed_policy_updates: serde_json::Value,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to generate a compliance report
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    pub framework: String,
    pub report_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub format: Option<String>,
}

/// Request to record a regulatory change
#[derive(Debug, Clone, Deserialize)]
pub struct RecordRegulatoryChangeRequest {
    pub regulation: String,
    pub change_type: String,
    pub title: String,
    pub description: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub proposed_policy_updates: serde_json::Value,
}

// =============================================================================
// Service
// =============================================================================

pub struct ComplianceApiService {
    pool: PgPool,
}

impl ComplianceApiService {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    // =========================================================================
    // Governance API — Policy Decision Point
    // =========================================================================

    pub async fn evaluate_policy_decision(
        &self,
        tenant_id: Uuid,
        req: PolicyDecisionRequest,
    ) -> Result<PolicyDecisionResponse> {
        let start = std::time::Instant::now();

        // Evaluate against stored policies
        let policies = sqlx::query_as::<_, (Uuid, String, String, serde_json::Value)>(
            "SELECT id, name, policy_type, rules FROM policies \
             WHERE tenant_id = $1 AND status = 'active' \
             ORDER BY priority DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut allowed = true;
        let mut reasons = Vec::new();
        let mut matched = Vec::new();
        let mut conditions = Vec::new();

        for (policy_id, name, policy_type, rules) in &policies {
            // Check if policy applies to this resource/action
            let applies = rules.get("resource_types")
                .and_then(|rt| rt.as_array())
                .map(|types| types.iter().any(|t| t.as_str() == Some(&req.resource_type) || t.as_str() == Some("*")))
                .unwrap_or(true);

            if !applies {
                continue;
            }

            let action_applies = rules.get("actions")
                .and_then(|a| a.as_array())
                .map(|actions| actions.iter().any(|a| a.as_str() == Some(&req.action) || a.as_str() == Some("*")))
                .unwrap_or(true);

            if !action_applies {
                continue;
            }

            matched.push(name.clone());

            // Check deny rules
            if let Some(deny_rules) = rules.get("deny") {
                if let Some(deny_array) = deny_rules.as_array() {
                    for rule in deny_array {
                        if let Some(condition) = rule.get("condition").and_then(|c| c.as_str()) {
                            allowed = false;
                            reasons.push(format!("Denied by policy '{}': {}", name, condition));
                        }
                    }
                }
            }

            // Check conditional rules
            if let Some(require) = rules.get("require") {
                if let Some(req_array) = require.as_array() {
                    for requirement in req_array {
                        if let Some(cond) = requirement.get("condition").and_then(|c| c.as_str()) {
                            conditions.push(cond.to_string());
                        }
                    }
                }
            }
        }

        let evaluation_time_ms = start.elapsed().as_millis() as i64;
        let decision_id = Uuid::new_v4();

        let reason = if reasons.is_empty() {
            if matched.is_empty() {
                "No matching policies found — default allow".to_string()
            } else {
                format!("Allowed by {} matching policies", matched.len())
            }
        } else {
            reasons.join("; ")
        };

        // Record the decision for audit
        sqlx::query(
            "INSERT INTO policy_decisions \
             (id, tenant_id, requesting_system, subject_type, subject_id, \
              resource_type, resource_id, action, context, decision, reason, \
              matched_policies, evaluation_time_ms) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
        )
        .bind(decision_id)
        .bind(tenant_id)
        .bind(&req.requesting_system)
        .bind(&req.subject_type)
        .bind(&req.subject_id)
        .bind(&req.resource_type)
        .bind(&req.resource_id)
        .bind(&req.action)
        .bind(&req.context)
        .bind(if allowed { "allow" } else { "deny" })
        .bind(&reason)
        .bind(serde_json::json!(matched))
        .bind(evaluation_time_ms)
        .execute(&self.pool)
        .await?;

        Ok(PolicyDecisionResponse {
            decision_id,
            allowed,
            reason,
            matched_policies: matched,
            conditions,
            evaluated_at: Utc::now(),
            evaluation_time_ms,
        })
    }

    // =========================================================================
    // Compliance Scoring
    // =========================================================================

    pub async fn calculate_compliance_score(
        &self,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<ComplianceScore> {
        // Gather signals for scoring
        let policy_violations: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM policy_violations \
             WHERE tenant_id = $1 AND entity_id = $2 AND status = 'open'"
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        let total_policies: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM policies WHERE tenant_id = $1 AND status = 'active'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((1,));

        let policy_adherence = if total_policies.0 > 0 {
            ((total_policies.0 - policy_violations.0.min(total_policies.0)) as f64 / total_policies.0 as f64) * 100.0
        } else {
            100.0
        };

        // Audit completeness: check for recent audit log entries
        let audit_entries: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log \
             WHERE tenant_id = $1 AND created_at > NOW() - INTERVAL '30 days'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        let audit_completeness = (audit_entries.0 as f64).min(100.0);

        // Data quality and access control are simulated scores based on available data
        let data_quality_score = 85.0; // Would be populated from quality service
        let access_control_score = 90.0; // Would be populated from RBAC analysis

        let overall = (policy_adherence * 0.35
            + data_quality_score * 0.25
            + access_control_score * 0.25
            + audit_completeness * 0.15)
            .min(100.0);

        let risk_level = if overall >= 90.0 { "low" }
            else if overall >= 70.0 { "medium" }
            else if overall >= 50.0 { "high" }
            else { "critical" };

        // Upsert score
        let score = sqlx::query_as::<_, ComplianceScore>(
            "INSERT INTO compliance_scores \
             (tenant_id, entity_type, entity_id, overall_score, policy_adherence_score, \
              data_quality_score, access_control_score, audit_completeness_score, \
              risk_level, issues_count, last_evaluated) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW()) \
             ON CONFLICT (tenant_id, entity_type, entity_id) DO UPDATE SET \
              overall_score = EXCLUDED.overall_score, \
              policy_adherence_score = EXCLUDED.policy_adherence_score, \
              data_quality_score = EXCLUDED.data_quality_score, \
              access_control_score = EXCLUDED.access_control_score, \
              audit_completeness_score = EXCLUDED.audit_completeness_score, \
              risk_level = EXCLUDED.risk_level, \
              issues_count = EXCLUDED.issues_count, \
              last_evaluated = NOW(), \
              updated_at = NOW() \
             RETURNING id, tenant_id, entity_type, entity_id, overall_score, \
              policy_adherence_score, data_quality_score, access_control_score, \
              audit_completeness_score, risk_level, issues_count, last_evaluated, \
              created_at, updated_at"
        )
        .bind(tenant_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(overall)
        .bind(policy_adherence)
        .bind(data_quality_score)
        .bind(access_control_score)
        .bind(audit_completeness)
        .bind(risk_level)
        .bind(policy_violations.0 as i32)
        .fetch_one(&self.pool)
        .await?;

        Ok(score)
    }

    pub async fn get_compliance_scores(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<ComplianceScore>> {
        let scores = sqlx::query_as::<_, ComplianceScore>(
            "SELECT id, tenant_id, entity_type, entity_id, overall_score, \
             policy_adherence_score, data_quality_score, access_control_score, \
             audit_completeness_score, risk_level, issues_count, last_evaluated, \
             created_at, updated_at \
             FROM compliance_scores WHERE tenant_id = $1 ORDER BY overall_score ASC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(scores)
    }

    // =========================================================================
    // Compliance Report Generation
    // =========================================================================

    pub async fn generate_compliance_report(
        &self,
        tenant_id: Uuid,
        req: GenerateReportRequest,
        generated_by: Option<Uuid>,
    ) -> Result<ComplianceReportRecord> {
        // Gather data for the report
        let scores = self.get_compliance_scores(tenant_id).await?;

        let avg_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().map(|s| s.overall_score).sum::<f64>() / scores.len() as f64
        };

        let high_risk_count = scores.iter().filter(|s| s.risk_level == "high" || s.risk_level == "critical").count();
        let compliant_count = scores.iter().filter(|s| s.overall_score >= 70.0).count();
        let non_compliant_count = scores.len() - compliant_count;

        let overall_status = if avg_score >= 90.0 { "Substantially Compliant" }
            else if avg_score >= 70.0 { "Partially Compliant" }
            else { "Non-Compliant" };

        let report_data = serde_json::json!({
            "executive_summary": format!(
                "Compliance report for {} framework covering {} to {}. \
                 Overall score: {:.1}%. {} entities assessed, {} compliant, {} require attention.",
                req.framework,
                req.period_start.format("%Y-%m-%d"),
                req.period_end.format("%Y-%m-%d"),
                avg_score,
                scores.len(),
                compliant_count,
                non_compliant_count
            ),
            "scores": scores,
            "high_risk_entities": high_risk_count,
            "recommendations": generate_recommendations(avg_score, high_risk_count),
        });

        let report = sqlx::query_as::<_, ComplianceReportRecord>(
            "INSERT INTO compliance_reports \
             (tenant_id, framework, report_type, format, period_start, period_end, \
              overall_status, compliance_score, total_controls, passing_controls, \
              failing_controls, report_data, generated_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, tenant_id, framework, report_type, format, period_start, \
              period_end, overall_status, compliance_score, total_controls, \
              passing_controls, failing_controls, report_data, generated_at, generated_by"
        )
        .bind(tenant_id)
        .bind(&req.framework)
        .bind(&req.report_type)
        .bind(req.format.as_deref().unwrap_or("json"))
        .bind(req.period_start)
        .bind(req.period_end)
        .bind(overall_status)
        .bind(avg_score)
        .bind(scores.len() as i32)
        .bind(compliant_count as i32)
        .bind(non_compliant_count as i32)
        .bind(&report_data)
        .bind(generated_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    pub async fn list_compliance_reports(
        &self,
        tenant_id: Uuid,
        framework: Option<&str>,
    ) -> Result<Vec<ComplianceReportRecord>> {
        let reports = if let Some(fw) = framework {
            sqlx::query_as::<_, ComplianceReportRecord>(
                "SELECT id, tenant_id, framework, report_type, format, period_start, \
                 period_end, overall_status, compliance_score, total_controls, \
                 passing_controls, failing_controls, report_data, generated_at, generated_by \
                 FROM compliance_reports WHERE tenant_id = $1 AND framework = $2 \
                 ORDER BY generated_at DESC"
            )
            .bind(tenant_id)
            .bind(fw)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ComplianceReportRecord>(
                "SELECT id, tenant_id, framework, report_type, format, period_start, \
                 period_end, overall_status, compliance_score, total_controls, \
                 passing_controls, failing_controls, report_data, generated_at, generated_by \
                 FROM compliance_reports WHERE tenant_id = $1 ORDER BY generated_at DESC"
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(reports)
    }

    // =========================================================================
    // Regulatory Change Tracking
    // =========================================================================

    pub async fn record_regulatory_change(
        &self,
        req: RecordRegulatoryChangeRequest,
    ) -> Result<RegulatoryChange> {
        let change = sqlx::query_as::<_, RegulatoryChange>(
            "INSERT INTO regulatory_changes \
             (regulation, change_type, title, description, effective_date, \
              proposed_policy_updates, status) \
             VALUES ($1, $2, $3, $4, $5, $6, 'pending_review') \
             RETURNING id, regulation, change_type, title, description, effective_date, \
              impact_assessment, proposed_policy_updates, status, reviewed_by, \
              approved_at, created_at, updated_at"
        )
        .bind(&req.regulation)
        .bind(&req.change_type)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.effective_date)
        .bind(&req.proposed_policy_updates)
        .fetch_one(&self.pool)
        .await?;

        Ok(change)
    }

    pub async fn list_regulatory_changes(
        &self,
        status: Option<&str>,
    ) -> Result<Vec<RegulatoryChange>> {
        let changes = if let Some(s) = status {
            sqlx::query_as::<_, RegulatoryChange>(
                "SELECT id, regulation, change_type, title, description, effective_date, \
                 impact_assessment, proposed_policy_updates, status, reviewed_by, \
                 approved_at, created_at, updated_at \
                 FROM regulatory_changes WHERE status = $1 ORDER BY created_at DESC"
            )
            .bind(s)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RegulatoryChange>(
                "SELECT id, regulation, change_type, title, description, effective_date, \
                 impact_assessment, proposed_policy_updates, status, reviewed_by, \
                 approved_at, created_at, updated_at \
                 FROM regulatory_changes ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(changes)
    }

    pub async fn approve_regulatory_change(
        &self,
        change_id: Uuid,
        reviewed_by: Uuid,
    ) -> Result<RegulatoryChange> {
        let change = sqlx::query_as::<_, RegulatoryChange>(
            "UPDATE regulatory_changes SET \
             status = 'approved', reviewed_by = $1, approved_at = NOW(), updated_at = NOW() \
             WHERE id = $2 \
             RETURNING id, regulation, change_type, title, description, effective_date, \
              impact_assessment, proposed_policy_updates, status, reviewed_by, \
              approved_at, created_at, updated_at"
        )
        .bind(reviewed_by)
        .bind(change_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(change)
    }

    // =========================================================================
    // Policy Decision History
    // =========================================================================

    pub async fn get_decision_history(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PolicyDecisionRecord>> {
        let decisions = sqlx::query_as::<_, PolicyDecisionRecord>(
            "SELECT id, tenant_id, requesting_system, subject_type, subject_id, \
             resource_type, resource_id, action, context, decision, reason, \
             matched_policies, evaluation_time_ms, created_at \
             FROM policy_decisions WHERE tenant_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(decisions)
    }

    pub async fn get_decision_analytics(
        &self,
        tenant_id: Uuid,
    ) -> Result<serde_json::Value> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM policy_decisions WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        let denied: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM policy_decisions WHERE tenant_id = $1 AND decision = 'deny'"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((0,));

        let avg_eval_time: (Option<f64>,) = sqlx::query_as(
            "SELECT AVG(evaluation_time_ms)::float8 FROM policy_decisions WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or((None,));

        Ok(serde_json::json!({
            "total_decisions": total.0,
            "allowed": total.0 - denied.0,
            "denied": denied.0,
            "deny_rate": if total.0 > 0 { denied.0 as f64 / total.0 as f64 * 100.0 } else { 0.0 },
            "avg_evaluation_time_ms": avg_eval_time.0.unwrap_or(0.0),
        }))
    }
}

fn generate_recommendations(avg_score: f64, high_risk_count: usize) -> Vec<String> {
    let mut recommendations = Vec::new();

    if avg_score < 70.0 {
        recommendations.push("Immediate attention required: Overall compliance score is below acceptable threshold. Review and remediate all critical policy violations.".to_string());
    }

    if high_risk_count > 0 {
        recommendations.push(format!(
            "{} high-risk entities detected. Prioritize remediation of these entities to reduce organizational risk exposure.",
            high_risk_count
        ));
    }

    if avg_score < 90.0 {
        recommendations.push("Consider implementing automated policy enforcement to prevent future compliance drift.".to_string());
    }

    recommendations.push("Schedule quarterly compliance reviews to maintain continuous compliance posture.".to_string());

    recommendations
}

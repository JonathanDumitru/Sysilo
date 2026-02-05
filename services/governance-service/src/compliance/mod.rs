use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A compliance framework (e.g., SOC2, GDPR, HIPAA)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceFramework {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub controls: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single control within a framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControl {
    pub control_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub evidence_requirements: Vec<String>,
    pub audit_query: Option<String>,  // Query to run against audit log
    pub policy_ids: Vec<Uuid>,        // Related policies
}

/// Compliance status for a control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    Compliant,
    NonCompliant,
    Partial,
    NotAssessed,
    NotApplicable,
}

impl ControlStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlStatus::Compliant => "compliant",
            ControlStatus::NonCompliant => "non_compliant",
            ControlStatus::Partial => "partial",
            ControlStatus::NotAssessed => "not_assessed",
            ControlStatus::NotApplicable => "not_applicable",
        }
    }
}

/// Compliance status record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceStatus {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub framework_id: Uuid,
    pub control_id: String,
    pub status: String,
    pub evidence_refs: Option<Vec<Uuid>>,
    pub notes: Option<String>,
    pub assessed_by: Option<Uuid>,
    pub last_assessed: Option<DateTime<Utc>>,
    pub next_review: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance evidence record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceEvidence {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub framework_id: Uuid,
    pub control_id: String,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub collected_by: Option<Uuid>,
    pub valid_until: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Request to add evidence
#[derive(Debug, Clone, Deserialize)]
pub struct AddEvidenceRequest {
    pub framework_id: Uuid,
    pub control_id: String,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub valid_until: Option<DateTime<Utc>>,
}

/// Request to update compliance status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: ControlStatus,
    pub notes: Option<String>,
}

/// Compliance assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub framework_id: Uuid,
    pub framework_name: String,
    pub total_controls: i64,
    pub compliant: i64,
    pub non_compliant: i64,
    pub partial: i64,
    pub not_assessed: i64,
    pub not_applicable: i64,
    pub compliance_score: f64,
    pub assessed_at: DateTime<Utc>,
    pub control_results: Vec<ControlAssessmentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlAssessmentResult {
    pub control_id: String,
    pub control_name: String,
    pub status: String,
    pub findings: Vec<String>,
    pub evidence_count: i64,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub tenant_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub assessment_period_start: DateTime<Utc>,
    pub assessment_period_end: DateTime<Utc>,
    pub overall_status: String,
    pub compliance_score: f64,
    pub executive_summary: String,
    pub control_details: Vec<ControlReportDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlReportDetail {
    pub control_id: String,
    pub control_name: String,
    pub category: String,
    pub status: String,
    pub notes: Option<String>,
    pub evidence_summary: Vec<String>,
}

/// Service for managing compliance
pub struct ComplianceService {
    pool: PgPool,
}

impl ComplianceService {
    /// Create a new compliance service
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
    // Frameworks
    // ========================================================================

    /// List all available compliance frameworks
    pub async fn list_frameworks(&self) -> Result<Vec<ComplianceFramework>> {
        let frameworks = sqlx::query_as::<_, ComplianceFramework>(
            r#"
            SELECT id, name, description, version, controls, created_at, updated_at
            FROM compliance_frameworks
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(frameworks)
    }

    /// Get a single framework
    pub async fn get_framework(&self, id: Uuid) -> Result<Option<ComplianceFramework>> {
        let framework = sqlx::query_as::<_, ComplianceFramework>(
            r#"
            SELECT id, name, description, version, controls, created_at, updated_at
            FROM compliance_frameworks
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(framework)
    }

    /// Get framework by name
    pub async fn get_framework_by_name(&self, name: &str) -> Result<Option<ComplianceFramework>> {
        let framework = sqlx::query_as::<_, ComplianceFramework>(
            r#"
            SELECT id, name, description, version, controls, created_at, updated_at
            FROM compliance_frameworks
            WHERE LOWER(name) = LOWER($1)
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(framework)
    }

    // ========================================================================
    // Compliance Status
    // ========================================================================

    /// Get compliance status for a tenant and framework
    pub async fn get_status(
        &self,
        tenant_id: Uuid,
        framework_id: Uuid,
    ) -> Result<Vec<ComplianceStatus>> {
        let statuses = sqlx::query_as::<_, ComplianceStatus>(
            r#"
            SELECT id, tenant_id, framework_id, control_id, status,
                   evidence_refs, notes, assessed_by, last_assessed,
                   next_review, created_at, updated_at
            FROM compliance_status
            WHERE tenant_id = $1 AND framework_id = $2
            ORDER BY control_id
            "#
        )
        .bind(tenant_id)
        .bind(framework_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(statuses)
    }

    /// Update compliance status for a control
    pub async fn update_status(
        &self,
        tenant_id: Uuid,
        framework_id: Uuid,
        control_id: &str,
        req: UpdateStatusRequest,
        assessed_by: Option<Uuid>,
    ) -> Result<ComplianceStatus> {
        // Upsert the status
        let status = sqlx::query_as::<_, ComplianceStatus>(
            r#"
            INSERT INTO compliance_status
                (tenant_id, framework_id, control_id, status, notes, assessed_by, last_assessed)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (tenant_id, framework_id, control_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                notes = EXCLUDED.notes,
                assessed_by = EXCLUDED.assessed_by,
                last_assessed = NOW(),
                updated_at = NOW()
            RETURNING id, tenant_id, framework_id, control_id, status,
                      evidence_refs, notes, assessed_by, last_assessed,
                      next_review, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(framework_id)
        .bind(control_id)
        .bind(req.status.as_str())
        .bind(&req.notes)
        .bind(assessed_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(status)
    }

    // ========================================================================
    // Evidence
    // ========================================================================

    /// Add evidence for a control
    pub async fn add_evidence(
        &self,
        tenant_id: Uuid,
        req: AddEvidenceRequest,
        collected_by: Option<Uuid>,
    ) -> Result<ComplianceEvidence> {
        let evidence = sqlx::query_as::<_, ComplianceEvidence>(
            r#"
            INSERT INTO compliance_evidence
                (tenant_id, framework_id, control_id, evidence_type, title, description,
                 file_path, metadata, collected_by, valid_until)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tenant_id, framework_id, control_id, evidence_type, title, description,
                      file_path, collected_at, collected_by, valid_until, metadata, created_at
            "#
        )
        .bind(tenant_id)
        .bind(req.framework_id)
        .bind(&req.control_id)
        .bind(&req.evidence_type)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.file_path)
        .bind(&req.metadata)
        .bind(collected_by)
        .bind(req.valid_until)
        .fetch_one(&self.pool)
        .await?;

        // Update the evidence_refs in compliance_status
        sqlx::query(
            r#"
            UPDATE compliance_status SET
                evidence_refs = array_append(COALESCE(evidence_refs, ARRAY[]::uuid[]), $1)
            WHERE tenant_id = $2 AND framework_id = $3 AND control_id = $4
            "#
        )
        .bind(evidence.id)
        .bind(tenant_id)
        .bind(req.framework_id)
        .bind(&req.control_id)
        .execute(&self.pool)
        .await?;

        Ok(evidence)
    }

    /// Get evidence for a control
    pub async fn get_evidence(
        &self,
        tenant_id: Uuid,
        framework_id: Uuid,
        control_id: &str,
    ) -> Result<Vec<ComplianceEvidence>> {
        let evidence = sqlx::query_as::<_, ComplianceEvidence>(
            r#"
            SELECT id, tenant_id, framework_id, control_id, evidence_type, title, description,
                   file_path, collected_at, collected_by, valid_until, metadata, created_at
            FROM compliance_evidence
            WHERE tenant_id = $1 AND framework_id = $2 AND control_id = $3
            ORDER BY collected_at DESC
            "#
        )
        .bind(tenant_id)
        .bind(framework_id)
        .bind(control_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(evidence)
    }

    // ========================================================================
    // Assessment
    // ========================================================================

    /// Run a compliance assessment
    pub async fn run_assessment(
        &self,
        tenant_id: Uuid,
        framework_id: Uuid,
    ) -> Result<AssessmentResult> {
        let framework = self.get_framework(framework_id).await?
            .ok_or_else(|| anyhow::anyhow!("Framework not found"))?;

        let controls: Vec<ComplianceControl> = serde_json::from_value(framework.controls.clone())?;
        let statuses = self.get_status(tenant_id, framework_id).await?;

        // Build a map of control_id -> status
        let status_map: std::collections::HashMap<String, ComplianceStatus> = statuses
            .into_iter()
            .map(|s| (s.control_id.clone(), s))
            .collect();

        let mut compliant = 0i64;
        let mut non_compliant = 0i64;
        let mut partial = 0i64;
        let mut not_assessed = 0i64;
        let mut not_applicable = 0i64;
        let mut control_results = Vec::new();

        for control in &controls {
            let (status, findings) = if let Some(s) = status_map.get(&control.control_id) {
                let findings = s.notes.as_ref()
                    .map(|n| vec![n.clone()])
                    .unwrap_or_default();
                (s.status.clone(), findings)
            } else {
                ("not_assessed".to_string(), vec![])
            };

            match status.as_str() {
                "compliant" => compliant += 1,
                "non_compliant" => non_compliant += 1,
                "partial" => partial += 1,
                "not_applicable" => not_applicable += 1,
                _ => not_assessed += 1,
            }

            // Get evidence count
            let evidence = self.get_evidence(tenant_id, framework_id, &control.control_id).await?;

            control_results.push(ControlAssessmentResult {
                control_id: control.control_id.clone(),
                control_name: control.name.clone(),
                status,
                findings,
                evidence_count: evidence.len() as i64,
            });
        }

        let total_controls = controls.len() as i64;
        let assessed_controls = total_controls - not_assessed - not_applicable;
        let compliance_score = if assessed_controls > 0 {
            ((compliant as f64 + (partial as f64 * 0.5)) / assessed_controls as f64) * 100.0
        } else {
            0.0
        };

        Ok(AssessmentResult {
            framework_id,
            framework_name: framework.name,
            total_controls,
            compliant,
            non_compliant,
            partial,
            not_assessed,
            not_applicable,
            compliance_score,
            assessed_at: Utc::now(),
            control_results,
        })
    }

    /// Generate a compliance report
    pub async fn generate_report(
        &self,
        tenant_id: Uuid,
        framework_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ComplianceReport> {
        let framework = self.get_framework(framework_id).await?
            .ok_or_else(|| anyhow::anyhow!("Framework not found"))?;

        let assessment = self.run_assessment(tenant_id, framework_id).await?;

        let controls: Vec<ComplianceControl> = serde_json::from_value(framework.controls.clone())?;
        let statuses = self.get_status(tenant_id, framework_id).await?;

        let status_map: std::collections::HashMap<String, ComplianceStatus> = statuses
            .into_iter()
            .map(|s| (s.control_id.clone(), s))
            .collect();

        let mut control_details = Vec::new();

        for control in &controls {
            let status_record = status_map.get(&control.control_id);

            // Get evidence summaries
            let evidence = self.get_evidence(tenant_id, framework_id, &control.control_id).await?;
            let evidence_summary: Vec<String> = evidence
                .iter()
                .map(|e| format!("{}: {}", e.evidence_type, e.title))
                .collect();

            control_details.push(ControlReportDetail {
                control_id: control.control_id.clone(),
                control_name: control.name.clone(),
                category: control.category.clone(),
                status: status_record.map(|s| s.status.clone()).unwrap_or_else(|| "not_assessed".to_string()),
                notes: status_record.and_then(|s| s.notes.clone()),
                evidence_summary,
            });
        }

        let overall_status = if assessment.compliance_score >= 90.0 {
            "Substantially Compliant"
        } else if assessment.compliance_score >= 70.0 {
            "Partially Compliant"
        } else {
            "Non-Compliant"
        };

        let executive_summary = format!(
            "This compliance report covers the {} framework for the assessment period {} to {}. \
             Overall compliance score: {:.1}% ({} of {} controls compliant). \
             {} controls require attention.",
            framework.name,
            start_time.format("%Y-%m-%d"),
            end_time.format("%Y-%m-%d"),
            assessment.compliance_score,
            assessment.compliant,
            assessment.total_controls - assessment.not_applicable,
            assessment.non_compliant + assessment.partial,
        );

        Ok(ComplianceReport {
            framework,
            tenant_id,
            generated_at: Utc::now(),
            assessment_period_start: start_time,
            assessment_period_end: end_time,
            overall_status: overall_status.to_string(),
            compliance_score: assessment.compliance_score,
            executive_summary,
            control_details,
        })
    }

    /// Get compliance summary across all frameworks for a tenant
    pub async fn get_summary(&self, tenant_id: Uuid) -> Result<Vec<AssessmentResult>> {
        let frameworks = self.list_frameworks().await?;
        let mut results = Vec::new();

        for framework in frameworks {
            let assessment = self.run_assessment(tenant_id, framework.id).await?;
            results.push(assessment);
        }

        Ok(results)
    }
}

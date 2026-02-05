use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Severity levels for quality issues
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "severity", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Types of quality rules
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rule_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Completeness,
    Uniqueness,
    Validity,
    Consistency,
    Timeliness,
    Custom,
}

/// A data quality rule definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QualityRule {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub expression: String,
    pub severity: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a single rule execution
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QualityCheckResult {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub entity_id: Uuid,
    pub passed: bool,
    pub records_checked: i64,
    pub records_failed: i64,
    pub failure_rate: f64,
    pub sample_failures: Option<serde_json::Value>,
    pub executed_at: DateTime<Utc>,
}

/// Aggregated quality score for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub entity_id: Uuid,
    pub overall_score: f64,
    pub dimension_scores: DimensionScores,
    pub rules_passed: i32,
    pub rules_failed: i32,
    pub last_checked: DateTime<Utc>,
}

/// Scores broken down by quality dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScores {
    pub completeness: Option<f64>,
    pub uniqueness: Option<f64>,
    pub validity: Option<f64>,
    pub consistency: Option<f64>,
    pub timeliness: Option<f64>,
}

/// A quality issue detected by rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub severity: String,
    pub failure_rate: f64,
    pub records_affected: i64,
    pub description: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Service for managing data quality
pub struct QualityService {
    pool: PgPool,
}

impl QualityService {
    /// Create a new quality service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// List quality rules, optionally filtered by entity
    pub async fn list_rules(
        &self,
        tenant_id: Uuid,
        entity_id: Option<Uuid>,
    ) -> Result<Vec<QualityRule>> {
        let rules = if let Some(eid) = entity_id {
            sqlx::query_as::<_, QualityRule>(
                r#"
                SELECT id, tenant_id, entity_id, name, rule_type, expression,
                       severity, description, enabled, created_at, updated_at
                FROM quality_rules
                WHERE tenant_id = $1 AND entity_id = $2
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .bind(eid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, QualityRule>(
                r#"
                SELECT id, tenant_id, entity_id, name, rule_type, expression,
                       severity, description, enabled, created_at, updated_at
                FROM quality_rules
                WHERE tenant_id = $1
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rules)
    }

    /// Create a new quality rule
    pub async fn create_rule(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        name: String,
        rule_type: String,
        expression: String,
        severity: String,
        description: Option<String>,
    ) -> Result<QualityRule> {
        let rule = sqlx::query_as::<_, QualityRule>(
            r#"
            INSERT INTO quality_rules
                (tenant_id, entity_id, name, rule_type, expression, severity, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, tenant_id, entity_id, name, rule_type, expression,
                      severity, description, enabled, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .bind(name)
        .bind(rule_type)
        .bind(expression)
        .bind(severity)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Get quality score for an entity
    pub async fn get_score(&self, tenant_id: Uuid, entity_id: Uuid) -> Result<Option<QualityScore>> {
        // Get latest check results for each rule
        let results: Vec<(String, bool, i64, i64, f64, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (qr.id)
                   qr.rule_type, qcr.passed, qcr.records_checked,
                   qcr.records_failed, qcr.failure_rate, qcr.executed_at
            FROM quality_rules qr
            JOIN quality_check_results qcr ON qcr.rule_id = qr.id
            WHERE qr.tenant_id = $1 AND qr.entity_id = $2 AND qr.enabled = true
            ORDER BY qr.id, qcr.executed_at DESC
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        if results.is_empty() {
            return Ok(None);
        }

        // Calculate dimension scores
        let mut completeness_scores: Vec<f64> = vec![];
        let mut uniqueness_scores: Vec<f64> = vec![];
        let mut validity_scores: Vec<f64> = vec![];
        let mut consistency_scores: Vec<f64> = vec![];
        let mut timeliness_scores: Vec<f64> = vec![];

        let mut rules_passed = 0;
        let mut rules_failed = 0;
        let mut last_checked = Utc::now();

        for (rule_type, passed, _, _, failure_rate, executed_at) in &results {
            let score = 1.0 - failure_rate;

            match rule_type.as_str() {
                "completeness" => completeness_scores.push(score),
                "uniqueness" => uniqueness_scores.push(score),
                "validity" => validity_scores.push(score),
                "consistency" => consistency_scores.push(score),
                "timeliness" => timeliness_scores.push(score),
                _ => {}
            }

            if *passed {
                rules_passed += 1;
            } else {
                rules_failed += 1;
            }

            if *executed_at > last_checked {
                last_checked = *executed_at;
            }
        }

        let avg = |scores: &[f64]| -> Option<f64> {
            if scores.is_empty() {
                None
            } else {
                Some(scores.iter().sum::<f64>() / scores.len() as f64)
            }
        };

        let dimension_scores = DimensionScores {
            completeness: avg(&completeness_scores),
            uniqueness: avg(&uniqueness_scores),
            validity: avg(&validity_scores),
            consistency: avg(&consistency_scores),
            timeliness: avg(&timeliness_scores),
        };

        // Calculate overall score as weighted average
        let all_scores: Vec<f64> = results.iter().map(|r| 1.0 - r.4).collect();
        let overall_score = all_scores.iter().sum::<f64>() / all_scores.len() as f64;

        Ok(Some(QualityScore {
            entity_id,
            overall_score,
            dimension_scores,
            rules_passed,
            rules_failed,
            last_checked,
        }))
    }

    /// Get active quality issues for an entity
    pub async fn get_issues(&self, tenant_id: Uuid, entity_id: Uuid) -> Result<Vec<QualityIssue>> {
        let issues: Vec<(Uuid, String, String, f64, i64, Option<String>, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (qr.id)
                   qr.id, qr.name, qr.severity, qcr.failure_rate,
                   qcr.records_failed, qr.description, qcr.executed_at
            FROM quality_rules qr
            JOIN quality_check_results qcr ON qcr.rule_id = qr.id
            WHERE qr.tenant_id = $1
              AND qr.entity_id = $2
              AND qr.enabled = true
              AND qcr.passed = false
            ORDER BY qr.id, qcr.executed_at DESC
            "#
        )
        .bind(tenant_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        let quality_issues = issues
            .into_iter()
            .map(|(rule_id, rule_name, severity, failure_rate, records_affected, description, detected_at)| {
                QualityIssue {
                    rule_id,
                    rule_name,
                    severity,
                    failure_rate,
                    records_affected,
                    description,
                    detected_at,
                }
            })
            .collect();

        Ok(quality_issues)
    }

    /// Record a quality check result
    pub async fn record_check_result(
        &self,
        rule_id: Uuid,
        entity_id: Uuid,
        passed: bool,
        records_checked: i64,
        records_failed: i64,
        sample_failures: Option<serde_json::Value>,
    ) -> Result<QualityCheckResult> {
        let failure_rate = if records_checked > 0 {
            records_failed as f64 / records_checked as f64
        } else {
            0.0
        };

        let result = sqlx::query_as::<_, QualityCheckResult>(
            r#"
            INSERT INTO quality_check_results
                (rule_id, entity_id, passed, records_checked, records_failed,
                 failure_rate, sample_failures)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, rule_id, entity_id, passed, records_checked,
                      records_failed, failure_rate, sample_failures, executed_at
            "#
        )
        .bind(rule_id)
        .bind(entity_id)
        .bind(passed)
        .bind(records_checked)
        .bind(records_failed)
        .bind(failure_rate)
        .bind(sample_failures)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Toggle rule enabled status
    pub async fn toggle_rule(&self, tenant_id: Uuid, rule_id: Uuid, enabled: bool) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE quality_rules
            SET enabled = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND id = $3
            "#
        )
        .bind(enabled)
        .bind(tenant_id)
        .bind(rule_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a quality rule
    pub async fn delete_rule(&self, tenant_id: Uuid, rule_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM quality_rules WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(rule_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

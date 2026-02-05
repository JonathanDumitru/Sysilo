use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimeQuadrant {
    Tolerate,
    Invest,
    Migrate,
    Eliminate,
}

impl TimeQuadrant {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeQuadrant::Tolerate => "tolerate",
            TimeQuadrant::Invest => "invest",
            TimeQuadrant::Migrate => "migrate",
            TimeQuadrant::Eliminate => "eliminate",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        match s.to_lowercase().as_str() {
            "tolerate" => Ok(TimeQuadrant::Tolerate),
            "invest" => Ok(TimeQuadrant::Invest),
            "migrate" => Ok(TimeQuadrant::Migrate),
            "eliminate" => Ok(TimeQuadrant::Eliminate),
            _ => Err("Invalid quadrant"),
        }
    }

    /// Determine quadrant based on value and health scores
    /// High Value + High Health = Invest
    /// High Value + Low Health = Migrate
    /// Low Value + High Health = Tolerate
    /// Low Value + Low Health = Eliminate
    pub fn from_scores(value_score: Decimal, health_score: Decimal) -> Self {
        let threshold = Decimal::new(5, 0); // 5.0 is the midpoint on a 0-10 scale

        if value_score >= threshold {
            if health_score >= threshold {
                TimeQuadrant::Invest
            } else {
                TimeQuadrant::Migrate
            }
        } else {
            if health_score >= threshold {
                TimeQuadrant::Tolerate
            } else {
                TimeQuadrant::Eliminate
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoringDimension {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub weight: Decimal,
    pub scoring_criteria: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApplicationScore {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub application_id: Uuid,
    pub dimension_id: Uuid,
    pub score: Decimal,
    pub notes: Option<String>,
    pub evidence: Option<serde_json::Value>,
    pub scored_by: Option<Uuid>,
    pub scored_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimeAssessment {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub application_id: Uuid,
    pub quadrant: String,
    pub business_value_score: Option<Decimal>,
    pub technical_health_score: Option<Decimal>,
    pub is_override: bool,
    pub override_reason: Option<String>,
    pub recommended_actions: Option<serde_json::Value>,
    pub assessed_by: Option<Uuid>,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApplicationDependency {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_application_id: Uuid,
    pub target_application_id: Uuid,
    pub dependency_type: String,
    pub criticality: String,
    pub description: Option<String>,
    pub integration_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ImpactAnalysis {
    pub application_id: Uuid,
    pub direct_dependents: Vec<ApplicationDependency>,
    pub direct_dependencies: Vec<ApplicationDependency>,
    pub transitive_impact_count: i64,
    pub critical_dependencies: i64,
    pub risk_level: String,
}

// ============================================================================
// Service
// ============================================================================

pub struct ScoringService {
    pool: PgPool,
}

impl ScoringService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Dimensions
    // ========================================================================

    pub async fn list_dimensions(&self, tenant_id: Uuid) -> Result<Vec<ScoringDimension>> {
        let dimensions = sqlx::query_as::<_, ScoringDimension>(
            r#"
            SELECT id, tenant_id, name, description, category, weight,
                   scoring_criteria, enabled, created_at
            FROM scoring_dimensions
            WHERE tenant_id = $1 AND enabled = true
            ORDER BY category, name
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(dimensions)
    }

    pub async fn create_dimension(
        &self,
        tenant_id: Uuid,
        name: String,
        description: Option<String>,
        category: String,
        weight: Decimal,
        scoring_criteria: serde_json::Value,
    ) -> Result<ScoringDimension> {
        let dimension = sqlx::query_as::<_, ScoringDimension>(
            r#"
            INSERT INTO scoring_dimensions (tenant_id, name, description, category, weight, scoring_criteria)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, description, category, weight, scoring_criteria, enabled, created_at
            "#
        )
        .bind(tenant_id)
        .bind(&name)
        .bind(&description)
        .bind(&category)
        .bind(weight)
        .bind(&scoring_criteria)
        .fetch_one(&self.pool)
        .await?;

        Ok(dimension)
    }

    pub async fn update_dimension(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: String,
        description: Option<String>,
        category: String,
        weight: Decimal,
        scoring_criteria: serde_json::Value,
    ) -> Result<Option<ScoringDimension>> {
        let dimension = sqlx::query_as::<_, ScoringDimension>(
            r#"
            UPDATE scoring_dimensions SET
                name = $1, description = $2, category = $3, weight = $4, scoring_criteria = $5
            WHERE id = $6 AND tenant_id = $7
            RETURNING id, tenant_id, name, description, category, weight, scoring_criteria, enabled, created_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&category)
        .bind(weight)
        .bind(&scoring_criteria)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(dimension)
    }

    // ========================================================================
    // Scores
    // ========================================================================

    pub async fn get_scores(&self, tenant_id: Uuid, application_id: Uuid) -> Result<Vec<ApplicationScore>> {
        let scores = sqlx::query_as::<_, ApplicationScore>(
            r#"
            SELECT id, tenant_id, application_id, dimension_id, score,
                   notes, evidence, scored_by, scored_at
            FROM application_scores
            WHERE tenant_id = $1 AND application_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(scores)
    }

    pub async fn set_score(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
        dimension_id: Uuid,
        score: Decimal,
        notes: Option<String>,
        evidence: Option<serde_json::Value>,
    ) -> Result<ApplicationScore> {
        // Upsert score
        let app_score = sqlx::query_as::<_, ApplicationScore>(
            r#"
            INSERT INTO application_scores (tenant_id, application_id, dimension_id, score, notes, evidence)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (application_id, dimension_id) DO UPDATE SET
                score = EXCLUDED.score,
                notes = EXCLUDED.notes,
                evidence = EXCLUDED.evidence,
                scored_at = NOW()
            RETURNING id, tenant_id, application_id, dimension_id, score, notes, evidence, scored_by, scored_at
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .bind(dimension_id)
        .bind(score)
        .bind(&notes)
        .bind(&evidence)
        .fetch_one(&self.pool)
        .await?;

        // Record in history
        sqlx::query(
            r#"
            INSERT INTO application_score_history (tenant_id, application_id, dimension_id, score)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .bind(dimension_id)
        .bind(score)
        .execute(&self.pool)
        .await?;

        Ok(app_score)
    }

    // ========================================================================
    // TIME Assessment
    // ========================================================================

    pub async fn calculate_time_assessment(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<TimeAssessment> {
        // Get weighted scores by category
        let scores = sqlx::query_as::<_, (String, Decimal)>(
            r#"
            SELECT sd.category, SUM(s.score * sd.weight) / SUM(sd.weight) as weighted_score
            FROM application_scores s
            JOIN scoring_dimensions sd ON sd.id = s.dimension_id
            WHERE s.tenant_id = $1 AND s.application_id = $2 AND sd.enabled = true
            GROUP BY sd.category
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;

        let mut value_score = Decimal::ZERO;
        let mut health_score = Decimal::ZERO;
        let mut value_count = 0;
        let mut health_count = 0;

        for (category, score) in &scores {
            match category.as_str() {
                "value" | "fit" => {
                    value_score += score;
                    value_count += 1;
                }
                "health" | "complexity" => {
                    // For complexity, invert the score (high complexity = low health)
                    if category == "complexity" {
                        health_score += Decimal::new(10, 0) - score;
                    } else {
                        health_score += score;
                    }
                    health_count += 1;
                }
                "cost" => {
                    // Cost efficiency contributes to health
                    health_score += score;
                    health_count += 1;
                }
                _ => {}
            }
        }

        // Average the scores
        if value_count > 0 {
            value_score = value_score / Decimal::from(value_count);
        }
        if health_count > 0 {
            health_score = health_score / Decimal::from(health_count);
        }

        let quadrant = TimeQuadrant::from_scores(value_score, health_score);

        // Generate recommended actions based on quadrant
        let recommended_actions = self.generate_recommended_actions(&quadrant);

        // Upsert assessment
        let assessment = sqlx::query_as::<_, TimeAssessment>(
            r#"
            INSERT INTO time_assessments (
                tenant_id, application_id, quadrant, business_value_score,
                technical_health_score, recommended_actions
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (application_id) DO UPDATE SET
                quadrant = EXCLUDED.quadrant,
                business_value_score = EXCLUDED.business_value_score,
                technical_health_score = EXCLUDED.technical_health_score,
                recommended_actions = EXCLUDED.recommended_actions,
                is_override = false,
                override_reason = NULL,
                assessed_at = NOW()
            RETURNING id, tenant_id, application_id, quadrant, business_value_score,
                      technical_health_score, is_override, override_reason,
                      recommended_actions, assessed_by, assessed_at
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .bind(quadrant.as_str())
        .bind(value_score)
        .bind(health_score)
        .bind(&recommended_actions)
        .fetch_one(&self.pool)
        .await?;

        Ok(assessment)
    }

    fn generate_recommended_actions(&self, quadrant: &TimeQuadrant) -> serde_json::Value {
        match quadrant {
            TimeQuadrant::Invest => serde_json::json!([
                {"action": "Prioritize for strategic initiatives", "priority": "high", "timeline": "immediate"},
                {"action": "Allocate resources for feature development", "priority": "medium", "timeline": "quarterly"},
                {"action": "Consider platform expansion", "priority": "medium", "timeline": "annual"}
            ]),
            TimeQuadrant::Tolerate => serde_json::json!([
                {"action": "Maintain current investment level", "priority": "low", "timeline": "ongoing"},
                {"action": "Monitor for changes in business value", "priority": "medium", "timeline": "quarterly"},
                {"action": "Document for knowledge preservation", "priority": "low", "timeline": "annual"}
            ]),
            TimeQuadrant::Migrate => serde_json::json!([
                {"action": "Develop migration roadmap", "priority": "high", "timeline": "immediate"},
                {"action": "Identify target platform or replacement", "priority": "high", "timeline": "quarterly"},
                {"action": "Plan data migration strategy", "priority": "high", "timeline": "quarterly"},
                {"action": "Execute phased migration", "priority": "high", "timeline": "annual"}
            ]),
            TimeQuadrant::Eliminate => serde_json::json!([
                {"action": "Identify replacement solution", "priority": "high", "timeline": "immediate"},
                {"action": "Plan sunset timeline", "priority": "high", "timeline": "quarterly"},
                {"action": "Migrate users and data", "priority": "high", "timeline": "quarterly"},
                {"action": "Decommission application", "priority": "high", "timeline": "annual"}
            ]),
        }
    }

    pub async fn bulk_calculate_time_assessments(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<TimeAssessment>> {
        // Get all application IDs for tenant
        let app_ids = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM applications WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut assessments = Vec::new();
        for app_id in app_ids {
            if let Ok(assessment) = self.calculate_time_assessment(tenant_id, app_id).await {
                assessments.push(assessment);
            }
        }

        Ok(assessments)
    }

    pub async fn list_time_assessments(&self, tenant_id: Uuid) -> Result<Vec<TimeAssessment>> {
        let assessments = sqlx::query_as::<_, TimeAssessment>(
            r#"
            SELECT id, tenant_id, application_id, quadrant, business_value_score,
                   technical_health_score, is_override, override_reason,
                   recommended_actions, assessed_by, assessed_at
            FROM time_assessments
            WHERE tenant_id = $1
            ORDER BY assessed_at DESC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(assessments)
    }

    pub async fn get_time_assessment(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<Option<TimeAssessment>> {
        let assessment = sqlx::query_as::<_, TimeAssessment>(
            r#"
            SELECT id, tenant_id, application_id, quadrant, business_value_score,
                   technical_health_score, is_override, override_reason,
                   recommended_actions, assessed_by, assessed_at
            FROM time_assessments
            WHERE tenant_id = $1 AND application_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(assessment)
    }

    pub async fn override_time_assessment(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
        quadrant: TimeQuadrant,
        reason: String,
    ) -> Result<Option<TimeAssessment>> {
        let assessment = sqlx::query_as::<_, TimeAssessment>(
            r#"
            UPDATE time_assessments SET
                quadrant = $1,
                is_override = true,
                override_reason = $2,
                assessed_at = NOW()
            WHERE tenant_id = $3 AND application_id = $4
            RETURNING id, tenant_id, application_id, quadrant, business_value_score,
                      technical_health_score, is_override, override_reason,
                      recommended_actions, assessed_by, assessed_at
            "#
        )
        .bind(quadrant.as_str())
        .bind(&reason)
        .bind(tenant_id)
        .bind(application_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(assessment)
    }

    pub async fn get_time_summary(&self, tenant_id: Uuid) -> Result<crate::api::TimeSummary> {
        let counts = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT quadrant, COUNT(*) as count
            FROM time_assessments
            WHERE tenant_id = $1
            GROUP BY quadrant
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut summary = crate::api::TimeSummary {
            tolerate: 0,
            invest: 0,
            migrate: 0,
            eliminate: 0,
            total_applications: 0,
        };

        for (quadrant, count) in counts {
            match quadrant.as_str() {
                "tolerate" => summary.tolerate = count,
                "invest" => summary.invest = count,
                "migrate" => summary.migrate = count,
                "eliminate" => summary.eliminate = count,
                _ => {}
            }
        }

        summary.total_applications = summary.tolerate + summary.invest + summary.migrate + summary.eliminate;

        Ok(summary)
    }

    // ========================================================================
    // Dependencies
    // ========================================================================

    pub async fn get_dependencies(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<Vec<ApplicationDependency>> {
        let deps = sqlx::query_as::<_, ApplicationDependency>(
            r#"
            SELECT id, tenant_id, source_application_id, target_application_id,
                   dependency_type, criticality, description, integration_id,
                   metadata, created_at
            FROM application_dependencies
            WHERE tenant_id = $1 AND (source_application_id = $2 OR target_application_id = $2)
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(deps)
    }

    pub async fn analyze_impact(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<ImpactAnalysis> {
        // Direct dependents (apps that depend on this one)
        let direct_dependents = sqlx::query_as::<_, ApplicationDependency>(
            r#"
            SELECT id, tenant_id, source_application_id, target_application_id,
                   dependency_type, criticality, description, integration_id,
                   metadata, created_at
            FROM application_dependencies
            WHERE tenant_id = $1 AND target_application_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;

        // Direct dependencies (apps this one depends on)
        let direct_dependencies = sqlx::query_as::<_, ApplicationDependency>(
            r#"
            SELECT id, tenant_id, source_application_id, target_application_id,
                   dependency_type, criticality, description, integration_id,
                   metadata, created_at
            FROM application_dependencies
            WHERE tenant_id = $1 AND source_application_id = $2
            "#
        )
        .bind(tenant_id)
        .bind(application_id)
        .fetch_all(&self.pool)
        .await?;

        let critical_dependencies = direct_dependents
            .iter()
            .filter(|d| d.criticality == "critical")
            .count() as i64;

        // Calculate transitive impact (simplified - would use recursive CTE in production)
        let transitive_impact_count = direct_dependents.len() as i64;

        let risk_level = if critical_dependencies > 2 || transitive_impact_count > 10 {
            "high"
        } else if critical_dependencies > 0 || transitive_impact_count > 5 {
            "medium"
        } else {
            "low"
        };

        Ok(ImpactAnalysis {
            application_id,
            direct_dependents,
            direct_dependencies,
            transitive_impact_count,
            critical_dependencies,
            risk_level: risk_level.to_string(),
        })
    }
}

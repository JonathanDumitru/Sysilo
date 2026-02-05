use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Recommendation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub application_id: Option<Uuid>,
    pub scenario_id: Option<Uuid>,
    pub recommendation_type: String,
    pub title: String,
    pub summary: String,
    pub detailed_analysis: Option<String>,
    pub confidence_score: Option<Decimal>,
    pub reasoning: Option<serde_json::Value>,
    pub supporting_data: Option<serde_json::Value>,
    pub estimated_savings: Option<Decimal>,
    pub estimated_effort: Option<String>,
    pub risk_assessment: Option<String>,
    pub status: String,
    pub user_feedback: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AIRecommendationRequest {
    pub tenant_id: Uuid,
    pub application_id: Option<Uuid>,
    pub scenario_id: Option<Uuid>,
    pub application_data: Option<ApplicationContext>,
    pub portfolio_context: PortfolioContext,
}

#[derive(Debug, Serialize)]
pub struct ApplicationContext {
    pub name: String,
    pub application_type: Option<String>,
    pub criticality: String,
    pub lifecycle_stage: String,
    pub total_cost: Option<Decimal>,
    pub scores: Vec<ScoreContext>,
    pub time_quadrant: Option<String>,
    pub dependencies_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ScoreContext {
    pub dimension: String,
    pub category: String,
    pub score: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PortfolioContext {
    pub total_applications: i32,
    pub quadrant_distribution: QuadrantDistribution,
    pub total_annual_cost: Decimal,
    pub avg_health_score: f64,
}

#[derive(Debug, Serialize)]
pub struct QuadrantDistribution {
    pub tolerate: i32,
    pub invest: i32,
    pub migrate: i32,
    pub eliminate: i32,
}

#[derive(Debug, Deserialize)]
pub struct AIRecommendationResponse {
    pub recommendations: Vec<GeneratedRecommendation>,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedRecommendation {
    pub recommendation_type: String,
    pub title: String,
    pub summary: String,
    pub detailed_analysis: String,
    pub confidence_score: f64,
    pub reasoning: serde_json::Value,
    pub estimated_savings: Option<f64>,
    pub estimated_effort: String,
    pub risk_assessment: String,
}

// ============================================================================
// Service
// ============================================================================

pub struct RecommendationsService {
    pool: PgPool,
    ai_service_url: String,
}

impl RecommendationsService {
    pub fn new(pool: PgPool, ai_service_url: String) -> Self {
        Self { pool, ai_service_url }
    }

    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<Recommendation>> {
        let recommendations = sqlx::query_as::<_, Recommendation>(
            r#"
            SELECT id, tenant_id, application_id, scenario_id, recommendation_type,
                   title, summary, detailed_analysis, confidence_score, reasoning,
                   supporting_data, estimated_savings, estimated_effort, risk_assessment,
                   status, user_feedback, generated_at, reviewed_by, reviewed_at
            FROM ai_recommendations
            WHERE tenant_id = $1
            ORDER BY generated_at DESC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(recommendations)
    }

    pub async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Recommendation>> {
        let recommendation = sqlx::query_as::<_, Recommendation>(
            r#"
            SELECT id, tenant_id, application_id, scenario_id, recommendation_type,
                   title, summary, detailed_analysis, confidence_score, reasoning,
                   supporting_data, estimated_savings, estimated_effort, risk_assessment,
                   status, user_feedback, generated_at, reviewed_by, reviewed_at
            FROM ai_recommendations
            WHERE id = $1 AND tenant_id = $2
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(recommendation)
    }

    pub async fn generate(
        &self,
        tenant_id: Uuid,
        application_id: Option<Uuid>,
        scenario_id: Option<Uuid>,
    ) -> Result<Vec<Recommendation>> {
        // Gather context for AI
        let portfolio_context = self.get_portfolio_context(tenant_id).await?;

        let application_data = if let Some(app_id) = application_id {
            Some(self.get_application_context(tenant_id, app_id).await?)
        } else {
            None
        };

        // For now, generate rule-based recommendations
        // In production, this would call the AI service
        let generated = self.generate_rule_based_recommendations(
            tenant_id,
            application_id,
            scenario_id,
            &application_data,
            &portfolio_context,
        ).await?;

        // Store recommendations
        let mut recommendations = Vec::new();
        for rec in generated {
            let saved = sqlx::query_as::<_, Recommendation>(
                r#"
                INSERT INTO ai_recommendations (
                    tenant_id, application_id, scenario_id, recommendation_type,
                    title, summary, detailed_analysis, confidence_score, reasoning,
                    supporting_data, estimated_savings, estimated_effort, risk_assessment
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id, tenant_id, application_id, scenario_id, recommendation_type,
                          title, summary, detailed_analysis, confidence_score, reasoning,
                          supporting_data, estimated_savings, estimated_effort, risk_assessment,
                          status, user_feedback, generated_at, reviewed_by, reviewed_at
                "#
            )
            .bind(tenant_id)
            .bind(application_id)
            .bind(scenario_id)
            .bind(&rec.recommendation_type)
            .bind(&rec.title)
            .bind(&rec.summary)
            .bind(&rec.detailed_analysis)
            .bind(Decimal::from_f64_retain(rec.confidence_score))
            .bind(&rec.reasoning)
            .bind(serde_json::json!({}))
            .bind(rec.estimated_savings.map(Decimal::from_f64_retain))
            .bind(&rec.estimated_effort)
            .bind(&rec.risk_assessment)
            .fetch_one(&self.pool)
            .await?;

            recommendations.push(saved);
        }

        Ok(recommendations)
    }

    pub async fn update_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        status: String,
        feedback: Option<String>,
    ) -> Result<Option<Recommendation>> {
        let recommendation = sqlx::query_as::<_, Recommendation>(
            r#"
            UPDATE ai_recommendations SET
                status = $1,
                user_feedback = $2,
                reviewed_at = NOW()
            WHERE id = $3 AND tenant_id = $4
            RETURNING id, tenant_id, application_id, scenario_id, recommendation_type,
                      title, summary, detailed_analysis, confidence_score, reasoning,
                      supporting_data, estimated_savings, estimated_effort, risk_assessment,
                      status, user_feedback, generated_at, reviewed_by, reviewed_at
            "#
        )
        .bind(&status)
        .bind(&feedback)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(recommendation)
    }

    async fn get_portfolio_context(&self, tenant_id: Uuid) -> Result<PortfolioContext> {
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM applications WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let total_cost = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT SUM(total_cost) FROM applications WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?
        .unwrap_or_default();

        // Get quadrant distribution
        let quadrants = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT quadrant, COUNT(*) FROM time_assessments
            WHERE tenant_id = $1 GROUP BY quadrant
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut distribution = QuadrantDistribution {
            tolerate: 0,
            invest: 0,
            migrate: 0,
            eliminate: 0,
        };

        for (q, count) in quadrants {
            match q.as_str() {
                "tolerate" => distribution.tolerate = count as i32,
                "invest" => distribution.invest = count as i32,
                "migrate" => distribution.migrate = count as i32,
                "eliminate" => distribution.eliminate = count as i32,
                _ => {}
            }
        }

        Ok(PortfolioContext {
            total_applications: total as i32,
            quadrant_distribution: distribution,
            total_annual_cost: total_cost,
            avg_health_score: 6.5, // Would calculate from actual scores
        })
    }

    async fn get_application_context(
        &self,
        tenant_id: Uuid,
        application_id: Uuid,
    ) -> Result<ApplicationContext> {
        let app = sqlx::query_as::<_, (String, Option<String>, String, String, Option<Decimal>)>(
            r#"
            SELECT name, application_type, criticality, lifecycle_stage, total_cost
            FROM applications WHERE id = $1 AND tenant_id = $2
            "#
        )
        .bind(application_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let scores = sqlx::query_as::<_, (String, String, Decimal)>(
            r#"
            SELECT sd.name, sd.category, s.score
            FROM application_scores s
            JOIN scoring_dimensions sd ON sd.id = s.dimension_id
            WHERE s.application_id = $1 AND s.tenant_id = $2
            "#
        )
        .bind(application_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let time_quadrant = sqlx::query_scalar::<_, Option<String>>(
            "SELECT quadrant FROM time_assessments WHERE application_id = $1 AND tenant_id = $2"
        )
        .bind(application_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        let deps_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM application_dependencies
            WHERE (source_application_id = $1 OR target_application_id = $1) AND tenant_id = $2
            "#
        )
        .bind(application_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ApplicationContext {
            name: app.0,
            application_type: app.1,
            criticality: app.2,
            lifecycle_stage: app.3,
            total_cost: app.4,
            scores: scores.into_iter().map(|(dim, cat, score)| ScoreContext {
                dimension: dim,
                category: cat,
                score,
            }).collect(),
            time_quadrant,
            dependencies_count: deps_count as i32,
        })
    }

    async fn generate_rule_based_recommendations(
        &self,
        _tenant_id: Uuid,
        application_id: Option<Uuid>,
        _scenario_id: Option<Uuid>,
        application_data: &Option<ApplicationContext>,
        portfolio_context: &PortfolioContext,
    ) -> Result<Vec<GeneratedRecommendation>> {
        let mut recommendations = Vec::new();

        // Application-specific recommendations
        if let Some(app) = application_data {
            // Check TIME quadrant
            if let Some(ref quadrant) = app.time_quadrant {
                match quadrant.as_str() {
                    "eliminate" => {
                        recommendations.push(GeneratedRecommendation {
                            recommendation_type: "retirement".to_string(),
                            title: format!("Consider retiring {}", app.name),
                            summary: "This application has low business value and poor technical health. Consider decommissioning.".to_string(),
                            detailed_analysis: format!(
                                "Application '{}' is in the ELIMINATE quadrant, indicating both low business value \
                                and poor technical health. With {} dependencies, a retirement plan should account for \
                                migration paths for dependent systems.",
                                app.name, app.dependencies_count
                            ),
                            confidence_score: 0.85,
                            reasoning: serde_json::json!({
                                "quadrant": "eliminate",
                                "dependencies": app.dependencies_count,
                                "lifecycle_stage": app.lifecycle_stage
                            }),
                            estimated_savings: app.total_cost.map(|c| c.to_string().parse().unwrap_or(0.0)),
                            estimated_effort: if app.dependencies_count > 5 { "high" } else { "medium" }.to_string(),
                            risk_assessment: if app.dependencies_count > 5 { "high" } else { "medium" }.to_string(),
                        });
                    }
                    "migrate" => {
                        recommendations.push(GeneratedRecommendation {
                            recommendation_type: "migration".to_string(),
                            title: format!("Modernize or replace {}", app.name),
                            summary: "High business value but poor technical health suggests modernization is needed.".to_string(),
                            detailed_analysis: format!(
                                "Application '{}' provides significant business value but has technical challenges. \
                                Consider a phased modernization approach to maintain business continuity while improving \
                                the technical foundation.",
                                app.name
                            ),
                            confidence_score: 0.80,
                            reasoning: serde_json::json!({
                                "quadrant": "migrate",
                                "criticality": app.criticality
                            }),
                            estimated_savings: Some(app.total_cost.map(|c| c.to_string().parse::<f64>().unwrap_or(0.0) * 0.2).unwrap_or(0.0)),
                            estimated_effort: "high".to_string(),
                            risk_assessment: "medium".to_string(),
                        });
                    }
                    _ => {}
                }
            }

            // High cost applications
            if let Some(cost) = app.total_cost {
                if cost > Decimal::new(100000, 0) {
                    recommendations.push(GeneratedRecommendation {
                        recommendation_type: "optimization".to_string(),
                        title: format!("Cost optimization opportunity for {}", app.name),
                        summary: "High-cost application may benefit from infrastructure optimization.".to_string(),
                        detailed_analysis: format!(
                            "At ${} annual cost, '{}' represents a significant investment. \
                            Consider reviewing resource utilization, licensing optimization, \
                            or cloud migration opportunities.",
                            cost, app.name
                        ),
                        confidence_score: 0.70,
                        reasoning: serde_json::json!({
                            "annual_cost": cost.to_string(),
                            "threshold": "100000"
                        }),
                        estimated_savings: Some(cost.to_string().parse::<f64>().unwrap_or(0.0) * 0.15),
                        estimated_effort: "low".to_string(),
                        risk_assessment: "low".to_string(),
                    });
                }
            }
        }

        // Portfolio-wide recommendations
        if portfolio_context.quadrant_distribution.eliminate > 3 {
            recommendations.push(GeneratedRecommendation {
                recommendation_type: "consolidation".to_string(),
                title: "Application rationalization opportunity".to_string(),
                summary: format!(
                    "{} applications are in the ELIMINATE quadrant. Consider a portfolio rationalization initiative.",
                    portfolio_context.quadrant_distribution.eliminate
                ),
                detailed_analysis: format!(
                    "Your portfolio has {} applications marked for elimination. Consolidating these through \
                    a structured rationalization program could significantly reduce costs and complexity.",
                    portfolio_context.quadrant_distribution.eliminate
                ),
                confidence_score: 0.75,
                reasoning: serde_json::json!({
                    "eliminate_count": portfolio_context.quadrant_distribution.eliminate,
                    "total_applications": portfolio_context.total_applications
                }),
                estimated_savings: Some(portfolio_context.total_annual_cost.to_string().parse::<f64>().unwrap_or(0.0) * 0.1),
                estimated_effort: "high".to_string(),
                risk_assessment: "medium".to_string(),
            });
        }

        Ok(recommendations)
    }
}

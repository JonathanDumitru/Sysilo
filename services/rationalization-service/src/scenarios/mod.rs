use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Scenario {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: String,
    pub status: String,
    pub affected_applications: Vec<Uuid>,
    pub assumptions: Option<serde_json::Value>,
    pub current_state: Option<serde_json::Value>,
    pub projected_state: Option<serde_json::Value>,
    pub implementation_cost: Option<Decimal>,
    pub annual_savings: Option<Decimal>,
    pub payback_months: Option<i32>,
    pub npv: Option<Decimal>,
    pub roi_percent: Option<Decimal>,
    pub risk_level: Option<String>,
    pub risk_factors: Option<serde_json::Value>,
    pub estimated_duration_months: Option<i32>,
    pub proposed_start_date: Option<NaiveDate>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: String,
    pub affected_applications: Vec<Uuid>,
    pub assumptions: Option<serde_json::Value>,
    pub implementation_cost: Option<Decimal>,
    pub estimated_duration_months: Option<i32>,
    pub proposed_start_date: Option<NaiveDate>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScenarioRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub affected_applications: Option<Vec<Uuid>>,
    pub assumptions: Option<serde_json::Value>,
    pub implementation_cost: Option<Decimal>,
    pub annual_savings: Option<Decimal>,
    pub risk_level: Option<String>,
    pub risk_factors: Option<serde_json::Value>,
    pub estimated_duration_months: Option<i32>,
    pub proposed_start_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioAnalysis {
    pub scenario_id: Uuid,
    pub current_state: CurrentState,
    pub projected_state: ProjectedState,
    pub financial_analysis: FinancialAnalysis,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Serialize)]
pub struct CurrentState {
    pub total_applications: i32,
    pub total_annual_cost: Decimal,
    pub avg_health_score: f64,
    pub avg_value_score: f64,
    pub integration_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ProjectedState {
    pub total_applications: i32,
    pub total_annual_cost: Decimal,
    pub estimated_health_improvement: f64,
    pub estimated_value_change: f64,
    pub reduced_integrations: i32,
}

#[derive(Debug, Serialize)]
pub struct FinancialAnalysis {
    pub implementation_cost: Decimal,
    pub annual_savings: Decimal,
    pub payback_months: i32,
    pub five_year_npv: Decimal,
    pub roi_percent: Decimal,
}

#[derive(Debug, Serialize)]
pub struct RiskAssessment {
    pub overall_risk_level: String,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RiskFactor {
    pub factor: String,
    pub impact: String,
    pub likelihood: String,
    pub mitigation: String,
}

#[derive(Debug, Serialize)]
pub struct ScenarioComparison {
    pub scenarios: Vec<ScenarioSummary>,
    pub metrics_comparison: MetricsComparison,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct ScenarioSummary {
    pub id: Uuid,
    pub name: String,
    pub scenario_type: String,
    pub implementation_cost: Decimal,
    pub annual_savings: Decimal,
    pub roi_percent: Decimal,
    pub risk_level: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsComparison {
    pub cost_comparison: Vec<ComparisonMetric>,
    pub savings_comparison: Vec<ComparisonMetric>,
    pub risk_comparison: Vec<ComparisonMetric>,
}

#[derive(Debug, Serialize)]
pub struct ComparisonMetric {
    pub scenario_id: Uuid,
    pub value: String,
    pub rank: i32,
}

// ============================================================================
// Service
// ============================================================================

pub struct ScenariosService {
    pool: PgPool,
}

impl ScenariosService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<Scenario>> {
        let scenarios = sqlx::query_as::<_, Scenario>(
            r#"
            SELECT id, tenant_id, name, description, scenario_type, status,
                   affected_applications, assumptions, current_state, projected_state,
                   implementation_cost, annual_savings, payback_months, npv, roi_percent,
                   risk_level, risk_factors, estimated_duration_months, proposed_start_date,
                   created_by, created_at, updated_at
            FROM rationalization_scenarios
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(scenarios)
    }

    pub async fn create(&self, tenant_id: Uuid, req: CreateScenarioRequest) -> Result<Scenario> {
        let scenario = sqlx::query_as::<_, Scenario>(
            r#"
            INSERT INTO rationalization_scenarios (
                tenant_id, name, description, scenario_type, affected_applications,
                assumptions, implementation_cost, estimated_duration_months,
                proposed_start_date, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tenant_id, name, description, scenario_type, status,
                      affected_applications, assumptions, current_state, projected_state,
                      implementation_cost, annual_savings, payback_months, npv, roi_percent,
                      risk_level, risk_factors, estimated_duration_months, proposed_start_date,
                      created_by, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.scenario_type)
        .bind(&req.affected_applications)
        .bind(&req.assumptions)
        .bind(req.implementation_cost)
        .bind(req.estimated_duration_months)
        .bind(req.proposed_start_date)
        .bind(req.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(scenario)
    }

    pub async fn get(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Scenario>> {
        let scenario = sqlx::query_as::<_, Scenario>(
            r#"
            SELECT id, tenant_id, name, description, scenario_type, status,
                   affected_applications, assumptions, current_state, projected_state,
                   implementation_cost, annual_savings, payback_months, npv, roi_percent,
                   risk_level, risk_factors, estimated_duration_months, proposed_start_date,
                   created_by, created_at, updated_at
            FROM rationalization_scenarios
            WHERE id = $1 AND tenant_id = $2
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(scenario)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateScenarioRequest,
    ) -> Result<Option<Scenario>> {
        let scenario = sqlx::query_as::<_, Scenario>(
            r#"
            UPDATE rationalization_scenarios SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                status = COALESCE($3, status),
                affected_applications = COALESCE($4, affected_applications),
                assumptions = COALESCE($5, assumptions),
                implementation_cost = COALESCE($6, implementation_cost),
                annual_savings = COALESCE($7, annual_savings),
                risk_level = COALESCE($8, risk_level),
                risk_factors = COALESCE($9, risk_factors),
                estimated_duration_months = COALESCE($10, estimated_duration_months),
                proposed_start_date = COALESCE($11, proposed_start_date),
                updated_at = NOW()
            WHERE id = $12 AND tenant_id = $13
            RETURNING id, tenant_id, name, description, scenario_type, status,
                      affected_applications, assumptions, current_state, projected_state,
                      implementation_cost, annual_savings, payback_months, npv, roi_percent,
                      risk_level, risk_factors, estimated_duration_months, proposed_start_date,
                      created_by, created_at, updated_at
            "#
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.status)
        .bind(&req.affected_applications)
        .bind(&req.assumptions)
        .bind(req.implementation_cost)
        .bind(req.annual_savings)
        .bind(&req.risk_level)
        .bind(&req.risk_factors)
        .bind(req.estimated_duration_months)
        .bind(req.proposed_start_date)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(scenario)
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM rationalization_scenarios WHERE id = $1 AND tenant_id = $2"
        )
        .bind(id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn analyze(&self, tenant_id: Uuid, id: Uuid) -> Result<ScenarioAnalysis> {
        let scenario = self.get(tenant_id, id).await?
            .ok_or_else(|| anyhow::anyhow!("Scenario not found"))?;

        // Calculate current state from affected applications
        let current_costs = sqlx::query_scalar::<_, Option<Decimal>>(
            r#"
            SELECT SUM(total_cost)
            FROM applications
            WHERE tenant_id = $1 AND id = ANY($2)
            "#
        )
        .bind(tenant_id)
        .bind(&scenario.affected_applications)
        .fetch_one(&self.pool)
        .await?
        .unwrap_or_default();

        let app_count = scenario.affected_applications.len() as i32;

        let current_state = CurrentState {
            total_applications: app_count,
            total_annual_cost: current_costs,
            avg_health_score: 6.5, // Would calculate from actual scores
            avg_value_score: 5.8,
            integration_count: app_count * 2, // Estimate
        };

        // Project future state based on scenario type
        let (cost_reduction, health_improvement) = match scenario.scenario_type.as_str() {
            "consolidation" => (Decimal::new(30, 2), 1.5),  // 30% cost reduction
            "migration" => (Decimal::new(20, 2), 2.0),
            "modernization" => (Decimal::new(10, 2), 3.0),
            "retirement" => (Decimal::new(100, 2), 0.0),  // Full cost elimination
            _ => (Decimal::new(15, 2), 1.0),
        };

        let annual_savings = current_costs * cost_reduction / Decimal::new(100, 0);
        let projected_cost = current_costs - annual_savings;

        let projected_state = ProjectedState {
            total_applications: if scenario.scenario_type == "retirement" { 0 } else { app_count },
            total_annual_cost: projected_cost,
            estimated_health_improvement: health_improvement,
            estimated_value_change: 0.5,
            reduced_integrations: app_count,
        };

        // Financial analysis
        let implementation_cost = scenario.implementation_cost.unwrap_or(Decimal::ZERO);
        let payback_months = if annual_savings > Decimal::ZERO {
            ((implementation_cost / annual_savings) * Decimal::new(12, 0)).to_string().parse::<i32>().unwrap_or(0)
        } else {
            0
        };

        // Simple NPV calculation (5 years, 10% discount rate)
        let discount_rate = Decimal::new(10, 2);
        let mut npv = -implementation_cost;
        let mut compound = Decimal::ONE;
        for _year in 1..=5 {
            compound *= Decimal::ONE + discount_rate;
            let discount_factor = Decimal::ONE / compound;
            npv += annual_savings * discount_factor;
        }

        let roi_percent = if implementation_cost > Decimal::ZERO {
            (annual_savings * Decimal::new(5, 0) - implementation_cost) / implementation_cost * Decimal::new(100, 0)
        } else {
            Decimal::ZERO
        };

        let financial_analysis = FinancialAnalysis {
            implementation_cost,
            annual_savings,
            payback_months,
            five_year_npv: npv,
            roi_percent,
        };

        // Risk assessment
        let risk_factors = vec![
            RiskFactor {
                factor: "Technical complexity".to_string(),
                impact: "high".to_string(),
                likelihood: "medium".to_string(),
                mitigation: "Detailed technical assessment and phased approach".to_string(),
            },
            RiskFactor {
                factor: "Business disruption".to_string(),
                impact: "high".to_string(),
                likelihood: "low".to_string(),
                mitigation: "Parallel running and comprehensive testing".to_string(),
            },
            RiskFactor {
                factor: "Resource constraints".to_string(),
                impact: "medium".to_string(),
                likelihood: "medium".to_string(),
                mitigation: "Early resource planning and skill assessment".to_string(),
            },
        ];

        let risk_assessment = RiskAssessment {
            overall_risk_level: scenario.risk_level.clone().unwrap_or_else(|| "medium".to_string()),
            risk_factors,
            mitigation_strategies: vec![
                "Establish governance committee".to_string(),
                "Create rollback plan".to_string(),
                "Implement monitoring dashboards".to_string(),
            ],
        };

        // Update scenario with analysis results
        sqlx::query(
            r#"
            UPDATE rationalization_scenarios SET
                current_state = $1,
                projected_state = $2,
                annual_savings = $3,
                payback_months = $4,
                npv = $5,
                roi_percent = $6,
                status = 'completed',
                updated_at = NOW()
            WHERE id = $7 AND tenant_id = $8
            "#
        )
        .bind(serde_json::to_value(&current_state)?)
        .bind(serde_json::to_value(&projected_state)?)
        .bind(annual_savings)
        .bind(payback_months)
        .bind(npv)
        .bind(roi_percent)
        .bind(id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(ScenarioAnalysis {
            scenario_id: id,
            current_state,
            projected_state,
            financial_analysis,
            risk_assessment,
        })
    }

    pub async fn compare(&self, tenant_id: Uuid, scenario_ids: Vec<Uuid>) -> Result<ScenarioComparison> {
        let mut scenarios = Vec::new();
        let mut cost_comparison = Vec::new();
        let mut savings_comparison = Vec::new();
        let mut risk_comparison = Vec::new();

        for (idx, id) in scenario_ids.iter().enumerate() {
            if let Some(scenario) = self.get(tenant_id, *id).await? {
                let impl_cost = scenario.implementation_cost.unwrap_or_default();
                let savings = scenario.annual_savings.unwrap_or_default();
                let roi = scenario.roi_percent.unwrap_or_default();

                scenarios.push(ScenarioSummary {
                    id: scenario.id,
                    name: scenario.name.clone(),
                    scenario_type: scenario.scenario_type.clone(),
                    implementation_cost: impl_cost,
                    annual_savings: savings,
                    roi_percent: roi,
                    risk_level: scenario.risk_level.clone().unwrap_or_else(|| "medium".to_string()),
                });

                cost_comparison.push(ComparisonMetric {
                    scenario_id: *id,
                    value: impl_cost.to_string(),
                    rank: (idx + 1) as i32,
                });

                savings_comparison.push(ComparisonMetric {
                    scenario_id: *id,
                    value: savings.to_string(),
                    rank: (idx + 1) as i32,
                });

                risk_comparison.push(ComparisonMetric {
                    scenario_id: *id,
                    value: scenario.risk_level.unwrap_or_else(|| "medium".to_string()),
                    rank: (idx + 1) as i32,
                });
            }
        }

        // Sort by ROI to determine recommendation
        scenarios.sort_by(|a, b| b.roi_percent.partial_cmp(&a.roi_percent).unwrap());

        let recommendation = if let Some(best) = scenarios.first() {
            format!(
                "Recommended: {} - Highest ROI at {}% with {} risk",
                best.name, best.roi_percent, best.risk_level
            )
        } else {
            "No scenarios available for comparison".to_string()
        };

        Ok(ScenarioComparison {
            scenarios,
            metrics_comparison: MetricsComparison {
                cost_comparison,
                savings_comparison,
                risk_comparison,
            },
            recommendation,
        })
    }
}

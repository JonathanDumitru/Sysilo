pub mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// =============================================================================
// Types
// =============================================================================

/// Industry vertical for template categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IndustryVertical {
    Healthcare,
    Finance,
    Manufacturing,
    RetailEcommerce,
    Government,
    Education,
    Telecommunications,
    Energy,
    Logistics,
    Custom,
}

impl IndustryVertical {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthcare => "healthcare",
            Self::Finance => "finance",
            Self::Manufacturing => "manufacturing",
            Self::RetailEcommerce => "retail_ecommerce",
            Self::Government => "government",
            Self::Education => "education",
            Self::Telecommunications => "telecommunications",
            Self::Energy => "energy",
            Self::Logistics => "logistics",
            Self::Custom => "custom",
        }
    }
}

/// A pre-built industry solution template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndustryTemplate {
    pub id: Uuid,
    pub name: String,
    pub vertical: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon_url: Option<String>,
    /// JSON array of connector IDs this template requires
    pub required_connectors: serde_json::Value,
    /// JSON object of governance policies to deploy
    pub governance_policies: serde_json::Value,
    /// JSON array of playbook definitions
    pub playbooks: serde_json::Value,
    /// JSON object of data model / schema definitions
    pub data_models: serde_json::Value,
    /// JSON object of dashboard layout configurations
    pub dashboard_layouts: serde_json::Value,
    /// JSON object of compliance framework mappings
    pub compliance_mappings: serde_json::Value,
    /// JSON array of tags for discoverability
    pub tags: serde_json::Value,
    pub status: String,
    pub install_count: i64,
    pub avg_rating: f64,
    pub rating_count: i64,
    pub pricing_tier: String,
    pub price: Option<f64>,
    pub estimated_setup_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// A template deployment instance
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateDeployment {
    pub id: Uuid,
    pub template_id: Uuid,
    pub tenant_id: Uuid,
    pub deployed_by: Uuid,
    pub status: String,
    /// JSON object tracking which components were deployed
    pub deployed_components: serde_json::Value,
    /// JSON object of tenant-specific customizations
    pub customizations: serde_json::Value,
    pub deployed_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub rollback_at: Option<DateTime<Utc>>,
}

/// Request to create a new template
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub vertical: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub required_connectors: serde_json::Value,
    pub governance_policies: serde_json::Value,
    pub playbooks: serde_json::Value,
    pub data_models: serde_json::Value,
    pub dashboard_layouts: serde_json::Value,
    pub compliance_mappings: serde_json::Value,
    pub tags: serde_json::Value,
    pub pricing_tier: Option<String>,
    pub price: Option<f64>,
    pub estimated_setup_minutes: Option<i32>,
}

/// Request to deploy a template
#[derive(Debug, Clone, Deserialize)]
pub struct DeployTemplateRequest {
    pub template_id: Uuid,
    pub customizations: Option<serde_json::Value>,
}

/// Template deployment progress
#[derive(Debug, Clone, Serialize)]
pub struct DeploymentProgress {
    pub deployment_id: Uuid,
    pub status: String,
    pub total_steps: i32,
    pub completed_steps: i32,
    pub current_step: String,
    pub deployed_components: serde_json::Value,
}

/// Pre-built template catalog entry (for seeding)
#[derive(Debug, Clone, Serialize)]
pub struct TemplateCatalogEntry {
    pub vertical: IndustryVertical,
    pub name: String,
    pub description: String,
    pub key_connectors: Vec<String>,
    pub compliance_frameworks: Vec<String>,
    pub estimated_setup_minutes: i32,
}

// =============================================================================
// Built-in Template Catalog
// =============================================================================

pub fn builtin_template_catalog() -> Vec<TemplateCatalogEntry> {
    vec![
        TemplateCatalogEntry {
            vertical: IndustryVertical::Healthcare,
            name: "Healthcare HIPAA Compliance Suite".to_string(),
            description: "HIPAA governance policies, patient data lineage, HL7/FHIR integration playbooks, PHI tagging rules, and clinical data quality monitoring".to_string(),
            key_connectors: vec!["epic".into(), "cerner".into(), "hl7_fhir".into(), "lab_systems".into()],
            compliance_frameworks: vec!["HIPAA".into(), "HITECH".into()],
            estimated_setup_minutes: 45,
        },
        TemplateCatalogEntry {
            vertical: IndustryVertical::Finance,
            name: "Financial Services SOX & Risk Suite".to_string(),
            description: "SOX compliance workflows, fraud signal routing, risk engine integration, audit-ready dashboards, and trade surveillance pipelines".to_string(),
            key_connectors: vec!["bloomberg".into(), "plaid".into(), "core_banking".into(), "swift".into()],
            compliance_frameworks: vec!["SOX".into(), "PCI_DSS".into(), "GDPR".into()],
            estimated_setup_minutes: 60,
        },
        TemplateCatalogEntry {
            vertical: IndustryVertical::Manufacturing,
            name: "Manufacturing IoT & Supply Chain Suite".to_string(),
            description: "IoT sensor ingestion pipelines, supply chain DAGs, quality control automation, ERP sync, and predictive maintenance models".to_string(),
            key_connectors: vec!["sap".into(), "siemens_mindsphere".into(), "opc_ua".into(), "mqtt".into()],
            compliance_frameworks: vec!["ISO_9001".into(), "ISO_27001".into()],
            estimated_setup_minutes: 50,
        },
        TemplateCatalogEntry {
            vertical: IndustryVertical::RetailEcommerce,
            name: "Retail & E-commerce Customer 360 Suite".to_string(),
            description: "Inventory sync, order routing, marketplace connectors, customer 360 data model, and recommendation engine pipelines".to_string(),
            key_connectors: vec!["shopify".into(), "amazon_sp".into(), "salesforce_commerce".into(), "stripe".into()],
            compliance_frameworks: vec!["PCI_DSS".into(), "GDPR".into(), "CCPA".into()],
            estimated_setup_minutes: 40,
        },
        TemplateCatalogEntry {
            vertical: IndustryVertical::Government,
            name: "Government FedRAMP & Data Residency Suite".to_string(),
            description: "FedRAMP-aligned governance, data residency enforcement, citizen data masking, FOIA response automation, and legacy system connectors".to_string(),
            key_connectors: vec!["mainframe_cics".into(), "sftp".into(), "govcloud_s3".into(), "active_directory".into()],
            compliance_frameworks: vec!["FedRAMP".into(), "FISMA".into(), "NIST_800_53".into()],
            estimated_setup_minutes: 90,
        },
    ]
}

// =============================================================================
// Service
// =============================================================================

pub struct TemplatesService {
    pool: PgPool,
}

impl TemplatesService {
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
    // Template CRUD
    // =========================================================================

    pub async fn list_templates(
        &self,
        vertical: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<IndustryTemplate>, i64)> {
        let mut query = String::from(
            "SELECT id, name, vertical, description, version, author, icon_url, \
             required_connectors, governance_policies, playbooks, data_models, \
             dashboard_layouts, compliance_mappings, tags, status, install_count, \
             avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
             created_at, updated_at, published_at \
             FROM industry_templates WHERE 1=1"
        );
        let mut count_query = String::from("SELECT COUNT(*) FROM industry_templates WHERE 1=1");

        if let Some(v) = vertical {
            let clause = format!(" AND vertical = '{}'", v.replace('\'', "''"));
            query.push_str(&clause);
            count_query.push_str(&clause);
        }
        if let Some(s) = status {
            let clause = format!(" AND status = '{}'", s.replace('\'', "''"));
            query.push_str(&clause);
            count_query.push_str(&clause);
        }
        if let Some(q) = search {
            let escaped = q.replace('\'', "''").to_lowercase();
            let clause = format!(
                " AND (LOWER(name) LIKE '%{}%' OR LOWER(description) LIKE '%{}%')",
                escaped, escaped
            );
            query.push_str(&clause);
            count_query.push_str(&clause);
        }

        query.push_str(&format!(" ORDER BY install_count DESC LIMIT {} OFFSET {}", limit, offset));

        let templates = sqlx::query_as::<_, IndustryTemplate>(&query)
            .fetch_all(&self.pool)
            .await?;

        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await?;

        Ok((templates, total.0))
    }

    pub async fn get_template(&self, id: Uuid) -> Result<Option<IndustryTemplate>> {
        let template = sqlx::query_as::<_, IndustryTemplate>(
            "SELECT id, name, vertical, description, version, author, icon_url, \
             required_connectors, governance_policies, playbooks, data_models, \
             dashboard_layouts, compliance_mappings, tags, status, install_count, \
             avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
             created_at, updated_at, published_at \
             FROM industry_templates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(template)
    }

    pub async fn create_template(&self, req: CreateTemplateRequest) -> Result<IndustryTemplate> {
        let template = sqlx::query_as::<_, IndustryTemplate>(
            "INSERT INTO industry_templates \
             (name, vertical, description, version, author, icon_url, \
              required_connectors, governance_policies, playbooks, data_models, \
              dashboard_layouts, compliance_mappings, tags, pricing_tier, price, \
              estimated_setup_minutes, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, 'draft') \
             RETURNING id, name, vertical, description, version, author, icon_url, \
              required_connectors, governance_policies, playbooks, data_models, \
              dashboard_layouts, compliance_mappings, tags, status, install_count, \
              avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
              created_at, updated_at, published_at"
        )
        .bind(&req.name)
        .bind(&req.vertical)
        .bind(&req.description)
        .bind(&req.version)
        .bind(&req.author)
        .bind(&req.icon_url)
        .bind(&req.required_connectors)
        .bind(&req.governance_policies)
        .bind(&req.playbooks)
        .bind(&req.data_models)
        .bind(&req.dashboard_layouts)
        .bind(&req.compliance_mappings)
        .bind(&req.tags)
        .bind(req.pricing_tier.unwrap_or_else(|| "free".to_string()))
        .bind(req.price)
        .bind(req.estimated_setup_minutes.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        Ok(template)
    }

    pub async fn publish_template(&self, id: Uuid) -> Result<IndustryTemplate> {
        let template = sqlx::query_as::<_, IndustryTemplate>(
            "UPDATE industry_templates SET status = 'published', published_at = NOW(), updated_at = NOW() \
             WHERE id = $1 \
             RETURNING id, name, vertical, description, version, author, icon_url, \
              required_connectors, governance_policies, playbooks, data_models, \
              dashboard_layouts, compliance_mappings, tags, status, install_count, \
              avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
              created_at, updated_at, published_at"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(template)
    }

    // =========================================================================
    // Template Deployment
    // =========================================================================

    pub async fn deploy_template(
        &self,
        tenant_id: Uuid,
        deployed_by: Uuid,
        req: DeployTemplateRequest,
    ) -> Result<TemplateDeployment> {
        // Verify template exists and is published
        let template = self.get_template(req.template_id).await?
            .ok_or_else(|| anyhow::anyhow!("Template not found"))?;

        if template.status != "published" {
            anyhow::bail!("Template is not published");
        }

        // Create deployment record
        let customizations = req.customizations.unwrap_or(serde_json::json!({}));
        let initial_components = serde_json::json!({
            "connectors": { "status": "pending" },
            "governance_policies": { "status": "pending" },
            "playbooks": { "status": "pending" },
            "data_models": { "status": "pending" },
            "dashboards": { "status": "pending" },
            "compliance": { "status": "pending" }
        });

        let deployment = sqlx::query_as::<_, TemplateDeployment>(
            "INSERT INTO template_deployments \
             (template_id, tenant_id, deployed_by, status, deployed_components, customizations) \
             VALUES ($1, $2, $3, 'deploying', $4, $5) \
             RETURNING id, template_id, tenant_id, deployed_by, status, \
              deployed_components, customizations, deployed_at, completed_at, rollback_at"
        )
        .bind(req.template_id)
        .bind(tenant_id)
        .bind(deployed_by)
        .bind(&initial_components)
        .bind(&customizations)
        .fetch_one(&self.pool)
        .await?;

        // Increment install count
        sqlx::query("UPDATE industry_templates SET install_count = install_count + 1 WHERE id = $1")
            .bind(req.template_id)
            .execute(&self.pool)
            .await?;

        Ok(deployment)
    }

    pub async fn get_deployment(&self, deployment_id: Uuid) -> Result<Option<TemplateDeployment>> {
        let deployment = sqlx::query_as::<_, TemplateDeployment>(
            "SELECT id, template_id, tenant_id, deployed_by, status, \
             deployed_components, customizations, deployed_at, completed_at, rollback_at \
             FROM template_deployments WHERE id = $1"
        )
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    pub async fn list_deployments(&self, tenant_id: Uuid) -> Result<Vec<TemplateDeployment>> {
        let deployments = sqlx::query_as::<_, TemplateDeployment>(
            "SELECT id, template_id, tenant_id, deployed_by, status, \
             deployed_components, customizations, deployed_at, completed_at, rollback_at \
             FROM template_deployments WHERE tenant_id = $1 ORDER BY deployed_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(deployments)
    }

    pub async fn complete_deployment(&self, deployment_id: Uuid) -> Result<TemplateDeployment> {
        let deployment = sqlx::query_as::<_, TemplateDeployment>(
            "UPDATE template_deployments SET status = 'completed', completed_at = NOW() \
             WHERE id = $1 \
             RETURNING id, template_id, tenant_id, deployed_by, status, \
              deployed_components, customizations, deployed_at, completed_at, rollback_at"
        )
        .bind(deployment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(deployment)
    }

    pub async fn rollback_deployment(&self, deployment_id: Uuid) -> Result<TemplateDeployment> {
        let deployment = sqlx::query_as::<_, TemplateDeployment>(
            "UPDATE template_deployments SET status = 'rolled_back', rollback_at = NOW() \
             WHERE id = $1 \
             RETURNING id, template_id, tenant_id, deployed_by, status, \
              deployed_components, customizations, deployed_at, completed_at, rollback_at"
        )
        .bind(deployment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(deployment)
    }

    pub async fn get_templates_by_vertical(&self, vertical: &str) -> Result<Vec<IndustryTemplate>> {
        let templates = sqlx::query_as::<_, IndustryTemplate>(
            "SELECT id, name, vertical, description, version, author, icon_url, \
             required_connectors, governance_policies, playbooks, data_models, \
             dashboard_layouts, compliance_mappings, tags, status, install_count, \
             avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
             created_at, updated_at, published_at \
             FROM industry_templates WHERE vertical = $1 AND status = 'published' \
             ORDER BY install_count DESC"
        )
        .bind(vertical)
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }

    pub async fn get_featured_templates(&self, limit: i64) -> Result<Vec<IndustryTemplate>> {
        let templates = sqlx::query_as::<_, IndustryTemplate>(
            "SELECT id, name, vertical, description, version, author, icon_url, \
             required_connectors, governance_policies, playbooks, data_models, \
             dashboard_layouts, compliance_mappings, tags, status, install_count, \
             avg_rating, rating_count, pricing_tier, price, estimated_setup_minutes, \
             created_at, updated_at, published_at \
             FROM industry_templates WHERE status = 'published' \
             ORDER BY (avg_rating * LN(rating_count + 1) + LN(install_count + 1)) DESC \
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }
}

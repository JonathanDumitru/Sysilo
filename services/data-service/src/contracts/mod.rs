use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

/// Status of a semantic data contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Draft,
    Active,
    Deprecated,
    Violated,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Active => write!(f, "active"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Violated => write!(f, "violated"),
        }
    }
}

impl ContractStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "draft" => Self::Draft,
            "active" => Self::Active,
            "deprecated" => Self::Deprecated,
            "violated" => Self::Violated,
            _ => Self::Draft,
        }
    }
}

/// Semantic type classifying the business meaning of a field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticType {
    Currency,
    Revenue,
    Cost,
    Profit,
    Percentage,
    Count,
    Identifier,
    PersonName,
    Email,
    Phone,
    Address,
    Timestamp,
    Duration,
    Category,
    Status,
    FreeText,
    Custom(String),
}

impl std::fmt::Display for SemanticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Currency => write!(f, "currency"),
            Self::Revenue => write!(f, "revenue"),
            Self::Cost => write!(f, "cost"),
            Self::Profit => write!(f, "profit"),
            Self::Percentage => write!(f, "percentage"),
            Self::Count => write!(f, "count"),
            Self::Identifier => write!(f, "identifier"),
            Self::PersonName => write!(f, "person_name"),
            Self::Email => write!(f, "email"),
            Self::Phone => write!(f, "phone"),
            Self::Address => write!(f, "address"),
            Self::Timestamp => write!(f, "timestamp"),
            Self::Duration => write!(f, "duration"),
            Self::Category => write!(f, "category"),
            Self::Status => write!(f, "status"),
            Self::FreeText => write!(f, "free_text"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Use cases that govern how data may or may not be used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataUseCase {
    Analytics,
    Reporting,
    MlTraining,
    PiiProcessing,
    ExternalSharing,
    MarketingTargeting,
    RiskScoring,
    ComplianceAudit,
    Custom(String),
}

impl std::fmt::Display for DataUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Analytics => write!(f, "analytics"),
            Self::Reporting => write!(f, "reporting"),
            Self::MlTraining => write!(f, "ml_training"),
            Self::PiiProcessing => write!(f, "pii_processing"),
            Self::ExternalSharing => write!(f, "external_sharing"),
            Self::MarketingTargeting => write!(f, "marketing_targeting"),
            Self::RiskScoring => write!(f, "risk_scoring"),
            Self::ComplianceAudit => write!(f, "compliance_audit"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Quality thresholds that data must meet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub min_completeness: Option<f64>,
    pub min_accuracy: Option<f64>,
    pub max_null_percentage: Option<f64>,
    pub max_duplicate_percentage: Option<f64>,
}

/// Types of validation rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRuleType {
    Range,
    Regex,
    Enum,
    Custom,
}

/// A validation rule attached to a contract term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub parameters: serde_json::Value,
}

/// Severity levels for contract check results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

/// A semantic data contract
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SemanticContract {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
    pub version: i32,
    pub status: String,
    pub owner_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A term within a semantic contract defining expectations for a single field
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContractTerm {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub field_name: String,
    pub semantic_type: String,
    pub business_definition: String,
    pub freshness_sla_seconds: Option<i64>,
    pub allowed_uses: serde_json::Value,
    pub prohibited_uses: serde_json::Value,
    pub quality_thresholds: serde_json::Value,
    pub validation_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A contract together with all its terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractWithTerms {
    #[serde(flatten)]
    pub contract: SemanticContract,
    pub terms: Vec<ContractTerm>,
}

/// A historical snapshot of a contract version
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContractHistory {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub version: i32,
    pub name: String,
    pub description: String,
    pub status: String,
    pub snapshot: serde_json::Value,
    pub changed_at: DateTime<Utc>,
}

// ============================================================================
// Validation Result Types
// ============================================================================

/// Overall validation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
}

/// Result of validating an entire contract at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    pub contract_id: Uuid,
    pub validated_at: DateTime<Utc>,
    pub status: ValidationStatus,
    pub term_results: Vec<TermValidationResult>,
}

/// Result of validating a single term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermValidationResult {
    pub term_id: Uuid,
    pub field_name: String,
    pub checks: Vec<CheckResult>,
}

/// Result of a single check within a term validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_type: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
}

// ============================================================================
// Request / Input Types
// ============================================================================

/// Request to create a new contract
#[derive(Debug, Clone, Deserialize)]
pub struct CreateContractRequest {
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub entity_id: Option<Uuid>,
}

/// Request to update an existing contract
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContractRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
}

/// Request to add a term to a contract
#[derive(Debug, Clone, Deserialize)]
pub struct AddTermRequest {
    pub field_name: String,
    pub semantic_type: SemanticType,
    pub business_definition: String,
    pub freshness_sla_seconds: Option<i64>,
    pub allowed_uses: Option<Vec<DataUseCase>>,
    pub prohibited_uses: Option<Vec<DataUseCase>>,
    pub quality_thresholds: Option<QualityThresholds>,
    pub validation_rules: Option<Vec<ValidationRule>>,
}

/// Request to update an existing term
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTermRequest {
    pub field_name: Option<String>,
    pub semantic_type: Option<SemanticType>,
    pub business_definition: Option<String>,
    pub freshness_sla_seconds: Option<i64>,
    pub allowed_uses: Option<Vec<DataUseCase>>,
    pub prohibited_uses: Option<Vec<DataUseCase>>,
    pub quality_thresholds: Option<QualityThresholds>,
    pub validation_rules: Option<Vec<ValidationRule>>,
}

/// Filters for listing contracts
#[derive(Debug, Clone, Deserialize)]
pub struct ListContractsFilter {
    pub entity_id: Option<Uuid>,
    pub status: Option<String>,
}

/// Context provided when validating a contract at runtime
#[derive(Debug, Clone, Deserialize)]
pub struct ValidationContext {
    /// The use case being checked against
    pub use_case: Option<DataUseCase>,
    /// Per-field timestamps indicating when data was last refreshed
    pub field_timestamps: Option<serde_json::Value>,
    /// Per-field sample data for validation rule checks
    pub sample_data: Option<serde_json::Value>,
    /// Latest quality scores (completeness, accuracy, null_pct, duplicate_pct)
    pub quality_scores: Option<serde_json::Value>,
}

/// Request to check if a specific use case is allowed
#[derive(Debug, Clone, Deserialize)]
pub struct CheckUsageRequest {
    pub use_case: DataUseCase,
}

/// Response for a usage check
#[derive(Debug, Clone, Serialize)]
pub struct CheckUsageResponse {
    pub allowed: bool,
    pub prohibited: bool,
    pub details: Vec<String>,
}

// ============================================================================
// Service
// ============================================================================

/// Service for managing semantic data contracts
pub struct ContractsService {
    pool: PgPool,
}

impl ContractsService {
    /// Create a new contracts service with database connection and run migrations
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS semantic_contracts (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                version INT NOT NULL DEFAULT 1,
                status VARCHAR(20) NOT NULL DEFAULT 'draft',
                owner_id UUID NOT NULL,
                entity_id UUID,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contract_terms (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                contract_id UUID NOT NULL REFERENCES semantic_contracts(id) ON DELETE CASCADE,
                field_name VARCHAR(255) NOT NULL,
                semantic_type VARCHAR(100) NOT NULL,
                business_definition TEXT NOT NULL DEFAULT '',
                freshness_sla_seconds BIGINT,
                allowed_uses JSONB NOT NULL DEFAULT '[]',
                prohibited_uses JSONB NOT NULL DEFAULT '[]',
                quality_thresholds JSONB NOT NULL DEFAULT '{}',
                validation_rules JSONB NOT NULL DEFAULT '[]',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contract_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                contract_id UUID NOT NULL REFERENCES semantic_contracts(id) ON DELETE CASCADE,
                version INT NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status VARCHAR(20) NOT NULL,
                snapshot JSONB NOT NULL DEFAULT '{}',
                changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        // Create indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_semantic_contracts_tenant ON semantic_contracts(tenant_id)",
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_semantic_contracts_entity ON semantic_contracts(entity_id)",
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_contract_terms_contract ON contract_terms(contract_id)",
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_contract_history_contract ON contract_history(contract_id)",
        )
        .execute(&pool)
        .await
        .ok();

        info!("ContractsService initialized with PostgreSQL");

        Ok(Self { pool })
    }

    // ------------------------------------------------------------------------
    // Contract CRUD
    // ------------------------------------------------------------------------

    /// Create a new semantic contract with version 1 and draft status
    pub async fn create_contract(
        &self,
        tenant_id: Uuid,
        request: CreateContractRequest,
    ) -> Result<SemanticContract> {
        let contract = sqlx::query_as::<_, SemanticContract>(
            r#"
            INSERT INTO semantic_contracts
                (tenant_id, name, description, version, status, owner_id, entity_id)
            VALUES ($1, $2, $3, 1, 'draft', $4, $5)
            RETURNING id, tenant_id, name, description, version, status,
                      owner_id, entity_id, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.owner_id)
        .bind(request.entity_id)
        .fetch_one(&self.pool)
        .await?;

        // Record initial history
        self.record_history(&contract).await?;

        Ok(contract)
    }

    /// Get a contract by ID together with all its terms
    pub async fn get_contract(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ContractWithTerms>> {
        let contract = sqlx::query_as::<_, SemanticContract>(
            r#"
            SELECT id, tenant_id, name, description, version, status,
                   owner_id, entity_id, created_at, updated_at
            FROM semantic_contracts
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let contract = match contract {
            Some(c) => c,
            None => return Ok(None),
        };

        let terms = sqlx::query_as::<_, ContractTerm>(
            r#"
            SELECT id, contract_id, field_name, semantic_type, business_definition,
                   freshness_sla_seconds, allowed_uses, prohibited_uses,
                   quality_thresholds, validation_rules, created_at, updated_at
            FROM contract_terms
            WHERE contract_id = $1
            ORDER BY field_name
            "#,
        )
        .bind(contract.id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(ContractWithTerms { contract, terms }))
    }

    /// List contracts with optional filters on entity_id and status
    pub async fn list_contracts(
        &self,
        tenant_id: Uuid,
        filters: ListContractsFilter,
    ) -> Result<Vec<SemanticContract>> {
        let contracts = match (filters.entity_id, filters.status.as_deref()) {
            (Some(eid), Some(status)) => {
                sqlx::query_as::<_, SemanticContract>(
                    r#"
                    SELECT id, tenant_id, name, description, version, status,
                           owner_id, entity_id, created_at, updated_at
                    FROM semantic_contracts
                    WHERE tenant_id = $1 AND entity_id = $2 AND status = $3
                    ORDER BY name
                    "#,
                )
                .bind(tenant_id)
                .bind(eid)
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(eid), None) => {
                sqlx::query_as::<_, SemanticContract>(
                    r#"
                    SELECT id, tenant_id, name, description, version, status,
                           owner_id, entity_id, created_at, updated_at
                    FROM semantic_contracts
                    WHERE tenant_id = $1 AND entity_id = $2
                    ORDER BY name
                    "#,
                )
                .bind(tenant_id)
                .bind(eid)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(status)) => {
                sqlx::query_as::<_, SemanticContract>(
                    r#"
                    SELECT id, tenant_id, name, description, version, status,
                           owner_id, entity_id, created_at, updated_at
                    FROM semantic_contracts
                    WHERE tenant_id = $1 AND status = $2
                    ORDER BY name
                    "#,
                )
                .bind(tenant_id)
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, SemanticContract>(
                    r#"
                    SELECT id, tenant_id, name, description, version, status,
                           owner_id, entity_id, created_at, updated_at
                    FROM semantic_contracts
                    WHERE tenant_id = $1
                    ORDER BY name
                    "#,
                )
                .bind(tenant_id)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(contracts)
    }

    /// Update a contract and bump its version number
    pub async fn update_contract(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        request: UpdateContractRequest,
    ) -> Result<Option<SemanticContract>> {
        // Fetch the current contract
        let existing = sqlx::query_as::<_, SemanticContract>(
            r#"
            SELECT id, tenant_id, name, description, version, status,
                   owner_id, entity_id, created_at, updated_at
            FROM semantic_contracts
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let existing = match existing {
            Some(c) => c,
            None => return Ok(None),
        };

        let name = request.name.unwrap_or(existing.name);
        let description = request.description.unwrap_or(existing.description);
        let owner_id = request.owner_id.unwrap_or(existing.owner_id);
        let entity_id = request.entity_id.or(existing.entity_id);
        let new_version = existing.version + 1;

        let updated = sqlx::query_as::<_, SemanticContract>(
            r#"
            UPDATE semantic_contracts
            SET name = $1, description = $2, owner_id = $3, entity_id = $4,
                version = $5, updated_at = NOW()
            WHERE tenant_id = $6 AND id = $7
            RETURNING id, tenant_id, name, description, version, status,
                      owner_id, entity_id, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(owner_id)
        .bind(entity_id)
        .bind(new_version)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref contract) = updated {
            self.record_history(contract).await?;
        }

        Ok(updated)
    }

    /// Activate a contract (set status to active)
    pub async fn activate_contract(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SemanticContract>> {
        let contract = sqlx::query_as::<_, SemanticContract>(
            r#"
            UPDATE semantic_contracts
            SET status = 'active', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            RETURNING id, tenant_id, name, description, version, status,
                      owner_id, entity_id, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref c) = contract {
            self.record_history(c).await?;
        }

        Ok(contract)
    }

    /// Deprecate a contract (set status to deprecated)
    pub async fn deprecate_contract(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SemanticContract>> {
        let contract = sqlx::query_as::<_, SemanticContract>(
            r#"
            UPDATE semantic_contracts
            SET status = 'deprecated', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            RETURNING id, tenant_id, name, description, version, status,
                      owner_id, entity_id, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref c) = contract {
            self.record_history(c).await?;
        }

        Ok(contract)
    }

    // ------------------------------------------------------------------------
    // Term CRUD
    // ------------------------------------------------------------------------

    /// Add a term to a contract
    pub async fn add_term(
        &self,
        contract_id: Uuid,
        request: AddTermRequest,
    ) -> Result<ContractTerm> {
        let semantic_type_str = request.semantic_type.to_string();
        let allowed_uses = serde_json::to_value(request.allowed_uses.unwrap_or_default())?;
        let prohibited_uses = serde_json::to_value(request.prohibited_uses.unwrap_or_default())?;
        let quality_thresholds = serde_json::to_value(
            request.quality_thresholds.unwrap_or(QualityThresholds {
                min_completeness: None,
                min_accuracy: None,
                max_null_percentage: None,
                max_duplicate_percentage: None,
            }),
        )?;
        let validation_rules = serde_json::to_value(request.validation_rules.unwrap_or_default())?;

        let term = sqlx::query_as::<_, ContractTerm>(
            r#"
            INSERT INTO contract_terms
                (contract_id, field_name, semantic_type, business_definition,
                 freshness_sla_seconds, allowed_uses, prohibited_uses,
                 quality_thresholds, validation_rules)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, contract_id, field_name, semantic_type, business_definition,
                      freshness_sla_seconds, allowed_uses, prohibited_uses,
                      quality_thresholds, validation_rules, created_at, updated_at
            "#,
        )
        .bind(contract_id)
        .bind(&request.field_name)
        .bind(&semantic_type_str)
        .bind(&request.business_definition)
        .bind(request.freshness_sla_seconds)
        .bind(&allowed_uses)
        .bind(&prohibited_uses)
        .bind(&quality_thresholds)
        .bind(&validation_rules)
        .fetch_one(&self.pool)
        .await?;

        Ok(term)
    }

    /// Update an existing term
    pub async fn update_term(
        &self,
        term_id: Uuid,
        request: UpdateTermRequest,
    ) -> Result<Option<ContractTerm>> {
        // Fetch existing term
        let existing = sqlx::query_as::<_, ContractTerm>(
            r#"
            SELECT id, contract_id, field_name, semantic_type, business_definition,
                   freshness_sla_seconds, allowed_uses, prohibited_uses,
                   quality_thresholds, validation_rules, created_at, updated_at
            FROM contract_terms
            WHERE id = $1
            "#,
        )
        .bind(term_id)
        .fetch_optional(&self.pool)
        .await?;

        let existing = match existing {
            Some(t) => t,
            None => return Ok(None),
        };

        let field_name = request.field_name.unwrap_or(existing.field_name);
        let semantic_type = request
            .semantic_type
            .map(|st| st.to_string())
            .unwrap_or(existing.semantic_type);
        let business_definition = request
            .business_definition
            .unwrap_or(existing.business_definition);
        let freshness_sla_seconds = request
            .freshness_sla_seconds
            .or(existing.freshness_sla_seconds);
        let allowed_uses = request
            .allowed_uses
            .map(|v| serde_json::to_value(v).unwrap_or_default())
            .unwrap_or(existing.allowed_uses);
        let prohibited_uses = request
            .prohibited_uses
            .map(|v| serde_json::to_value(v).unwrap_or_default())
            .unwrap_or(existing.prohibited_uses);
        let quality_thresholds = request
            .quality_thresholds
            .map(|v| serde_json::to_value(v).unwrap_or_default())
            .unwrap_or(existing.quality_thresholds);
        let validation_rules = request
            .validation_rules
            .map(|v| serde_json::to_value(v).unwrap_or_default())
            .unwrap_or(existing.validation_rules);

        let updated = sqlx::query_as::<_, ContractTerm>(
            r#"
            UPDATE contract_terms
            SET field_name = $1, semantic_type = $2, business_definition = $3,
                freshness_sla_seconds = $4, allowed_uses = $5, prohibited_uses = $6,
                quality_thresholds = $7, validation_rules = $8, updated_at = NOW()
            WHERE id = $9
            RETURNING id, contract_id, field_name, semantic_type, business_definition,
                      freshness_sla_seconds, allowed_uses, prohibited_uses,
                      quality_thresholds, validation_rules, created_at, updated_at
            "#,
        )
        .bind(&field_name)
        .bind(&semantic_type)
        .bind(&business_definition)
        .bind(freshness_sla_seconds)
        .bind(&allowed_uses)
        .bind(&prohibited_uses)
        .bind(&quality_thresholds)
        .bind(&validation_rules)
        .bind(term_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Remove a term from a contract
    pub async fn remove_term(&self, term_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM contract_terms WHERE id = $1")
            .bind(term_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ------------------------------------------------------------------------
    // Validation
    // ------------------------------------------------------------------------

    /// Validate a contract at runtime against the provided context
    pub async fn validate_contract(
        &self,
        tenant_id: Uuid,
        contract_id: Uuid,
        context: ValidationContext,
    ) -> Result<Option<ContractValidationResult>> {
        let contract_with_terms = self.get_contract(tenant_id, contract_id).await?;

        let contract_with_terms = match contract_with_terms {
            Some(c) => c,
            None => return Ok(None),
        };

        let mut term_results: Vec<TermValidationResult> = Vec::new();
        let mut has_failure = false;
        let mut has_warning = false;

        for term in &contract_with_terms.terms {
            let mut checks: Vec<CheckResult> = Vec::new();

            // 1. Freshness SLA check
            if let Some(sla_seconds) = term.freshness_sla_seconds {
                let check = self.check_freshness(term, sla_seconds, &context);
                if !check.passed {
                    if check.severity == Severity::Critical || check.severity == Severity::High {
                        has_failure = true;
                    } else {
                        has_warning = true;
                    }
                }
                checks.push(check);
            }

            // 2. Allowed / prohibited use case check
            if let Some(ref use_case) = context.use_case {
                let check = self.check_use_case(term, use_case);
                if !check.passed {
                    has_failure = true;
                }
                checks.push(check);
            }

            // 3. Quality threshold checks
            if let Some(ref quality_scores) = context.quality_scores {
                let quality_checks = self.check_quality_thresholds(term, quality_scores);
                for check in &quality_checks {
                    if !check.passed {
                        if check.severity == Severity::Critical || check.severity == Severity::High
                        {
                            has_failure = true;
                        } else {
                            has_warning = true;
                        }
                    }
                }
                checks.extend(quality_checks);
            }

            // 4. Validation rule checks
            if let Some(ref sample_data) = context.sample_data {
                let rule_checks = self.check_validation_rules(term, sample_data);
                for check in &rule_checks {
                    if !check.passed {
                        has_warning = true;
                    }
                }
                checks.extend(rule_checks);
            }

            term_results.push(TermValidationResult {
                term_id: term.id,
                field_name: term.field_name.clone(),
                checks,
            });
        }

        let status = if has_failure {
            ValidationStatus::Failed
        } else if has_warning {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Passed
        };

        // If validation failed, mark contract as violated
        if status == ValidationStatus::Failed {
            sqlx::query(
                "UPDATE semantic_contracts SET status = 'violated', updated_at = NOW() WHERE id = $1",
            )
            .bind(contract_id)
            .execute(&self.pool)
            .await
            .ok();
        }

        Ok(Some(ContractValidationResult {
            contract_id,
            validated_at: Utc::now(),
            status,
            term_results,
        }))
    }

    /// Quick check if a use case is allowed by a contract
    pub async fn check_usage(
        &self,
        tenant_id: Uuid,
        contract_id: Uuid,
        use_case: &DataUseCase,
    ) -> Result<Option<CheckUsageResponse>> {
        let contract_with_terms = self.get_contract(tenant_id, contract_id).await?;

        let contract_with_terms = match contract_with_terms {
            Some(c) => c,
            None => return Ok(None),
        };

        let use_case_str = serde_json::to_value(use_case)?;
        let mut allowed = true;
        let mut prohibited = false;
        let mut details: Vec<String> = Vec::new();

        for term in &contract_with_terms.terms {
            // Check prohibited list
            if let Some(prohibited_list) = term.prohibited_uses.as_array() {
                if prohibited_list.contains(&use_case_str) {
                    prohibited = true;
                    allowed = false;
                    details.push(format!(
                        "Use case '{}' is prohibited by term '{}' ({})",
                        use_case, term.field_name, term.business_definition
                    ));
                }
            }

            // Check allowed list (if specified, use case must be in it)
            if let Some(allowed_list) = term.allowed_uses.as_array() {
                if !allowed_list.is_empty() && !allowed_list.contains(&use_case_str) {
                    allowed = false;
                    details.push(format!(
                        "Use case '{}' is not in the allowed list for term '{}'",
                        use_case, term.field_name
                    ));
                }
            }
        }

        if details.is_empty() && allowed {
            details.push(format!(
                "Use case '{}' is allowed by all contract terms",
                use_case
            ));
        }

        Ok(Some(CheckUsageResponse {
            allowed,
            prohibited,
            details,
        }))
    }

    /// Get the version history of a contract
    pub async fn get_contract_history(
        &self,
        tenant_id: Uuid,
        contract_id: Uuid,
    ) -> Result<Vec<ContractHistory>> {
        // Verify contract belongs to tenant
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM semantic_contracts WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(contract_id)
        .fetch_optional(&self.pool)
        .await?;

        if exists.is_none() {
            anyhow::bail!("Contract not found");
        }

        let history = sqlx::query_as::<_, ContractHistory>(
            r#"
            SELECT id, contract_id, version, name, description, status, snapshot, changed_at
            FROM contract_history
            WHERE contract_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    // ------------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------------

    /// Record a snapshot in the contract history table
    async fn record_history(&self, contract: &SemanticContract) -> Result<()> {
        let snapshot = serde_json::to_value(contract)?;

        sqlx::query(
            r#"
            INSERT INTO contract_history
                (contract_id, version, name, description, status, snapshot)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(contract.id)
        .bind(contract.version)
        .bind(&contract.name)
        .bind(&contract.description)
        .bind(&contract.status)
        .bind(&snapshot)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check freshness SLA for a single term
    fn check_freshness(
        &self,
        term: &ContractTerm,
        sla_seconds: i64,
        context: &ValidationContext,
    ) -> CheckResult {
        if let Some(ref field_ts) = context.field_timestamps {
            if let Some(ts_str) = field_ts
                .get(&term.field_name)
                .and_then(|v| v.as_str())
            {
                if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                    let age = Utc::now().signed_duration_since(ts);
                    let age_seconds = age.num_seconds();

                    if age_seconds > sla_seconds {
                        return CheckResult {
                            check_type: "freshness".to_string(),
                            passed: false,
                            message: format!(
                                "Field '{}' data is {} seconds old, exceeds SLA of {} seconds",
                                term.field_name, age_seconds, sla_seconds
                            ),
                            severity: Severity::High,
                        };
                    }

                    return CheckResult {
                        check_type: "freshness".to_string(),
                        passed: true,
                        message: format!(
                            "Field '{}' data is {} seconds old, within SLA of {} seconds",
                            term.field_name, age_seconds, sla_seconds
                        ),
                        severity: Severity::Low,
                    };
                }
            }
        }

        // No timestamp data provided — cannot verify freshness
        CheckResult {
            check_type: "freshness".to_string(),
            passed: false,
            message: format!(
                "No timestamp data provided for field '{}', cannot verify freshness SLA",
                term.field_name
            ),
            severity: Severity::Medium,
        }
    }

    /// Check whether the requested use case is allowed by a term
    fn check_use_case(&self, term: &ContractTerm, use_case: &DataUseCase) -> CheckResult {
        let use_case_val = serde_json::to_value(use_case).unwrap_or_default();

        // Check prohibited uses first
        if let Some(prohibited_list) = term.prohibited_uses.as_array() {
            if prohibited_list.contains(&use_case_val) {
                return CheckResult {
                    check_type: "allowed_use".to_string(),
                    passed: false,
                    message: format!(
                        "Use case '{}' is explicitly prohibited for field '{}'",
                        use_case, term.field_name
                    ),
                    severity: Severity::Critical,
                };
            }
        }

        // Check allowed uses (if non-empty, must be present)
        if let Some(allowed_list) = term.allowed_uses.as_array() {
            if !allowed_list.is_empty() && !allowed_list.contains(&use_case_val) {
                return CheckResult {
                    check_type: "allowed_use".to_string(),
                    passed: false,
                    message: format!(
                        "Use case '{}' is not in the allowed list for field '{}'",
                        use_case, term.field_name
                    ),
                    severity: Severity::High,
                };
            }
        }

        CheckResult {
            check_type: "allowed_use".to_string(),
            passed: true,
            message: format!(
                "Use case '{}' is permitted for field '{}'",
                use_case, term.field_name
            ),
            severity: Severity::Low,
        }
    }

    /// Check quality thresholds for a term against provided quality scores
    fn check_quality_thresholds(
        &self,
        term: &ContractTerm,
        quality_scores: &serde_json::Value,
    ) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        let thresholds = &term.quality_thresholds;

        // Extract field-specific quality scores or use top-level scores
        let field_scores = quality_scores
            .get(&term.field_name)
            .unwrap_or(quality_scores);

        // Check min_completeness
        if let Some(min_completeness) = thresholds
            .get("min_completeness")
            .and_then(|v| v.as_f64())
        {
            let actual = field_scores
                .get("completeness")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            checks.push(CheckResult {
                check_type: "quality".to_string(),
                passed: actual >= min_completeness,
                message: format!(
                    "Field '{}' completeness: {:.2} (threshold: {:.2})",
                    term.field_name, actual, min_completeness
                ),
                severity: if actual >= min_completeness {
                    Severity::Low
                } else {
                    Severity::High
                },
            });
        }

        // Check min_accuracy
        if let Some(min_accuracy) = thresholds.get("min_accuracy").and_then(|v| v.as_f64()) {
            let actual = field_scores
                .get("accuracy")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            checks.push(CheckResult {
                check_type: "quality".to_string(),
                passed: actual >= min_accuracy,
                message: format!(
                    "Field '{}' accuracy: {:.2} (threshold: {:.2})",
                    term.field_name, actual, min_accuracy
                ),
                severity: if actual >= min_accuracy {
                    Severity::Low
                } else {
                    Severity::High
                },
            });
        }

        // Check max_null_percentage
        if let Some(max_null_pct) = thresholds
            .get("max_null_percentage")
            .and_then(|v| v.as_f64())
        {
            let actual = field_scores
                .get("null_percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            checks.push(CheckResult {
                check_type: "quality".to_string(),
                passed: actual <= max_null_pct,
                message: format!(
                    "Field '{}' null percentage: {:.2} (max: {:.2})",
                    term.field_name, actual, max_null_pct
                ),
                severity: if actual <= max_null_pct {
                    Severity::Low
                } else {
                    Severity::Medium
                },
            });
        }

        // Check max_duplicate_percentage
        if let Some(max_dup_pct) = thresholds
            .get("max_duplicate_percentage")
            .and_then(|v| v.as_f64())
        {
            let actual = field_scores
                .get("duplicate_percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            checks.push(CheckResult {
                check_type: "quality".to_string(),
                passed: actual <= max_dup_pct,
                message: format!(
                    "Field '{}' duplicate percentage: {:.2} (max: {:.2})",
                    term.field_name, actual, max_dup_pct
                ),
                severity: if actual <= max_dup_pct {
                    Severity::Low
                } else {
                    Severity::Medium
                },
            });
        }

        checks
    }

    /// Check validation rules against sample data for a term
    fn check_validation_rules(
        &self,
        term: &ContractTerm,
        sample_data: &serde_json::Value,
    ) -> Vec<CheckResult> {
        let mut checks = Vec::new();

        let rules: Vec<ValidationRule> =
            serde_json::from_value(term.validation_rules.clone()).unwrap_or_default();

        let field_samples = sample_data
            .get(&term.field_name)
            .and_then(|v| v.as_array());

        let field_samples = match field_samples {
            Some(samples) => samples,
            None => return checks,
        };

        for rule in &rules {
            match rule.rule_type {
                ValidationRuleType::Range => {
                    let min = rule.parameters.get("min").and_then(|v| v.as_f64());
                    let max = rule.parameters.get("max").and_then(|v| v.as_f64());
                    let mut violations = 0;

                    for sample in field_samples {
                        if let Some(val) = sample.as_f64() {
                            if let Some(min_val) = min {
                                if val < min_val {
                                    violations += 1;
                                    continue;
                                }
                            }
                            if let Some(max_val) = max {
                                if val > max_val {
                                    violations += 1;
                                }
                            }
                        }
                    }

                    checks.push(CheckResult {
                        check_type: "validation".to_string(),
                        passed: violations == 0,
                        message: format!(
                            "Field '{}' range check: {}/{} samples within range (min: {:?}, max: {:?})",
                            term.field_name,
                            field_samples.len() - violations,
                            field_samples.len(),
                            min,
                            max
                        ),
                        severity: if violations == 0 {
                            Severity::Low
                        } else {
                            Severity::Medium
                        },
                    });
                }
                ValidationRuleType::Regex => {
                    let pattern = rule
                        .parameters
                        .get("pattern")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".*");

                    match regex::Regex::new(pattern) {
                        Ok(re) => {
                            let mut violations = 0;
                            for sample in field_samples {
                                if let Some(val) = sample.as_str() {
                                    if !re.is_match(val) {
                                        violations += 1;
                                    }
                                }
                            }

                            checks.push(CheckResult {
                                check_type: "validation".to_string(),
                                passed: violations == 0,
                                message: format!(
                                    "Field '{}' regex check ('{}'): {}/{} samples match",
                                    term.field_name,
                                    pattern,
                                    field_samples.len() - violations,
                                    field_samples.len()
                                ),
                                severity: if violations == 0 {
                                    Severity::Low
                                } else {
                                    Severity::Medium
                                },
                            });
                        }
                        Err(e) => {
                            checks.push(CheckResult {
                                check_type: "validation".to_string(),
                                passed: false,
                                message: format!(
                                    "Field '{}' regex check failed: invalid pattern '{}': {}",
                                    term.field_name, pattern, e
                                ),
                                severity: Severity::High,
                            });
                        }
                    }
                }
                ValidationRuleType::Enum => {
                    let allowed_values = rule
                        .parameters
                        .get("allowed_values")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    let mut violations = 0;
                    for sample in field_samples {
                        if !allowed_values.contains(sample) {
                            violations += 1;
                        }
                    }

                    checks.push(CheckResult {
                        check_type: "validation".to_string(),
                        passed: violations == 0,
                        message: format!(
                            "Field '{}' enum check: {}/{} samples in allowed set",
                            term.field_name,
                            field_samples.len() - violations,
                            field_samples.len()
                        ),
                        severity: if violations == 0 {
                            Severity::Low
                        } else {
                            Severity::Medium
                        },
                    });
                }
                ValidationRuleType::Custom => {
                    let description = rule
                        .parameters
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("custom validation");

                    checks.push(CheckResult {
                        check_type: "validation".to_string(),
                        passed: true,
                        message: format!(
                            "Field '{}' custom rule '{}': requires manual verification",
                            term.field_name, description
                        ),
                        severity: Severity::Low,
                    });
                }
            }
        }

        checks
    }
}

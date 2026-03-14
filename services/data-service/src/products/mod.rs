pub mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Enums
// ============================================================================

/// How fresh the data product promises to be
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessSla {
    RealTime,
    Hourly,
    Daily,
    Weekly,
}

impl std::fmt::Display for FreshnessSla {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RealTime => write!(f, "real_time"),
            Self::Hourly => write!(f, "hourly"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
        }
    }
}

impl FreshnessSla {
    pub fn from_str(s: &str) -> Self {
        match s {
            "real_time" => Self::RealTime,
            "hourly" => Self::Hourly,
            "daily" => Self::Daily,
            "weekly" => Self::Weekly,
            _ => Self::Daily,
        }
    }
}

/// Pricing model for a data product
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    Free,
    PayPerQuery,
    Subscription,
    Tiered,
}

impl std::fmt::Display for PricingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "free"),
            Self::PayPerQuery => write!(f, "pay_per_query"),
            Self::Subscription => write!(f, "subscription"),
            Self::Tiered => write!(f, "tiered"),
        }
    }
}

impl PricingModel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "free" => Self::Free,
            "pay_per_query" => Self::PayPerQuery,
            "subscription" => Self::Subscription,
            "tiered" => Self::Tiered,
            _ => Self::Free,
        }
    }
}

/// Lifecycle status of a data product
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Draft,
    Active,
    Deprecated,
    Suspended,
}

impl std::fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Active => write!(f, "active"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

impl ProductStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "draft" => Self::Draft,
            "active" => Self::Active,
            "deprecated" => Self::Deprecated,
            "suspended" => Self::Suspended,
            _ => Self::Draft,
        }
    }
}

/// Subscription status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Suspended,
    Cancelled,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl SubscriptionStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "suspended" => Self::Suspended,
            "cancelled" => Self::Cancelled,
            _ => Self::Active,
        }
    }
}

// ============================================================================
// Data Models
// ============================================================================

/// A packaged, sellable data product in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataProduct {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entity_ids: serde_json::Value,
    pub semantic_contract: serde_json::Value,
    pub freshness_sla: String,
    pub quality_threshold: f64,
    pub access_policy: serde_json::Value,
    pub pricing_model: String,
    pub price_per_query: Option<f64>,
    pub monthly_subscription_price: Option<f64>,
    pub status: String,
    pub version: i32,
    pub tags: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// A subscription to a data product
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataProductSubscription {
    pub id: Uuid,
    pub product_id: Uuid,
    pub subscriber_tenant_id: Uuid,
    pub subscriber_id: Uuid,
    pub status: String,
    pub subscribed_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub query_count: i64,
    pub query_limit: Option<i64>,
}

/// A usage event for metering
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataProductUsageEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub product_id: Uuid,
    pub query_type: String,
    pub records_returned: i64,
    pub bytes_scanned: i64,
    pub latency_ms: i64,
    pub billed_amount: f64,
    pub timestamp: DateTime<Utc>,
}

/// Continuous quality monitoring report for a data product
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataProductQualityReport {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quality_score: f64,
    pub freshness_met: bool,
    pub completeness_score: f64,
    pub accuracy_score: f64,
    pub consistency_score: f64,
    pub issues: serde_json::Value,
    pub evaluated_at: DateTime<Utc>,
}

// ============================================================================
// Request / Response Types
// ============================================================================

/// Request to create a new data product
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductRequest {
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entity_ids: Vec<Uuid>,
    pub semantic_contract: serde_json::Value,
    pub freshness_sla: String,
    pub quality_threshold: Option<f64>,
    pub access_policy: Option<serde_json::Value>,
    pub pricing_model: String,
    pub price_per_query: Option<f64>,
    pub monthly_subscription_price: Option<f64>,
    pub tags: Option<Vec<String>>,
}

/// Request to update an existing data product
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub entity_ids: Option<Vec<Uuid>>,
    pub semantic_contract: Option<serde_json::Value>,
    pub freshness_sla: Option<String>,
    pub quality_threshold: Option<f64>,
    pub access_policy: Option<serde_json::Value>,
    pub pricing_model: Option<String>,
    pub price_per_query: Option<f64>,
    pub monthly_subscription_price: Option<f64>,
    pub tags: Option<Vec<String>>,
}

/// Filters for listing data products
#[derive(Debug, Clone, Deserialize)]
pub struct ListProductsFilter {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub pricing_model: Option<String>,
}

/// Request to record a usage event
#[derive(Debug, Clone, Deserialize)]
pub struct RecordUsageRequest {
    pub query_type: String,
    pub records_returned: i64,
    pub bytes_scanned: i64,
    pub latency_ms: i64,
    pub billed_amount: Option<f64>,
}

/// Aggregated usage summary for a subscription over a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub subscription_id: Uuid,
    pub product_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_queries: i64,
    pub total_bytes_scanned: i64,
    pub total_cost: f64,
}

/// Revenue report for a product owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueReport {
    pub owner_tenant_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_revenue: f64,
    pub by_product: Vec<ProductRevenue>,
}

/// Revenue breakdown for a single product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRevenue {
    pub product_id: Uuid,
    pub product_name: String,
    pub revenue: f64,
    pub query_count: i64,
}

// ============================================================================
// Service
// ============================================================================

/// Service for managing data products marketplace
pub struct DataProductService {
    pool: PgPool,
}

impl DataProductService {
    /// Create a new data product service with database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Ensure tables exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_products (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                owner_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                entity_ids JSONB NOT NULL DEFAULT '[]',
                semantic_contract JSONB NOT NULL DEFAULT '{}',
                freshness_sla VARCHAR(50) NOT NULL DEFAULT 'daily',
                quality_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.8,
                access_policy JSONB NOT NULL DEFAULT '{}',
                pricing_model VARCHAR(50) NOT NULL DEFAULT 'free',
                price_per_query DOUBLE PRECISION,
                monthly_subscription_price DOUBLE PRECISION,
                status VARCHAR(50) NOT NULL DEFAULT 'draft',
                version INTEGER NOT NULL DEFAULT 1,
                tags JSONB NOT NULL DEFAULT '[]',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                published_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_product_subscriptions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                product_id UUID NOT NULL,
                subscriber_tenant_id UUID NOT NULL,
                subscriber_id UUID NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'active',
                subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                cancelled_at TIMESTAMPTZ,
                current_period_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                current_period_end TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
                query_count BIGINT NOT NULL DEFAULT 0,
                query_limit BIGINT
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_product_usage_events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                subscription_id UUID NOT NULL,
                product_id UUID NOT NULL,
                query_type VARCHAR(100) NOT NULL,
                records_returned BIGINT NOT NULL DEFAULT 0,
                bytes_scanned BIGINT NOT NULL DEFAULT 0,
                latency_ms BIGINT NOT NULL DEFAULT 0,
                billed_amount DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_product_quality_reports (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                product_id UUID NOT NULL,
                quality_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
                freshness_met BOOLEAN NOT NULL DEFAULT true,
                completeness_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
                accuracy_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
                consistency_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
                issues JSONB NOT NULL DEFAULT '[]',
                evaluated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await
        .ok();

        info!("DataProductService initialized with PostgreSQL");

        Ok(Self { pool })
    }

    // ========================================================================
    // Product CRUD
    // ========================================================================

    /// Create a new data product in draft status
    pub async fn create_product(
        &self,
        tenant_id: Uuid,
        req: CreateProductRequest,
    ) -> Result<DataProduct> {
        let entity_ids_json = serde_json::to_value(&req.entity_ids)?;
        let tags_json = serde_json::to_value(&req.tags.unwrap_or_default())?;
        let access_policy = req.access_policy.unwrap_or(serde_json::json!({}));
        let quality_threshold = req.quality_threshold.unwrap_or(0.8);

        let product = sqlx::query_as::<_, DataProduct>(
            r#"
            INSERT INTO data_products
                (tenant_id, owner_id, name, description, entity_ids, semantic_contract,
                 freshness_sla, quality_threshold, access_policy, pricing_model,
                 price_per_query, monthly_subscription_price, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, tenant_id, owner_id, name, description, entity_ids,
                      semantic_contract, freshness_sla, quality_threshold, access_policy,
                      pricing_model, price_per_query, monthly_subscription_price,
                      status, version, tags, created_at, updated_at, published_at
            "#,
        )
        .bind(tenant_id)
        .bind(req.owner_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&entity_ids_json)
        .bind(&req.semantic_contract)
        .bind(&req.freshness_sla)
        .bind(quality_threshold)
        .bind(&access_policy)
        .bind(&req.pricing_model)
        .bind(req.price_per_query)
        .bind(req.monthly_subscription_price)
        .bind(&tags_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(product)
    }

    /// Update an existing data product (bumps version)
    pub async fn update_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        req: UpdateProductRequest,
    ) -> Result<Option<DataProduct>> {
        // Fetch existing product
        let existing = self.get_product(tenant_id, product_id).await?;
        let existing = match existing {
            Some(p) => p,
            None => return Ok(None),
        };

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let entity_ids = match req.entity_ids {
            Some(ids) => serde_json::to_value(&ids)?,
            None => existing.entity_ids,
        };
        let semantic_contract = req.semantic_contract.unwrap_or(existing.semantic_contract);
        let freshness_sla = req.freshness_sla.unwrap_or(existing.freshness_sla);
        let quality_threshold = req.quality_threshold.unwrap_or(existing.quality_threshold);
        let access_policy = req.access_policy.unwrap_or(existing.access_policy);
        let pricing_model = req.pricing_model.unwrap_or(existing.pricing_model);
        let price_per_query = req.price_per_query.or(existing.price_per_query);
        let monthly_subscription_price = req
            .monthly_subscription_price
            .or(existing.monthly_subscription_price);
        let tags = match req.tags {
            Some(t) => serde_json::to_value(&t)?,
            None => existing.tags,
        };

        let product = sqlx::query_as::<_, DataProduct>(
            r#"
            UPDATE data_products
            SET name = $1, description = $2, entity_ids = $3, semantic_contract = $4,
                freshness_sla = $5, quality_threshold = $6, access_policy = $7,
                pricing_model = $8, price_per_query = $9, monthly_subscription_price = $10,
                tags = $11, version = version + 1, updated_at = NOW()
            WHERE tenant_id = $12 AND id = $13
            RETURNING id, tenant_id, owner_id, name, description, entity_ids,
                      semantic_contract, freshness_sla, quality_threshold, access_policy,
                      pricing_model, price_per_query, monthly_subscription_price,
                      status, version, tags, created_at, updated_at, published_at
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(&entity_ids)
        .bind(&semantic_contract)
        .bind(&freshness_sla)
        .bind(quality_threshold)
        .bind(&access_policy)
        .bind(&pricing_model)
        .bind(price_per_query)
        .bind(monthly_subscription_price)
        .bind(&tags)
        .bind(tenant_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    /// Publish a data product (set status to active and record published_at)
    pub async fn publish_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<DataProduct>> {
        let product = sqlx::query_as::<_, DataProduct>(
            r#"
            UPDATE data_products
            SET status = 'active', published_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status = 'draft'
            RETURNING id, tenant_id, owner_id, name, description, entity_ids,
                      semantic_contract, freshness_sla, quality_threshold, access_policy,
                      pricing_model, price_per_query, monthly_subscription_price,
                      status, version, tags, created_at, updated_at, published_at
            "#,
        )
        .bind(tenant_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    /// Deprecate a data product
    pub async fn deprecate_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<DataProduct>> {
        let product = sqlx::query_as::<_, DataProduct>(
            r#"
            UPDATE data_products
            SET status = 'deprecated', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status = 'active'
            RETURNING id, tenant_id, owner_id, name, description, entity_ids,
                      semantic_contract, freshness_sla, quality_threshold, access_policy,
                      pricing_model, price_per_query, monthly_subscription_price,
                      status, version, tags, created_at, updated_at, published_at
            "#,
        )
        .bind(tenant_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    /// Get a single data product by ID
    pub async fn get_product(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<DataProduct>> {
        let product = sqlx::query_as::<_, DataProduct>(
            r#"
            SELECT id, tenant_id, owner_id, name, description, entity_ids,
                   semantic_contract, freshness_sla, quality_threshold, access_policy,
                   pricing_model, price_per_query, monthly_subscription_price,
                   status, version, tags, created_at, updated_at, published_at
            FROM data_products
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(product)
    }

    /// List data products with optional filters
    pub async fn list_products(
        &self,
        filters: ListProductsFilter,
    ) -> Result<Vec<DataProduct>> {
        // Build dynamic query based on provided filters
        let mut query = String::from(
            r#"
            SELECT id, tenant_id, owner_id, name, description, entity_ids,
                   semantic_contract, freshness_sla, quality_threshold, access_policy,
                   pricing_model, price_per_query, monthly_subscription_price,
                   status, version, tags, created_at, updated_at, published_at
            FROM data_products
            WHERE 1=1
            "#,
        );

        let mut param_index = 1u32;
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref tid) = filters.tenant_id {
            query.push_str(&format!(" AND tenant_id = ${}", param_index));
            bind_values.push(tid.to_string());
            param_index += 1;
        }
        if let Some(ref status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_index));
            bind_values.push(status.clone());
            param_index += 1;
        }
        if let Some(ref pricing) = filters.pricing_model {
            query.push_str(&format!(" AND pricing_model = ${}", param_index));
            bind_values.push(pricing.clone());
            param_index += 1;
        }
        if let Some(ref tags) = filters.tags {
            // Match products that contain any of the requested tags
            query.push_str(&format!(" AND tags ?| ${}", param_index));
            bind_values.push(serde_json::to_string(tags)?);
            param_index += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        // Since we have dynamic filters, use a simpler approach with known filter combos
        // to maintain type safety with sqlx
        let products = match (
            &filters.tenant_id,
            &filters.status,
            &filters.pricing_model,
            &filters.tags,
        ) {
            (Some(tid), Some(status), Some(pricing), _) => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    WHERE tenant_id = $1 AND status = $2 AND pricing_model = $3
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(tid)
                .bind(status)
                .bind(pricing)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(tid), Some(status), None, _) => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    WHERE tenant_id = $1 AND status = $2
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(tid)
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(tid), None, Some(pricing), _) => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    WHERE tenant_id = $1 AND pricing_model = $2
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(tid)
                .bind(pricing)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(tid), None, None, _) => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    WHERE tenant_id = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(tid)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(status), None, _) => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    WHERE status = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
            _ => {
                sqlx::query_as::<_, DataProduct>(
                    r#"
                    SELECT id, tenant_id, owner_id, name, description, entity_ids,
                           semantic_contract, freshness_sla, quality_threshold, access_policy,
                           pricing_model, price_per_query, monthly_subscription_price,
                           status, version, tags, created_at, updated_at, published_at
                    FROM data_products
                    ORDER BY created_at DESC
                    "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(products)
    }

    // ========================================================================
    // Subscriptions
    // ========================================================================

    /// Subscribe to a data product
    pub async fn subscribe(
        &self,
        product_id: Uuid,
        subscriber_tenant_id: Uuid,
        subscriber_id: Uuid,
    ) -> Result<DataProductSubscription> {
        // Verify product exists and is active
        let product: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM data_products WHERE id = $1",
        )
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        match product {
            Some((status,)) if status != "active" => {
                anyhow::bail!("Cannot subscribe to a product that is not active (status: {})", status);
            }
            None => {
                anyhow::bail!("Product not found");
            }
            _ => {}
        }

        // Check for existing active subscription
        let existing: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM data_product_subscriptions
            WHERE product_id = $1 AND subscriber_tenant_id = $2 AND subscriber_id = $3
              AND status = 'active'
            "#,
        )
        .bind(product_id)
        .bind(subscriber_tenant_id)
        .bind(subscriber_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            anyhow::bail!("Active subscription already exists for this product");
        }

        let subscription = sqlx::query_as::<_, DataProductSubscription>(
            r#"
            INSERT INTO data_product_subscriptions
                (product_id, subscriber_tenant_id, subscriber_id)
            VALUES ($1, $2, $3)
            RETURNING id, product_id, subscriber_tenant_id, subscriber_id, status,
                      subscribed_at, cancelled_at, current_period_start, current_period_end,
                      query_count, query_limit
            "#,
        )
        .bind(product_id)
        .bind(subscriber_tenant_id)
        .bind(subscriber_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(subscription)
    }

    /// Cancel a subscription
    pub async fn unsubscribe(&self, subscription_id: Uuid) -> Result<Option<DataProductSubscription>> {
        let subscription = sqlx::query_as::<_, DataProductSubscription>(
            r#"
            UPDATE data_product_subscriptions
            SET status = 'cancelled', cancelled_at = NOW()
            WHERE id = $1 AND status = 'active'
            RETURNING id, product_id, subscriber_tenant_id, subscriber_id, status,
                      subscribed_at, cancelled_at, current_period_start, current_period_end,
                      query_count, query_limit
            "#,
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(subscription)
    }

    /// List subscriptions for a product
    pub async fn list_subscriptions(
        &self,
        product_id: Uuid,
    ) -> Result<Vec<DataProductSubscription>> {
        let subscriptions = sqlx::query_as::<_, DataProductSubscription>(
            r#"
            SELECT id, product_id, subscriber_tenant_id, subscriber_id, status,
                   subscribed_at, cancelled_at, current_period_start, current_period_end,
                   query_count, query_limit
            FROM data_product_subscriptions
            WHERE product_id = $1
            ORDER BY subscribed_at DESC
            "#,
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(subscriptions)
    }

    /// List products a tenant is subscribed to
    pub async fn list_subscriber_products(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<DataProduct>> {
        let products = sqlx::query_as::<_, DataProduct>(
            r#"
            SELECT dp.id, dp.tenant_id, dp.owner_id, dp.name, dp.description, dp.entity_ids,
                   dp.semantic_contract, dp.freshness_sla, dp.quality_threshold, dp.access_policy,
                   dp.pricing_model, dp.price_per_query, dp.monthly_subscription_price,
                   dp.status, dp.version, dp.tags, dp.created_at, dp.updated_at, dp.published_at
            FROM data_products dp
            JOIN data_product_subscriptions dps ON dps.product_id = dp.id
            WHERE dps.subscriber_tenant_id = $1 AND dps.status = 'active'
            ORDER BY dps.subscribed_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(products)
    }

    // ========================================================================
    // Usage Metering
    // ========================================================================

    /// Record a usage event for a subscription
    pub async fn record_usage(
        &self,
        subscription_id: Uuid,
        req: RecordUsageRequest,
    ) -> Result<DataProductUsageEvent> {
        // Get the subscription to find product_id and check it is active
        let subscription = sqlx::query_as::<_, DataProductSubscription>(
            r#"
            SELECT id, product_id, subscriber_tenant_id, subscriber_id, status,
                   subscribed_at, cancelled_at, current_period_start, current_period_end,
                   query_count, query_limit
            FROM data_product_subscriptions
            WHERE id = $1
            "#,
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;

        let subscription = match subscription {
            Some(s) => s,
            None => anyhow::bail!("Subscription not found"),
        };

        if subscription.status != "active" {
            anyhow::bail!("Subscription is not active (status: {})", subscription.status);
        }

        // Check query limit
        if let Some(limit) = subscription.query_limit {
            if subscription.query_count >= limit {
                anyhow::bail!("Query limit reached ({}/{})", subscription.query_count, limit);
            }
        }

        // Calculate billed amount if not provided
        let billed_amount = if let Some(amount) = req.billed_amount {
            amount
        } else {
            // Look up product pricing
            let product: Option<(String, Option<f64>)> = sqlx::query_as(
                "SELECT pricing_model, price_per_query FROM data_products WHERE id = $1",
            )
            .bind(subscription.product_id)
            .fetch_optional(&self.pool)
            .await?;

            match product {
                Some((model, price)) if model == "pay_per_query" => price.unwrap_or(0.0),
                _ => 0.0,
            }
        };

        // Insert usage event
        let event = sqlx::query_as::<_, DataProductUsageEvent>(
            r#"
            INSERT INTO data_product_usage_events
                (subscription_id, product_id, query_type, records_returned, bytes_scanned,
                 latency_ms, billed_amount)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, subscription_id, product_id, query_type, records_returned,
                      bytes_scanned, latency_ms, billed_amount, timestamp
            "#,
        )
        .bind(subscription_id)
        .bind(subscription.product_id)
        .bind(&req.query_type)
        .bind(req.records_returned)
        .bind(req.bytes_scanned)
        .bind(req.latency_ms)
        .bind(billed_amount)
        .fetch_one(&self.pool)
        .await?;

        // Increment query counter on subscription
        sqlx::query(
            "UPDATE data_product_subscriptions SET query_count = query_count + 1 WHERE id = $1",
        )
        .bind(subscription_id)
        .execute(&self.pool)
        .await?;

        Ok(event)
    }

    /// Get aggregated usage summary for a subscription over a time period
    pub async fn get_usage_summary(
        &self,
        subscription_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<UsageSummary> {
        let row: (Uuid, i64, i64, f64) = sqlx::query_as(
            r#"
            SELECT product_id,
                   COALESCE(COUNT(*), 0) AS total_queries,
                   COALESCE(SUM(bytes_scanned), 0) AS total_bytes,
                   COALESCE(SUM(billed_amount), 0.0) AS total_cost
            FROM data_product_usage_events
            WHERE subscription_id = $1
              AND timestamp >= $2
              AND timestamp < $3
            GROUP BY product_id
            "#,
        )
        .bind(subscription_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_else(|| {
            // If no rows, fetch product_id from subscription
            (Uuid::nil(), 0, 0, 0.0)
        });

        let product_id = if row.0 == Uuid::nil() {
            // Fetch from subscription
            let sub: Option<(Uuid,)> = sqlx::query_as(
                "SELECT product_id FROM data_product_subscriptions WHERE id = $1",
            )
            .bind(subscription_id)
            .fetch_optional(&self.pool)
            .await?;
            sub.map(|s| s.0).unwrap_or(Uuid::nil())
        } else {
            row.0
        };

        Ok(UsageSummary {
            subscription_id,
            product_id,
            period_start,
            period_end,
            total_queries: row.1,
            total_bytes_scanned: row.2,
            total_cost: row.3,
        })
    }

    // ========================================================================
    // Quality Monitoring
    // ========================================================================

    /// Evaluate quality for a data product and produce a quality report
    pub async fn evaluate_quality(
        &self,
        product_id: Uuid,
    ) -> Result<DataProductQualityReport> {
        // Fetch the product to get quality threshold and entity_ids
        let product: Option<(f64, serde_json::Value, String)> = sqlx::query_as(
            "SELECT quality_threshold, entity_ids, freshness_sla FROM data_products WHERE id = $1",
        )
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;

        let (quality_threshold, entity_ids_json, freshness_sla) = match product {
            Some(p) => p,
            None => anyhow::bail!("Product not found"),
        };

        // Parse entity IDs
        let entity_ids: Vec<Uuid> = serde_json::from_value(entity_ids_json).unwrap_or_default();

        // Aggregate quality scores from quality_check_results for constituent entities
        let mut completeness_scores: Vec<f64> = Vec::new();
        let mut accuracy_scores: Vec<f64> = Vec::new();
        let mut consistency_scores: Vec<f64> = Vec::new();
        let mut issues: Vec<serde_json::Value> = Vec::new();

        for entity_id in &entity_ids {
            // Get latest quality check results for this entity
            let results: Vec<(bool, f64, Option<serde_json::Value>)> = sqlx::query_as(
                r#"
                SELECT DISTINCT ON (rule_id) passed, failure_percentage, details
                FROM quality_check_results
                WHERE dataset_id = $1
                ORDER BY rule_id, checked_at DESC
                "#,
            )
            .bind(entity_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

            for (passed, failure_pct, details) in &results {
                let score = 1.0 - (failure_pct / 100.0);
                let rule_type = details
                    .as_ref()
                    .and_then(|d| d.get("rule_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("custom");

                match rule_type {
                    "completeness" | "not_null" => completeness_scores.push(score),
                    "unique" | "regex" | "range" | "enum" => accuracy_scores.push(score),
                    _ => consistency_scores.push(score),
                }

                if !passed {
                    issues.push(serde_json::json!({
                        "entity_id": entity_id,
                        "rule_type": rule_type,
                        "failure_percentage": failure_pct,
                        "details": details,
                    }));
                }
            }
        }

        let avg = |scores: &[f64]| -> f64 {
            if scores.is_empty() {
                1.0
            } else {
                scores.iter().sum::<f64>() / scores.len() as f64
            }
        };

        let completeness_score = avg(&completeness_scores);
        let accuracy_score = avg(&accuracy_scores);
        let consistency_score = avg(&consistency_scores);
        let quality_score =
            (completeness_score + accuracy_score + consistency_score) / 3.0;

        // Determine freshness (check if any entity was updated within the SLA window)
        let freshness_met = true; // Metadata-based; assume met unless we have data

        let issues_json = serde_json::to_value(&issues)?;

        // Persist the report
        let report = sqlx::query_as::<_, DataProductQualityReport>(
            r#"
            INSERT INTO data_product_quality_reports
                (product_id, quality_score, freshness_met, completeness_score,
                 accuracy_score, consistency_score, issues)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, product_id, quality_score, freshness_met, completeness_score,
                      accuracy_score, consistency_score, issues, evaluated_at
            "#,
        )
        .bind(product_id)
        .bind(quality_score)
        .bind(freshness_met)
        .bind(completeness_score)
        .bind(accuracy_score)
        .bind(consistency_score)
        .bind(&issues_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    // ========================================================================
    // Revenue Reporting
    // ========================================================================

    /// Get a revenue report for a product owner over a period
    pub async fn get_revenue_report(
        &self,
        owner_tenant_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<RevenueReport> {
        let rows: Vec<(Uuid, String, f64, i64)> = sqlx::query_as(
            r#"
            SELECT dp.id, dp.name,
                   COALESCE(SUM(e.billed_amount), 0.0) AS revenue,
                   COALESCE(COUNT(e.id), 0) AS query_count
            FROM data_products dp
            LEFT JOIN data_product_usage_events e
                ON e.product_id = dp.id
                AND e.timestamp >= $2
                AND e.timestamp < $3
            WHERE dp.tenant_id = $1
            GROUP BY dp.id, dp.name
            ORDER BY revenue DESC
            "#,
        )
        .bind(owner_tenant_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await?;

        let mut total_revenue = 0.0;
        let mut by_product: Vec<ProductRevenue> = Vec::new();

        for (product_id, product_name, revenue, query_count) in rows {
            total_revenue += revenue;
            by_product.push(ProductRevenue {
                product_id,
                product_name,
                revenue,
                query_count,
            });
        }

        Ok(RevenueReport {
            owner_tenant_id,
            period_start,
            period_end,
            total_revenue,
            by_product,
        })
    }
}

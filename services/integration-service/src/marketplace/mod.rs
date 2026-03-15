pub mod api;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Enums
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListingTier {
    Free,
    Verified,
    Premium,
}

impl ListingTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Verified => "verified",
            Self::Premium => "premium",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "free" => Some(Self::Free),
            "verified" => Some(Self::Verified),
            "premium" => Some(Self::Premium),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    Free,
    UsageBased,
    FlatRate,
}

impl PricingModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::UsageBased => "usage_based",
            Self::FlatRate => "flat_rate",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "free" => Some(Self::Free),
            "usage_based" => Some(Self::UsageBased),
            "flat_rate" => Some(Self::FlatRate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListingStatus {
    Draft,
    PendingReview,
    Published,
    Suspended,
}

impl ListingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingReview => "pending_review",
            Self::Published => "published",
            Self::Suspended => "suspended",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "pending_review" => Some(Self::PendingReview),
            "published" => Some(Self::Published),
            "suspended" => Some(Self::Suspended),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallStatus {
    Active,
    Suspended,
    Uninstalled,
}

impl InstallStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Uninstalled => "uninstalled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListingSortBy {
    Installs,
    Rating,
    Newest,
}

// =============================================================================
// Data models
// =============================================================================

/// A connector listing in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceListing {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub publisher_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub icon_url: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub tier: String,
    pub pricing: String,
    pub price_per_call: Option<f64>,
    pub monthly_price: Option<f64>,
    pub revenue_share_pct: Option<f64>,
    pub install_count: i64,
    pub avg_rating: Option<f64>,
    pub rating_count: i64,
    pub status: String,
    pub sla_guaranteed: Option<f64>,
    pub documentation_url: Option<String>,
    pub source_repo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Tracks a connector installation by a tenant.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceInstall {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub tenant_id: Uuid,
    pub installed_by: Uuid,
    pub installed_at: DateTime<Utc>,
    pub uninstalled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub usage_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// A review/rating for a marketplace listing.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceReview {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub tenant_id: Uuid,
    pub reviewer_id: Uuid,
    pub rating: i32,
    pub title: String,
    pub body: String,
    pub helpful_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Analytics for a marketplace listing within a time period.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectorAnalytics {
    pub listing_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub installs: i64,
    pub uninstalls: i64,
    pub active_users: i64,
    pub api_calls: i64,
    pub error_count: i64,
    pub error_rate: f64,
    pub revenue_earned: f64,
    pub p50_latency_ms: f64,
    pub p99_latency_ms: f64,
}

// =============================================================================
// Request / Response types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListListingsFilter {
    pub category: Option<String>,
    pub tier: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PublishListingRequest {
    pub connector_id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub icon_url: Option<String>,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub tier: String,
    pub pricing: String,
    pub price_per_call: Option<f64>,
    pub monthly_price: Option<f64>,
    pub revenue_share_pct: Option<f64>,
    pub sla_guaranteed: Option<f64>,
    pub documentation_url: Option<String>,
    pub source_repo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddReviewRequest {
    pub rating: i32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct ListListingsResponse {
    pub listings: Vec<MarketplaceListing>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ListReviewsResponse {
    pub reviews: Vec<MarketplaceReview>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct RevenueSummary {
    pub publisher_id: Uuid,
    pub total_revenue: f64,
    pub total_installs: i64,
    pub total_api_calls: i64,
    pub listing_count: i64,
    pub periods: Vec<ConnectorAnalytics>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RejectListingRequest {
    pub reason: String,
}

// =============================================================================
// MarketplaceService
// =============================================================================

pub struct MarketplaceService {
    pool: PgPool,
}

impl MarketplaceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // -------------------------------------------------------------------------
    // Listing queries
    // -------------------------------------------------------------------------

    /// List marketplace listings with optional filters and sorting.
    pub async fn list_listings(
        &self,
        filter: &ListListingsFilter,
    ) -> anyhow::Result<ListListingsResponse> {
        let limit = filter.limit.unwrap_or(20).min(100);
        let offset = filter.offset.unwrap_or(0);

        let sort_clause = match filter.sort_by.as_deref() {
            Some("installs") => "ORDER BY install_count DESC",
            Some("rating") => "ORDER BY avg_rating DESC NULLS LAST",
            Some("newest") => "ORDER BY published_at DESC NULLS LAST",
            _ => "ORDER BY install_count DESC",
        };

        // Build dynamic WHERE clauses
        let mut conditions = vec!["status = 'published'".to_string()];
        let mut param_idx: usize = 1;
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref category) = filter.category {
            param_idx += 1;
            conditions.push(format!("category = ${}", param_idx));
            bind_values.push(category.clone());
        }

        if let Some(ref tier) = filter.tier {
            param_idx += 1;
            conditions.push(format!("tier = ${}", param_idx));
            bind_values.push(tier.clone());
        }

        if let Some(ref search) = filter.search {
            param_idx += 1;
            conditions.push(format!(
                "(name ILIKE '%' || ${p} || '%' OR description ILIKE '%' || ${p} || '%')",
                p = param_idx
            ));
            bind_values.push(search.clone());
        }

        let where_clause = conditions.join(" AND ");

        // Count query
        let count_sql = format!(
            "SELECT COUNT(*) FROM marketplace_listings WHERE {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for v in &bind_values {
            count_query = count_query.bind(v);
        }
        let total: i64 = count_query.fetch_one(&self.pool).await?;

        // Data query
        let data_sql = format!(
            r#"
            SELECT id, connector_id, publisher_id, tenant_id, name, description,
                   version, icon_url, category, tags, tier, pricing,
                   price_per_call, monthly_price, revenue_share_pct,
                   install_count, avg_rating, rating_count, status,
                   sla_guaranteed, documentation_url, source_repo_url,
                   created_at, updated_at, published_at
            FROM marketplace_listings
            WHERE {}
            {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, sort_clause, limit, offset
        );

        let mut data_query = sqlx::query_as::<_, MarketplaceListing>(&data_sql);
        for v in &bind_values {
            data_query = data_query.bind(v);
        }
        let listings: Vec<MarketplaceListing> = data_query.fetch_all(&self.pool).await?;

        Ok(ListListingsResponse { listings, total })
    }

    /// Get a single listing by ID.
    pub async fn get_listing(&self, listing_id: Uuid) -> anyhow::Result<MarketplaceListing> {
        let listing: MarketplaceListing = sqlx::query_as(
            r#"
            SELECT id, connector_id, publisher_id, tenant_id, name, description,
                   version, icon_url, category, tags, tier, pricing,
                   price_per_call, monthly_price, revenue_share_pct,
                   install_count, avg_rating, rating_count, status,
                   sla_guaranteed, documentation_url, source_repo_url,
                   created_at, updated_at, published_at
            FROM marketplace_listings
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Listing {} not found", listing_id))?;

        Ok(listing)
    }

    /// Publish a new listing as a draft.
    pub async fn publish_listing(
        &self,
        publisher_id: Uuid,
        tenant_id: Uuid,
        req: &PublishListingRequest,
    ) -> anyhow::Result<MarketplaceListing> {
        let tags = req.tags.clone().unwrap_or_default();

        let listing: MarketplaceListing = sqlx::query_as(
            r#"
            INSERT INTO marketplace_listings (
                connector_id, publisher_id, tenant_id, name, description,
                version, icon_url, category, tags, tier, pricing,
                price_per_call, monthly_price, revenue_share_pct,
                sla_guaranteed, documentation_url, source_repo_url,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, 'draft')
            RETURNING id, connector_id, publisher_id, tenant_id, name, description,
                      version, icon_url, category, tags, tier, pricing,
                      price_per_call, monthly_price, revenue_share_pct,
                      install_count, avg_rating, rating_count, status,
                      sla_guaranteed, documentation_url, source_repo_url,
                      created_at, updated_at, published_at
            "#,
        )
        .bind(req.connector_id)
        .bind(publisher_id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.version)
        .bind(&req.icon_url)
        .bind(&req.category)
        .bind(&tags)
        .bind(&req.tier)
        .bind(&req.pricing)
        .bind(req.price_per_call)
        .bind(req.monthly_price)
        .bind(req.revenue_share_pct)
        .bind(req.sla_guaranteed)
        .bind(&req.documentation_url)
        .bind(&req.source_repo_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(listing)
    }

    /// Submit a draft listing for review.
    pub async fn submit_for_review(
        &self,
        listing_id: Uuid,
        publisher_id: Uuid,
    ) -> anyhow::Result<MarketplaceListing> {
        let listing: MarketplaceListing = sqlx::query_as(
            r#"
            UPDATE marketplace_listings
            SET status = 'pending_review', updated_at = NOW()
            WHERE id = $1 AND publisher_id = $2 AND status = 'draft'
            RETURNING id, connector_id, publisher_id, tenant_id, name, description,
                      version, icon_url, category, tags, tier, pricing,
                      price_per_call, monthly_price, revenue_share_pct,
                      install_count, avg_rating, rating_count, status,
                      sla_guaranteed, documentation_url, source_repo_url,
                      created_at, updated_at, published_at
            "#,
        )
        .bind(listing_id)
        .bind(publisher_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Listing {} not found or not in draft status",
                listing_id
            )
        })?;

        Ok(listing)
    }

    /// Approve a listing that is pending review (admin action).
    pub async fn approve_listing(
        &self,
        listing_id: Uuid,
    ) -> anyhow::Result<MarketplaceListing> {
        let listing: MarketplaceListing = sqlx::query_as(
            r#"
            UPDATE marketplace_listings
            SET status = 'published', published_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'pending_review'
            RETURNING id, connector_id, publisher_id, tenant_id, name, description,
                      version, icon_url, category, tags, tier, pricing,
                      price_per_call, monthly_price, revenue_share_pct,
                      install_count, avg_rating, rating_count, status,
                      sla_guaranteed, documentation_url, source_repo_url,
                      created_at, updated_at, published_at
            "#,
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Listing {} not found or not pending review",
                listing_id
            )
        })?;

        Ok(listing)
    }

    /// Reject a listing that is pending review (admin action), returning it to draft.
    pub async fn reject_listing(
        &self,
        listing_id: Uuid,
        _reason: &str,
    ) -> anyhow::Result<MarketplaceListing> {
        let listing: MarketplaceListing = sqlx::query_as(
            r#"
            UPDATE marketplace_listings
            SET status = 'draft', updated_at = NOW()
            WHERE id = $1 AND status = 'pending_review'
            RETURNING id, connector_id, publisher_id, tenant_id, name, description,
                      version, icon_url, category, tags, tier, pricing,
                      price_per_call, monthly_price, revenue_share_pct,
                      install_count, avg_rating, rating_count, status,
                      sla_guaranteed, documentation_url, source_repo_url,
                      created_at, updated_at, published_at
            "#,
        )
        .bind(listing_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Listing {} not found or not pending review",
                listing_id
            )
        })?;

        // In a production system the rejection reason would be stored in
        // a separate review_decisions table or sent as a notification.
        tracing::info!(
            listing_id = %listing_id,
            reason = %_reason,
            "Listing rejected and returned to draft"
        );

        Ok(listing)
    }

    // -------------------------------------------------------------------------
    // Install / Uninstall
    // -------------------------------------------------------------------------

    /// Install a connector for a tenant.
    pub async fn install_connector(
        &self,
        listing_id: Uuid,
        tenant_id: Uuid,
        installed_by: Uuid,
    ) -> anyhow::Result<MarketplaceInstall> {
        // Verify the listing exists and is published
        let listing = self.get_listing(listing_id).await?;
        if listing.status != "published" {
            anyhow::bail!("Listing {} is not published", listing_id);
        }

        // Check for existing active install
        let existing: Option<MarketplaceInstall> = sqlx::query_as(
            r#"
            SELECT id, listing_id, tenant_id, installed_by, installed_at,
                   uninstalled_at, status, usage_count, last_used_at
            FROM marketplace_installs
            WHERE listing_id = $1 AND tenant_id = $2 AND status = 'active'
            "#,
        )
        .bind(listing_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            anyhow::bail!("Connector is already installed for this tenant");
        }

        let install: MarketplaceInstall = sqlx::query_as(
            r#"
            INSERT INTO marketplace_installs (listing_id, tenant_id, installed_by, status)
            VALUES ($1, $2, $3, 'active')
            RETURNING id, listing_id, tenant_id, installed_by, installed_at,
                      uninstalled_at, status, usage_count, last_used_at
            "#,
        )
        .bind(listing_id)
        .bind(tenant_id)
        .bind(installed_by)
        .fetch_one(&self.pool)
        .await?;

        // Increment install count
        sqlx::query(
            "UPDATE marketplace_listings SET install_count = install_count + 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(listing_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            listing_id = %listing_id,
            tenant_id = %tenant_id,
            "Connector installed"
        );

        Ok(install)
    }

    /// Uninstall a connector for a tenant.
    pub async fn uninstall_connector(
        &self,
        listing_id: Uuid,
        tenant_id: Uuid,
    ) -> anyhow::Result<MarketplaceInstall> {
        let install: MarketplaceInstall = sqlx::query_as(
            r#"
            UPDATE marketplace_installs
            SET status = 'uninstalled', uninstalled_at = NOW()
            WHERE listing_id = $1 AND tenant_id = $2 AND status = 'active'
            RETURNING id, listing_id, tenant_id, installed_by, installed_at,
                      uninstalled_at, status, usage_count, last_used_at
            "#,
        )
        .bind(listing_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("No active install found for listing {} and tenant {}", listing_id, tenant_id)
        })?;

        // Decrement install count (floor at 0)
        sqlx::query(
            r#"
            UPDATE marketplace_listings
            SET install_count = GREATEST(install_count - 1, 0), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .execute(&self.pool)
        .await?;

        tracing::info!(
            listing_id = %listing_id,
            tenant_id = %tenant_id,
            "Connector uninstalled"
        );

        Ok(install)
    }

    // -------------------------------------------------------------------------
    // Reviews
    // -------------------------------------------------------------------------

    /// Add a review for a listing.
    pub async fn add_review(
        &self,
        listing_id: Uuid,
        tenant_id: Uuid,
        reviewer_id: Uuid,
        req: &AddReviewRequest,
    ) -> anyhow::Result<MarketplaceReview> {
        if req.rating < 1 || req.rating > 5 {
            anyhow::bail!("Rating must be between 1 and 5");
        }

        // Verify the listing exists
        let _ = self.get_listing(listing_id).await?;

        // Check the tenant has the connector installed
        let installed: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM marketplace_installs WHERE listing_id = $1 AND tenant_id = $2 AND status = 'active'",
        )
        .bind(listing_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        if installed.is_none() {
            anyhow::bail!("You must have the connector installed to leave a review");
        }

        // Check for existing review from this reviewer
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM marketplace_reviews WHERE listing_id = $1 AND reviewer_id = $2",
        )
        .bind(listing_id)
        .bind(reviewer_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            anyhow::bail!("You have already reviewed this connector");
        }

        let review: MarketplaceReview = sqlx::query_as(
            r#"
            INSERT INTO marketplace_reviews (listing_id, tenant_id, reviewer_id, rating, title, body)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, listing_id, tenant_id, reviewer_id, rating, title, body,
                      helpful_count, created_at, updated_at
            "#,
        )
        .bind(listing_id)
        .bind(tenant_id)
        .bind(reviewer_id)
        .bind(req.rating)
        .bind(&req.title)
        .bind(&req.body)
        .fetch_one(&self.pool)
        .await?;

        // Update listing aggregate rating
        sqlx::query(
            r#"
            UPDATE marketplace_listings
            SET avg_rating = (
                    SELECT AVG(rating::float) FROM marketplace_reviews WHERE listing_id = $1
                ),
                rating_count = (
                    SELECT COUNT(*) FROM marketplace_reviews WHERE listing_id = $1
                ),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .execute(&self.pool)
        .await?;

        Ok(review)
    }

    /// List reviews for a listing.
    pub async fn list_reviews(
        &self,
        listing_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<ListReviewsResponse> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM marketplace_reviews WHERE listing_id = $1",
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        let reviews: Vec<MarketplaceReview> = sqlx::query_as(
            r#"
            SELECT id, listing_id, tenant_id, reviewer_id, rating, title, body,
                   helpful_count, created_at, updated_at
            FROM marketplace_reviews
            WHERE listing_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(listing_id)
        .bind(limit.min(100))
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(ListReviewsResponse { reviews, total })
    }

    // -------------------------------------------------------------------------
    // Analytics
    // -------------------------------------------------------------------------

    /// Get analytics for a publisher's listing.
    pub async fn get_publisher_analytics(
        &self,
        listing_id: Uuid,
        publisher_id: Uuid,
        period_start: Option<DateTime<Utc>>,
        period_end: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Vec<ConnectorAnalytics>> {
        // Verify ownership
        let listing = self.get_listing(listing_id).await?;
        if listing.publisher_id != publisher_id {
            anyhow::bail!("You do not own this listing");
        }

        let start = period_start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let end = period_end.unwrap_or_else(Utc::now);

        let analytics: Vec<ConnectorAnalytics> = sqlx::query_as(
            r#"
            SELECT listing_id, period_start, period_end, installs, uninstalls,
                   active_users, api_calls, error_count, error_rate,
                   revenue_earned, p50_latency_ms, p99_latency_ms
            FROM connector_analytics
            WHERE listing_id = $1 AND period_start >= $2 AND period_end <= $3
            ORDER BY period_start ASC
            "#,
        )
        .bind(listing_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(analytics)
    }

    /// Get revenue summary for a publisher across all their listings.
    pub async fn get_revenue_summary(
        &self,
        publisher_id: Uuid,
    ) -> anyhow::Result<RevenueSummary> {
        // Aggregate totals across all listings owned by this publisher
        let row: Option<(f64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(a.revenue_earned), 0.0) as total_revenue,
                COALESCE(SUM(l.install_count), 0) as total_installs,
                COALESCE(SUM(a.api_calls), 0) as total_api_calls,
                COUNT(DISTINCT l.id) as listing_count
            FROM marketplace_listings l
            LEFT JOIN connector_analytics a ON a.listing_id = l.id
            WHERE l.publisher_id = $1
            "#,
        )
        .bind(publisher_id)
        .fetch_optional(&self.pool)
        .await?;

        let (total_revenue, total_installs, total_api_calls, listing_count) =
            row.unwrap_or((0.0, 0, 0, 0));

        // Get recent period analytics for breakdown
        let periods: Vec<ConnectorAnalytics> = sqlx::query_as(
            r#"
            SELECT a.listing_id, a.period_start, a.period_end, a.installs, a.uninstalls,
                   a.active_users, a.api_calls, a.error_count, a.error_rate,
                   a.revenue_earned, a.p50_latency_ms, a.p99_latency_ms
            FROM connector_analytics a
            INNER JOIN marketplace_listings l ON l.id = a.listing_id
            WHERE l.publisher_id = $1
            ORDER BY a.period_start DESC
            LIMIT 30
            "#,
        )
        .bind(publisher_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(RevenueSummary {
            publisher_id,
            total_revenue,
            total_installs,
            total_api_calls,
            listing_count,
            periods,
        })
    }

    /// Get featured listings (top-rated published connectors).
    pub async fn get_featured_listings(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<MarketplaceListing>> {
        let listings: Vec<MarketplaceListing> = sqlx::query_as(
            r#"
            SELECT id, connector_id, publisher_id, tenant_id, name, description,
                   version, icon_url, category, tags, tier, pricing,
                   price_per_call, monthly_price, revenue_share_pct,
                   install_count, avg_rating, rating_count, status,
                   sla_guaranteed, documentation_url, source_repo_url,
                   created_at, updated_at, published_at
            FROM marketplace_listings
            WHERE status = 'published'
            ORDER BY
                CASE WHEN tier = 'verified' THEN 0 WHEN tier = 'premium' THEN 1 ELSE 2 END,
                (COALESCE(avg_rating, 0) * 0.6 + LN(install_count + 1) * 0.4) DESC
            LIMIT $1
            "#,
        )
        .bind(limit.min(50))
        .fetch_all(&self.pool)
        .await?;

        Ok(listings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_tier_roundtrip() {
        assert_eq!(ListingTier::from_str("free"), Some(ListingTier::Free));
        assert_eq!(ListingTier::from_str("verified"), Some(ListingTier::Verified));
        assert_eq!(ListingTier::from_str("premium"), Some(ListingTier::Premium));
        assert_eq!(ListingTier::from_str("unknown"), None);
    }

    #[test]
    fn pricing_model_roundtrip() {
        assert_eq!(PricingModel::from_str("free"), Some(PricingModel::Free));
        assert_eq!(PricingModel::from_str("usage_based"), Some(PricingModel::UsageBased));
        assert_eq!(PricingModel::from_str("flat_rate"), Some(PricingModel::FlatRate));
        assert_eq!(PricingModel::from_str("other"), None);
    }

    #[test]
    fn listing_status_roundtrip() {
        assert_eq!(ListingStatus::from_str("draft"), Some(ListingStatus::Draft));
        assert_eq!(ListingStatus::from_str("pending_review"), Some(ListingStatus::PendingReview));
        assert_eq!(ListingStatus::from_str("published"), Some(ListingStatus::Published));
        assert_eq!(ListingStatus::from_str("suspended"), Some(ListingStatus::Suspended));
        assert_eq!(ListingStatus::from_str("bad"), None);
    }

    #[test]
    fn tier_as_str() {
        assert_eq!(ListingTier::Free.as_str(), "free");
        assert_eq!(ListingTier::Verified.as_str(), "verified");
        assert_eq!(ListingTier::Premium.as_str(), "premium");
    }

    #[test]
    fn pricing_as_str() {
        assert_eq!(PricingModel::Free.as_str(), "free");
        assert_eq!(PricingModel::UsageBased.as_str(), "usage_based");
        assert_eq!(PricingModel::FlatRate.as_str(), "flat_rate");
    }

    #[test]
    fn install_status_as_str() {
        assert_eq!(InstallStatus::Active.as_str(), "active");
        assert_eq!(InstallStatus::Suspended.as_str(), "suspended");
        assert_eq!(InstallStatus::Uninstalled.as_str(), "uninstalled");
    }

    #[test]
    fn listing_status_as_str() {
        assert_eq!(ListingStatus::Draft.as_str(), "draft");
        assert_eq!(ListingStatus::PendingReview.as_str(), "pending_review");
        assert_eq!(ListingStatus::Published.as_str(), "published");
        assert_eq!(ListingStatus::Suspended.as_str(), "suspended");
    }

    #[test]
    fn add_review_request_deserializes() {
        let json = serde_json::json!({
            "rating": 4,
            "title": "Great connector",
            "body": "Works well with our setup"
        });
        let req: AddReviewRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.rating, 4);
        assert_eq!(req.title, "Great connector");
    }

    #[test]
    fn publish_listing_request_deserializes() {
        let json = serde_json::json!({
            "connector_id": "00000000-0000-0000-0000-000000000001",
            "name": "My Connector",
            "description": "A great connector",
            "version": "1.0.0",
            "category": "database",
            "tier": "free",
            "pricing": "free"
        });
        let req: PublishListingRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.name, "My Connector");
        assert_eq!(req.tier, "free");
        assert!(req.tags.is_none());
        assert!(req.price_per_call.is_none());
    }
}

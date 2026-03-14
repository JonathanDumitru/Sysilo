use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::products::{
    CreateProductRequest, DataProduct, DataProductQualityReport, DataProductSubscription,
    DataProductUsageEvent, ListProductsFilter, RecordUsageRequest, RevenueReport,
    UpdateProductRequest, UsageSummary,
};

// ============================================================================
// Query / Request Types
// ============================================================================

#[derive(Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

#[derive(Deserialize)]
pub struct ListProductsQuery {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
    pub pricing_model: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateProductApiRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub inner: CreateProductRequest,
}

#[derive(Deserialize)]
pub struct UpdateProductApiRequest {
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub inner: UpdateProductRequest,
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub subscriber_tenant_id: Uuid,
    pub subscriber_id: Uuid,
}

#[derive(Deserialize)]
pub struct RecordUsageApiRequest {
    #[serde(flatten)]
    pub inner: RecordUsageRequest,
}

#[derive(Deserialize)]
pub struct UsageSummaryQuery {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RevenueReportQuery {
    pub tenant_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

// ============================================================================
// Product Endpoints
// ============================================================================

/// POST /products - Create a new data product
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProductApiRequest>,
) -> Result<Json<DataProduct>, StatusCode> {
    match state.products.create_product(req.tenant_id, req.inner).await {
        Ok(product) => Ok(Json(product)),
        Err(e) => {
            tracing::error!("Failed to create data product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /products - List data products with optional filters
pub async fn list_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Vec<DataProduct>>, StatusCode> {
    let tags = query.tags.map(|t| {
        t.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>()
    });

    let filters = ListProductsFilter {
        tenant_id: query.tenant_id,
        status: query.status,
        tags,
        pricing_model: query.pricing_model,
    };

    match state.products.list_products(filters).await {
        Ok(products) => Ok(Json(products)),
        Err(e) => {
            tracing::error!("Failed to list data products: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /products/:id - Get a single data product
pub async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<DataProduct>, StatusCode> {
    match state.products.get_product(query.tenant_id, id).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get data product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// PUT /products/:id - Update a data product
pub async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductApiRequest>,
) -> Result<Json<DataProduct>, StatusCode> {
    match state.products.update_product(req.tenant_id, id, req.inner).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update data product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /products/:id/publish - Publish a data product
pub async fn publish_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<DataProduct>, StatusCode> {
    match state.products.publish_product(query.tenant_id, id).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to publish data product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /products/:id/deprecate - Deprecate a data product
pub async fn deprecate_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<DataProduct>, StatusCode> {
    match state.products.deprecate_product(query.tenant_id, id).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to deprecate data product: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Subscription Endpoints
// ============================================================================

/// POST /products/:id/subscribe - Subscribe to a data product
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<DataProductSubscription>, StatusCode> {
    match state
        .products
        .subscribe(product_id, req.subscriber_tenant_id, req.subscriber_id)
        .await
    {
        Ok(subscription) => Ok(Json(subscription)),
        Err(e) => {
            tracing::error!("Failed to subscribe to data product: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// POST /products/subscriptions/:id/unsubscribe - Cancel a subscription
pub async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<DataProductSubscription>, StatusCode> {
    match state.products.unsubscribe(subscription_id).await {
        Ok(Some(sub)) => Ok(Json(sub)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to unsubscribe: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /products/:id/subscriptions - List subscriptions for a product
pub async fn list_subscriptions(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<DataProductSubscription>>, StatusCode> {
    match state.products.list_subscriptions(product_id).await {
        Ok(subs) => Ok(Json(subs)),
        Err(e) => {
            tracing::error!("Failed to list subscriptions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /products/subscribed - List products a tenant is subscribed to
pub async fn list_subscriber_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<DataProduct>>, StatusCode> {
    match state.products.list_subscriber_products(query.tenant_id).await {
        Ok(products) => Ok(Json(products)),
        Err(e) => {
            tracing::error!("Failed to list subscriber products: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Usage Metering Endpoints
// ============================================================================

/// POST /products/subscriptions/:id/usage - Record a usage event
pub async fn record_usage(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
    Json(req): Json<RecordUsageApiRequest>,
) -> Result<Json<DataProductUsageEvent>, StatusCode> {
    match state.products.record_usage(subscription_id, req.inner).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => {
            tracing::error!("Failed to record usage: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// GET /products/subscriptions/:id/usage/summary - Get usage summary
pub async fn get_usage_summary(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
    Query(query): Query<UsageSummaryQuery>,
) -> Result<Json<UsageSummary>, StatusCode> {
    match state
        .products
        .get_usage_summary(subscription_id, query.period_start, query.period_end)
        .await
    {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => {
            tracing::error!("Failed to get usage summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Quality Monitoring Endpoints
// ============================================================================

/// POST /products/:id/quality/evaluate - Evaluate quality for a data product
pub async fn evaluate_quality(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<DataProductQualityReport>, StatusCode> {
    match state.products.evaluate_quality(product_id).await {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            tracing::error!("Failed to evaluate product quality: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Revenue Reporting Endpoints
// ============================================================================

/// GET /products/revenue - Get revenue report for an owner tenant
pub async fn get_revenue_report(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RevenueReportQuery>,
) -> Result<Json<RevenueReport>, StatusCode> {
    match state
        .products
        .get_revenue_report(query.tenant_id, query.period_start, query.period_end)
        .await
    {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            tracing::error!("Failed to get revenue report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

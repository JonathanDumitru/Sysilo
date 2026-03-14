use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::ApiError;
use crate::marketplace::{
    AddReviewRequest, AnalyticsQuery, ConnectorAnalytics, ListListingsFilter,
    ListListingsResponse, ListReviewsResponse, MarketplaceInstall, MarketplaceListing,
    MarketplaceReview, PublishListingRequest, RejectListingRequest, RevenueSummary,
};
use crate::middleware::TenantContext;
use crate::AppState;

// =============================================================================
// Helper
// =============================================================================

fn marketplace_error(error: &str, message: &str, status: StatusCode) -> ApiError {
    ApiError {
        error: error.to_string(),
        message: message.to_string(),
        status: Some(status),
        resource: None,
        current: None,
        limit: None,
        plan: None,
    }
}

fn service(state: &Arc<AppState>) -> anyhow::Result<&crate::marketplace::MarketplaceService> {
    state
        .marketplace
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Marketplace service not available"))
}

// =============================================================================
// Listing endpoints
// =============================================================================

/// GET /marketplace/listings
pub async fn list_listings(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<ListListingsFilter>,
) -> Result<Json<ListListingsResponse>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let response = svc
        .list_listings(&filter)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(response))
}

/// GET /marketplace/listings/featured
pub async fn get_featured_listings(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeaturedQuery>,
) -> Result<Json<Vec<MarketplaceListing>>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let limit = params.limit.unwrap_or(10);
    let listings = svc
        .get_featured_listings(limit)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(listings))
}

/// GET /marketplace/listings/:id
pub async fn get_listing(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketplaceListing>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let listing = svc.get_listing(id).await.map_err(|e| {
        marketplace_error("not_found", &e.to_string(), StatusCode::NOT_FOUND)
    })?;

    Ok(Json(listing))
}

/// POST /marketplace/listings
pub async fn publish_listing(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<PublishListingRequest>,
) -> Result<(StatusCode, Json<MarketplaceListing>), ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let publisher_id = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required to publish a listing",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let listing = svc
        .publish_listing(publisher_id, tenant.tenant_id, &req)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    tracing::info!(
        listing_id = %listing.id,
        publisher_id = %publisher_id,
        name = %listing.name,
        "Marketplace listing created as draft"
    );

    Ok((StatusCode::CREATED, Json(listing)))
}

/// POST /marketplace/listings/:id/submit
pub async fn submit_for_review(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketplaceListing>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let publisher_id = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let listing = svc
        .submit_for_review(id, publisher_id)
        .await
        .map_err(|e| {
            marketplace_error("submission_failed", &e.to_string(), StatusCode::CONFLICT)
        })?;

    tracing::info!(
        listing_id = %id,
        "Listing submitted for review"
    );

    Ok(Json(listing))
}

/// POST /marketplace/listings/:id/approve
pub async fn approve_listing(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketplaceListing>, ApiError> {
    // Only admins can approve
    if tenant.user_role.as_deref() != Some("admin") {
        return Err(marketplace_error(
            "forbidden",
            "Only admins can approve listings",
            StatusCode::FORBIDDEN,
        ));
    }

    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let listing = svc.approve_listing(id).await.map_err(|e| {
        marketplace_error("approval_failed", &e.to_string(), StatusCode::CONFLICT)
    })?;

    tracing::info!(
        listing_id = %id,
        approved_by = ?tenant.user_id,
        "Listing approved and published"
    );

    Ok(Json(listing))
}

/// POST /marketplace/listings/:id/reject
pub async fn reject_listing(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<RejectListingRequest>,
) -> Result<Json<MarketplaceListing>, ApiError> {
    // Only admins can reject
    if tenant.user_role.as_deref() != Some("admin") {
        return Err(marketplace_error(
            "forbidden",
            "Only admins can reject listings",
            StatusCode::FORBIDDEN,
        ));
    }

    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let listing = svc.reject_listing(id, &req.reason).await.map_err(|e| {
        marketplace_error("rejection_failed", &e.to_string(), StatusCode::CONFLICT)
    })?;

    tracing::info!(
        listing_id = %id,
        rejected_by = ?tenant.user_id,
        reason = %req.reason,
        "Listing rejected"
    );

    Ok(Json(listing))
}

// =============================================================================
// Install / Uninstall endpoints
// =============================================================================

/// POST /marketplace/listings/:id/install
pub async fn install_connector(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<MarketplaceInstall>), ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let installed_by = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required to install a connector",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let install = svc
        .install_connector(id, tenant.tenant_id, installed_by)
        .await
        .map_err(|e| {
            marketplace_error("install_failed", &e.to_string(), StatusCode::CONFLICT)
        })?;

    Ok((StatusCode::CREATED, Json(install)))
}

/// POST /marketplace/listings/:id/uninstall
pub async fn uninstall_connector(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketplaceInstall>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let install = svc
        .uninstall_connector(id, tenant.tenant_id)
        .await
        .map_err(|e| {
            marketplace_error("uninstall_failed", &e.to_string(), StatusCode::NOT_FOUND)
        })?;

    Ok(Json(install))
}

// =============================================================================
// Review endpoints
// =============================================================================

/// POST /marketplace/listings/:id/reviews
pub async fn add_review(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddReviewRequest>,
) -> Result<(StatusCode, Json<MarketplaceReview>), ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let reviewer_id = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required to leave a review",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let review = svc
        .add_review(id, tenant.tenant_id, reviewer_id, &req)
        .await
        .map_err(|e| {
            marketplace_error("review_failed", &e.to_string(), StatusCode::BAD_REQUEST)
        })?;

    Ok((StatusCode::CREATED, Json(review)))
}

/// GET /marketplace/listings/:id/reviews
pub async fn list_reviews(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(params): Query<ReviewsQuery>,
) -> Result<Json<ListReviewsResponse>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let response = svc
        .list_reviews(id, limit, offset)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(response))
}

// =============================================================================
// Analytics endpoints
// =============================================================================

/// GET /marketplace/listings/:id/analytics
pub async fn get_publisher_analytics(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<Vec<ConnectorAnalytics>>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let publisher_id = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let analytics = svc
        .get_publisher_analytics(id, publisher_id, params.period_start, params.period_end)
        .await
        .map_err(|e| {
            marketplace_error("analytics_error", &e.to_string(), StatusCode::FORBIDDEN)
        })?;

    Ok(Json(analytics))
}

/// GET /marketplace/revenue
pub async fn get_revenue_summary(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<RevenueSummary>, ApiError> {
    let svc = service(&state).map_err(|e| {
        ApiError::internal("marketplace_unavailable", e.to_string())
    })?;

    let publisher_id = tenant.user_id.ok_or_else(|| {
        marketplace_error(
            "authentication_required",
            "User ID is required",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    let summary = svc
        .get_revenue_summary(publisher_id)
        .await
        .map_err(|e| ApiError::internal("database_error", e.to_string()))?;

    Ok(Json(summary))
}

// =============================================================================
// Query parameter types
// =============================================================================

#[derive(Debug, serde::Deserialize)]
pub struct ReviewsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FeaturedQuery {
    pub limit: Option<i64>,
}

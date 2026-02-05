use axum::{
    body::Body,
    extract::Request,
    http::{header::HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

/// Headers used for passing tenant context from API Gateway
const TENANT_ID_HEADER: &str = "x-tenant-id";
const USER_ID_HEADER: &str = "x-user-id";
const USER_ROLE_HEADER: &str = "x-user-role";

/// Tenant context extracted from request headers
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
}

/// Error response for missing tenant context
#[derive(Debug, Serialize)]
pub struct TenantContextError {
    pub error: String,
    pub message: String,
}

impl IntoResponse for TenantContextError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, axum::Json(self)).into_response()
    }
}

/// Extract tenant context from headers
pub fn extract_tenant_context(headers: &HeaderMap) -> Result<TenantContext, TenantContextError> {
    // Extract tenant ID (required)
    let tenant_id = headers
        .get(TENANT_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| TenantContextError {
            error: "missing_tenant_context".to_string(),
            message: format!("Missing or invalid {} header", TENANT_ID_HEADER),
        })?;

    // Extract user ID (optional)
    let user_id = headers
        .get(USER_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    // Extract user role (optional)
    let user_role = headers
        .get(USER_ROLE_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    Ok(TenantContext {
        tenant_id,
        user_id,
        user_role,
    })
}

/// Extension trait for extracting TenantContext from axum extensions
pub trait TenantContextExt {
    fn tenant_context(&self) -> Option<&TenantContext>;
}

impl TenantContextExt for axum::extract::Extension<TenantContext> {
    fn tenant_context(&self) -> Option<&TenantContext> {
        Some(&self.0)
    }
}

/// Middleware that extracts tenant context from headers and adds it to request extensions
pub async fn tenant_context_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, TenantContextError> {
    let context = extract_tenant_context(request.headers())?;
    request.extensions_mut().insert(context);
    Ok(next.run(request).await)
}

/// For development/testing: middleware that uses a default tenant if not provided
pub async fn optional_tenant_context_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let context = extract_tenant_context(request.headers()).unwrap_or_else(|_| {
        // Default tenant for development
        TenantContext {
            tenant_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            user_id: None,
            user_role: Some("admin".to_string()),
        }
    });
    request.extensions_mut().insert(context);
    next.run(request).await
}

use axum::{
    body::Body,
    extract::Request,
    http::{header::HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Headers used for passing tenant context from API Gateway
const TENANT_ID_HEADER: &str = "x-tenant-id";
const USER_ID_HEADER: &str = "x-user-id";
const USER_ROLE_HEADER: &str = "x-user-role";
const PLAN_NAME_HEADER: &str = "x-plan-name";
const PLAN_LIMITS_HEADER: &str = "x-plan-limits";
const ENVIRONMENT_HEADER: &str = "x-environment";

/// Plan limits passed from API Gateway
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlanLimits {
    #[serde(default = "default_unlimited")]
    pub max_integrations: i64,
    #[serde(default = "default_unlimited")]
    pub max_connections: i64,
    #[serde(default = "default_unlimited")]
    pub max_playbooks: i64,
    #[serde(default = "default_unlimited")]
    pub max_runs_per_month: i64,
    #[serde(default = "default_unlimited")]
    pub max_agents: i64,
    #[serde(default = "default_unlimited")]
    pub max_users: i64,
    #[serde(default)]
    pub audit_retention_days: i64,
}

fn default_unlimited() -> i64 {
    -1
}

impl PlanLimits {
    pub fn is_unlimited(&self, val: i64) -> bool {
        val < 0
    }
}

/// Tenant context extracted from request headers
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_role: Option<String>,
    pub environment: String,
    pub plan_name: String,
    pub plan_limits: PlanLimits,
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

    let environment = headers
        .get(ENVIRONMENT_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .map(str::to_lowercase)
        .filter(|env| matches!(env.as_str(), "dev" | "staging" | "prod"))
        .ok_or_else(|| TenantContextError {
            error: "missing_environment_context".to_string(),
            message: format!("Missing or invalid {} header", ENVIRONMENT_HEADER),
        })?;

    // Extract plan name (optional, defaults to "trial")
    let plan_name = headers
        .get(PLAN_NAME_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("trial")
        .to_string();

    // Extract plan limits (optional, defaults to unlimited)
    let plan_limits = headers
        .get(PLAN_LIMITS_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| serde_json::from_str::<PlanLimits>(s).ok())
        .unwrap_or_default();

    Ok(TenantContext {
        tenant_id,
        user_id,
        user_role,
        environment,
        plan_name,
        plan_limits,
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
        // Default tenant for development (unlimited plan)
        TenantContext {
            tenant_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            user_id: None,
            user_role: Some("admin".to_string()),
            environment: "dev".to_string(),
            plan_name: "enterprise".to_string(),
            plan_limits: PlanLimits::default(),
        }
    });
    request.extensions_mut().insert(context);
    next.run(request).await
}

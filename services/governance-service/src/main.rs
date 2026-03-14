use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod approvals;
mod audit;
mod compliance;
mod compliance_api;
mod federated;
mod kafka;
mod policies;
mod standards;

use crate::approvals::ApprovalsService;
use crate::audit::AuditService;
use crate::compliance::ComplianceService;
use crate::compliance_api::ComplianceApiService;
use crate::compliance_api::api as compliance_api_handlers;
use crate::federated::FederatedGovernanceService;
use crate::kafka::{GovernanceEventProducer, KafkaConfig};
use crate::policies::PoliciesService;
use crate::standards::StandardsService;

/// Application state shared across handlers
pub struct AppState {
    pub policies: PoliciesService,
    pub standards: StandardsService,
    pub approvals: ApprovalsService,
    pub audit: AuditService,
    pub compliance: ComplianceService,
    pub compliance_api: Option<ComplianceApiService>,
    pub federated: FederatedGovernanceService,
    pub events: Option<GovernanceEventProducer>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "governance_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting Sysilo Governance Center Service");

    // Load configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sysilo:sysilo_dev@localhost:5432/sysilo".to_string());

    // Initialize Kafka event producer (optional - continues without if unavailable)
    let kafka_config = KafkaConfig::from_env();
    let events = GovernanceEventProducer::try_new(&kafka_config);
    if events.is_some() {
        info!("Kafka event publishing enabled");
    } else {
        info!("Kafka event publishing disabled - running in local mode");
    }

    // Initialize services
    let policies = PoliciesService::new(&database_url).await?;
    let standards = StandardsService::new(&database_url).await?;
    let approvals = ApprovalsService::new(&database_url).await?;
    let audit = AuditService::new(&database_url).await?;
    let compliance = ComplianceService::new(&database_url).await?;
    let federated = FederatedGovernanceService::new(&database_url).await?;

    // Initialize Compliance-as-a-Product API service
    let compliance_api = match ComplianceApiService::new(&database_url).await {
        Ok(svc) => {
            info!("Compliance API service initialized");
            Some(svc)
        }
        Err(e) => {
            tracing::warn!("Compliance API service not available: {}", e);
            None
        }
    };

    let state = Arc::new(AppState {
        policies,
        standards,
        approvals,
        audit,
        compliance,
        compliance_api,
        federated,
        events,
    });

    // Build router
    let app = Router::new()
        // Health endpoints
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Standards endpoints
        .route("/standards", get(api::list_standards))
        .route("/standards", post(api::create_standard))
        .route("/standards/:id", get(api::get_standard))
        .route("/standards/:id", put(api::update_standard))
        .route("/standards/:id", delete(api::delete_standard))
        // Policies endpoints
        .route("/policies", get(api::list_policies))
        .route("/policies", post(api::create_policy))
        .route("/policies/:id", get(api::get_policy))
        .route("/policies/:id", put(api::update_policy))
        .route("/policies/:id", delete(api::delete_policy))
        .route("/policies/evaluate", post(api::evaluate_policies))
        .route("/policies/violations", get(api::list_violations))
        .route("/policies/violations/:id/resolve", post(api::resolve_violation))
        // Approvals endpoints
        .route("/approvals/workflows", get(api::list_workflows))
        .route("/approvals/workflows", post(api::create_workflow))
        .route("/approvals/workflows/:id", get(api::get_workflow))
        .route("/approvals/workflows/:id", put(api::update_workflow))
        .route("/approvals/requests", get(api::list_requests))
        .route("/approvals/requests", post(api::create_request))
        .route("/approvals/requests/:id", get(api::get_request))
        .route("/approvals/requests/:id/decide", post(api::decide_request))
        // Audit endpoints
        .route("/audit", get(api::query_audit_log))
        .route("/audit/export", get(api::export_audit_log))
        // Compliance endpoints
        .route("/compliance/frameworks", get(api::list_frameworks))
        .route("/compliance/status", get(api::get_compliance_status))
        .route("/compliance/assess", post(api::run_assessment))
        .route("/compliance/report/:framework", get(api::generate_report))
        // Federated Governance — Domain endpoints
        .route("/domains", post(api::create_domain))
        .route("/domains", get(api::list_domains))
        .route("/domains/hierarchy", get(api::get_domain_hierarchy))
        .route("/domains/:id", get(api::get_domain))
        .route("/domains/:id", put(api::update_domain))
        .route("/domains/:id", delete(api::delete_domain))
        // Federated Governance — Domain Policy endpoints
        .route("/domains/:id/policies", post(api::create_domain_policy))
        .route("/domains/:id/policies", get(api::list_domain_policies))
        .route("/domains/policies/:id", put(api::update_domain_policy))
        .route("/domains/policies/:id", delete(api::delete_domain_policy))
        // Federated Governance — Inheritance & Evaluation
        .route("/governance/inheritance", get(api::get_inheritance_chain))
        .route("/governance/evaluate-federated", post(api::evaluate_federated))
        // Federated Governance — Health Scores
        .route("/governance/health", get(api::get_all_health_scores))
        .route("/governance/health/:domain_id", get(api::get_domain_health))
        .route("/governance/health/:domain_id/trends", get(api::get_health_trends))
        // Compliance-as-a-Product — Governance API
        .route("/governance/api/evaluate", post(compliance_api_handlers::evaluate_policy))
        .route("/governance/api/scores", get(compliance_api_handlers::get_compliance_scores))
        .route("/governance/api/scores/calculate", post(compliance_api_handlers::calculate_score))
        .route("/governance/api/reports", get(compliance_api_handlers::list_reports))
        .route("/governance/api/reports/generate", post(compliance_api_handlers::generate_report))
        .route("/governance/api/decisions", get(compliance_api_handlers::get_decision_history))
        .route("/governance/api/decisions/analytics", get(compliance_api_handlers::get_decision_analytics))
        .route("/governance/api/regulatory-changes", get(compliance_api_handlers::list_regulatory_changes))
        .route("/governance/api/regulatory-changes", post(compliance_api_handlers::record_regulatory_change))
        .route("/governance/api/regulatory-changes/:id/approve", post(compliance_api_handlers::approve_regulatory_change))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = std::env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8086".to_string())
        .parse()?;

    info!("Governance Center Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Governance Center Service shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}

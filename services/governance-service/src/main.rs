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
mod kafka;
mod policies;
mod standards;

use crate::approvals::ApprovalsService;
use crate::audit::AuditService;
use crate::compliance::ComplianceService;
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

    let state = Arc::new(AppState {
        policies,
        standards,
        approvals,
        audit,
        compliance,
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

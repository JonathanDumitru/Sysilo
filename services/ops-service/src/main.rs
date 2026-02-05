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
mod metrics;
mod alerts;
mod incidents;
mod notifications;

use crate::metrics::MetricsService;
use crate::alerts::AlertsService;
use crate::incidents::IncidentsService;
use crate::notifications::NotificationService;

/// Application state shared across handlers
pub struct AppState {
    pub metrics: MetricsService,
    pub alerts: AlertsService,
    pub incidents: IncidentsService,
    pub notifications: NotificationService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ops_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting Sysilo Operations Center Service");

    // Load configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sysilo:sysilo_dev@localhost:5432/sysilo".to_string());

    let kafka_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());

    // Initialize services
    let metrics = MetricsService::new(&database_url).await?;
    let alerts = AlertsService::new(&database_url).await?;
    let incidents = IncidentsService::new(&database_url).await?;
    let notifications = NotificationService::new(&database_url).await?;

    let state = Arc::new(AppState {
        metrics,
        alerts,
        incidents,
        notifications,
    });

    // Build router
    let app = Router::new()
        // Health endpoints
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Metrics endpoints
        .route("/metrics", get(api::query_metrics))
        .route("/metrics", post(api::ingest_metrics))
        .route("/metrics/aggregations", get(api::get_aggregations))
        // Alert rules endpoints
        .route("/alerts/rules", get(api::list_alert_rules))
        .route("/alerts/rules", post(api::create_alert_rule))
        .route("/alerts/rules/:id", get(api::get_alert_rule))
        .route("/alerts/rules/:id", put(api::update_alert_rule))
        .route("/alerts/rules/:id", delete(api::delete_alert_rule))
        // Alert instances endpoints
        .route("/alerts/instances", get(api::list_alert_instances))
        .route("/alerts/instances/:id/ack", post(api::acknowledge_alert))
        .route("/alerts/instances/:id/resolve", post(api::resolve_alert))
        // Incidents endpoints
        .route("/incidents", get(api::list_incidents))
        .route("/incidents", post(api::create_incident))
        .route("/incidents/:id", get(api::get_incident))
        .route("/incidents/:id", put(api::update_incident))
        .route("/incidents/:id/events", get(api::get_incident_events))
        .route("/incidents/:id/events", post(api::add_incident_event))
        .route("/incidents/:id/resolve", post(api::resolve_incident))
        // Notification channels endpoints
        .route("/notifications/channels", get(api::list_channels))
        .route("/notifications/channels", post(api::create_channel))
        .route("/notifications/channels/:id", get(api::get_channel))
        .route("/notifications/channels/:id", put(api::update_channel))
        .route("/notifications/channels/:id", delete(api::delete_channel))
        .route("/notifications/test/:id", post(api::test_channel))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = std::env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8085".to_string())
        .parse()?;

    info!("Operations Center Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Operations Center Service shutdown complete");
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

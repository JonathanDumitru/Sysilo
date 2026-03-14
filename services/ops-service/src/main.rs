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
mod evaluator;
mod incidents;
mod notifications;
mod digital_twin;

use crate::metrics::MetricsService;
use crate::alerts::AlertsService;
use crate::evaluator::AlertEvaluator;
use crate::incidents::IncidentsService;
use crate::notifications::NotificationService;
use crate::digital_twin::{DigitalTwinService, TwinLearner};

/// Application state shared across handlers
pub struct AppState {
    pub metrics: Arc<MetricsService>,
    pub alerts: Arc<AlertsService>,
    pub incidents: Arc<IncidentsService>,
    pub notifications: Arc<NotificationService>,
    pub digital_twins: Arc<DigitalTwinService>,
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
    let metrics = Arc::new(MetricsService::new(&database_url).await?);
    let alerts = Arc::new(AlertsService::new(&database_url).await?);
    let incidents = Arc::new(IncidentsService::new(&database_url).await?);
    let notifications = Arc::new(NotificationService::new(&database_url).await?);
    let digital_twins = Arc::new(DigitalTwinService::new(&database_url).await?);

    // Start the alert evaluation engine
    let eval_interval: u64 = std::env::var("ALERT_EVAL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    let evaluator = AlertEvaluator::new(
        Arc::clone(&alerts),
        Arc::clone(&metrics),
        Arc::clone(&notifications),
        eval_interval,
    );
    let _evaluator_handle = evaluator.start();
    info!("Alert evaluator started with {}s interval", eval_interval);

    // Start the digital twin background learner
    let twin_learner_interval: u64 = std::env::var("TWIN_LEARNER_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300); // 5 minutes default

    let twin_learner = TwinLearner::new(
        Arc::clone(&digital_twins),
        twin_learner_interval,
    );
    let _twin_learner_handle = twin_learner.start();
    info!("Digital twin learner started with {}s interval", twin_learner_interval);

    let state = Arc::new(AppState {
        metrics,
        alerts,
        incidents,
        notifications,
        digital_twins,
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
        // Digital Twin endpoints
        .route("/twins", get(api::list_twins))
        .route("/twins", post(api::create_twin))
        .route("/twins/anomalies", get(api::list_twin_anomalies))
        .route("/twins/predictions", get(api::list_twin_predictions))
        .route("/twins/fleet-health", get(api::fleet_health))
        .route("/twins/:integration_id", get(api::get_twin))
        .route("/twins/:integration_id/learn", post(api::learn_twin_baseline))
        .route("/twins/:integration_id/update", post(api::update_twin_state))
        .route("/twins/:integration_id/simulate", post(api::simulate_twin_change))
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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod connections;
mod consumer;
mod engine;
mod healing;
mod kafka;
mod middleware;
mod playbooks;
mod storage;

use crate::config::Config;
use crate::engine::Engine;
use crate::connections::api as connections_api;
use crate::healing::api as healing_api;
use crate::healing::HealingService;
use crate::playbooks::api as playbooks_api;
use crate::storage::Storage;

/// Application state shared across handlers
pub struct AppState {
    pub config: Config,
    pub storage: Storage,
    pub engine: Engine,
    pub healing: Option<Arc<HealingService>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "integration_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting Sysilo Integration Service");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize storage
    let storage = Storage::new(&config.database).await?;
    info!("Database connection established");

    // Initialize execution engine
    let engine = Engine::new(config.clone());
    info!("Execution engine initialized");

    // Start result consumer in background
    if config.consumer.enabled {
        let consumer_config = consumer::ConsumerConfig {
            bootstrap_servers: config.consumer.bootstrap_servers.clone(),
            group_id: config.consumer.group_id.clone(),
            asset_service_url: config.consumer.asset_service_url.clone(),
        };

        // Create separate storage and producer for the consumer
        let consumer_storage = match Storage::new(&config.database).await {
            Ok(s) => Some(Arc::new(s)),
            Err(e) => {
                tracing::warn!("Consumer storage not available: {}", e);
                None
            }
        };

        let consumer_producer = {
            let kafka_cfg = crate::kafka::KafkaConfig {
                bootstrap_servers: config.kafka.brokers.clone(),
                client_id: "integration-service-consumer".to_string(),
                acks: "all".to_string(),
                retries: 3,
                linger_ms: 5,
            };
            match crate::kafka::TaskProducer::new(&kafka_cfg) {
                Ok(p) => Some(Arc::new(p)),
                Err(e) => {
                    tracing::warn!("Consumer Kafka producer not available: {}", e);
                    None
                }
            }
        };

        tokio::spawn(async move {
            let consumer = match consumer::ResultConsumer::new(
                &consumer_config,
                consumer_storage,
                consumer_producer,
            ) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to create result consumer: {}", e);
                    return;
                }
            };

            if let Err(e) = consumer.run().await {
                tracing::error!("Result consumer error: {}", e);
            }
        });

        info!("Result consumer started");
    }

    // Initialize healing service
    let healing_config = healing::HealingConfig {
        enabled: std::env::var("HEALING_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        auto_approve_low_risk: std::env::var("HEALING_AUTO_APPROVE_LOW_RISK")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        max_auto_retries: std::env::var("HEALING_MAX_AUTO_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3),
        ai_service_url: std::env::var("AI_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8090".to_string()),
        governance_service_url: std::env::var("GOVERNANCE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8086".to_string()),
    };

    let healing = match HealingService::new(&config.database.url, healing_config).await {
        Ok(service) => {
            info!("Healing service initialized");
            let service = Arc::new(service);
            // Start background approval checker
            healing::spawn_approval_checker(Arc::clone(&service));
            info!("Healing approval checker started");
            Some(service)
        }
        Err(e) => {
            tracing::warn!("Healing service not available: {}", e);
            None
        }
    };

    // Create application state
    let state = Arc::new(AppState {
        config: config.clone(),
        storage,
        engine,
        healing,
    });

    // Routes requiring tenant context
    let protected_routes = Router::new()
        // Integration endpoints
        .route("/integrations", get(api::list_integrations))
        .route("/integrations", post(api::create_integration))
        .route("/integrations/:id", get(api::get_integration))
        .route("/integrations/:id/run", post(api::run_integration))
        // Integration run endpoints
        .route("/runs/:id", get(api::get_run))
        .route("/runs/:id/cancel", post(api::cancel_run))
        // Playbook endpoints
        .route("/playbooks", get(playbooks_api::list_playbooks))
        .route("/playbooks", post(playbooks_api::create_playbook))
        .route("/playbooks/:id", get(playbooks_api::get_playbook))
        .route("/playbooks/:id", put(playbooks_api::update_playbook))
        .route("/playbooks/:id", delete(playbooks_api::delete_playbook))
        .route("/playbooks/:id/run", post(playbooks_api::run_playbook))
        .route("/playbooks/:id/runs", get(playbooks_api::list_playbook_runs))
        // Playbook run endpoints
        .route("/playbook-runs/:id", get(playbooks_api::get_playbook_run))
        .route("/playbook-runs/:id/approve", post(playbooks_api::approve_run))
        .route("/playbook-runs/:id/reject", post(playbooks_api::reject_run))
        // Discovery endpoints
        .route("/discovery/run", post(api::run_discovery))
        .route("/discovery/runs", get(api::list_discovery_runs))
        // Development/mock endpoints (for local testing without Kafka)
        .route("/dev/mock-discovery", post(api::mock_discovery_result))
        // Connection endpoints
        .route("/connections", get(connections_api::list_connections))
        .route("/connections", post(connections_api::create_connection))
        .route("/connections/:id", get(connections_api::get_connection))
        .route("/connections/:id", put(connections_api::update_connection))
        .route("/connections/:id", delete(connections_api::delete_connection))
        .route("/connections/:id/test", post(connections_api::test_connection))
        // Tenant context middleware: strict fail-closed context extraction.
        .layer(axum_middleware::from_fn(
            middleware::tenant_context_middleware,
        ));

    // Build router
    let app = Router::new()
        // Health endpoints (no auth required)
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Merge protected routes
        .merge(protected_routes)
        // Global middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = config.server.address.parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Integration Service shutdown complete");
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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod engine;
mod kafka;
mod middleware;
mod storage;

use crate::config::Config;
use crate::engine::Engine;
use crate::storage::Storage;

/// Application state shared across handlers
pub struct AppState {
    pub config: Config,
    pub storage: Storage,
    pub engine: Engine,
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

    // Create application state
    let state = Arc::new(AppState {
        config: config.clone(),
        storage,
        engine,
    });

    // Routes requiring tenant context
    let protected_routes = Router::new()
        // Integration endpoints
        .route("/integrations", get(api::list_integrations))
        .route("/integrations", post(api::create_integration))
        .route("/integrations/:id", get(api::get_integration))
        .route("/integrations/:id/run", post(api::run_integration))
        // Run endpoints
        .route("/runs/:id", get(api::get_run))
        .route("/runs/:id/cancel", post(api::cancel_run))
        // Tenant context middleware (uses optional for dev - change to strict in production)
        .layer(axum_middleware::from_fn(
            middleware::optional_tenant_context_middleware,
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

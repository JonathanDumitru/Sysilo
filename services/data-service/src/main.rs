use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod catalog;
mod lineage;
mod quality;
mod ingestion;

use crate::catalog::CatalogService;
use crate::lineage::LineageService;
use crate::quality::QualityService;

/// Application state shared across handlers
pub struct AppState {
    pub catalog: CatalogService,
    pub lineage: LineageService,
    pub quality: QualityService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "data_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting Sysilo Data Hub Service");

    // Initialize services
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sysilo:sysilo_dev@localhost:5432/sysilo".to_string());

    // Neo4j connection settings
    let neo4j_uri = std::env::var("NEO4J_URI")
        .unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let neo4j_user = std::env::var("NEO4J_USER")
        .unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password = std::env::var("NEO4J_PASSWORD")
        .unwrap_or_else(|_| "neo4j_dev".to_string());

    let catalog = CatalogService::new(&database_url).await?;
    let lineage = LineageService::new(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    let quality = QualityService::new(&database_url).await?;

    let state = Arc::new(AppState {
        catalog,
        lineage,
        quality,
    });

    // Build router
    let app = Router::new()
        // Health endpoints
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Catalog endpoints
        .route("/catalog/entities", get(api::list_entities))
        .route("/catalog/entities", post(api::create_entity))
        .route("/catalog/entities/:id", get(api::get_entity))
        .route("/catalog/entities/:id", delete(api::delete_entity))
        .route("/catalog/entities/:id/schema", get(api::get_entity_schema))
        // Lineage endpoints
        .route("/lineage", post(api::record_lineage))
        .route("/lineage/:entity_id", get(api::get_lineage))
        .route("/lineage/:entity_id", delete(api::delete_lineage))
        .route("/lineage/:entity_id/impact", get(api::get_lineage_impact))
        .route("/lineage/:entity_id/sources", get(api::get_lineage_sources))
        // Quality endpoints
        .route("/quality/rules", get(api::list_quality_rules))
        .route("/quality/rules", post(api::create_quality_rule))
        .route("/quality/rules/:id", put(api::update_quality_rule))
        .route("/quality/rules/:id", delete(api::delete_quality_rule))
        .route("/quality/evaluate/:dataset_id", post(api::evaluate_dataset))
        .route("/quality/score/:dataset_id", get(api::get_quality_score))
        .route("/quality/pii-scan/:dataset_id", post(api::pii_scan))
        .route("/quality/entities/:id/issues", get(api::get_quality_issues))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = std::env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8083".to_string())
        .parse()?;

    info!("Data Hub Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Data Hub Service shutdown complete");
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

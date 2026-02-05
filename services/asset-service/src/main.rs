use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post, delete, put},
    Router,
};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod assets;
mod relationships;
mod graph;
mod impact;

use crate::assets::AssetService;
use crate::relationships::RelationshipService;
use crate::graph::GraphService;

/// Application state shared across handlers
pub struct AppState {
    pub assets: AssetService,
    pub relationships: RelationshipService,
    pub graph: GraphService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "asset_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting Sysilo Asset Registry Service");

    // Initialize Neo4j connection
    let neo4j_uri = std::env::var("NEO4J_URI")
        .unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let neo4j_user = std::env::var("NEO4J_USER")
        .unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password = std::env::var("NEO4J_PASSWORD")
        .unwrap_or_else(|_| "password".to_string());

    // Initialize PostgreSQL connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sysilo:sysilo_dev@localhost:5432/sysilo".to_string());

    let assets = AssetService::new(&database_url, &neo4j_uri, &neo4j_user, &neo4j_password).await?;
    let relationships = RelationshipService::new(&neo4j_uri, &neo4j_user, &neo4j_password).await?;
    let graph = GraphService::new(&neo4j_uri, &neo4j_user, &neo4j_password).await?;

    let state = Arc::new(AppState {
        assets,
        relationships,
        graph,
    });

    // Build router
    let app = Router::new()
        // Health endpoints
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Asset endpoints
        .route("/assets", get(api::list_assets))
        .route("/assets", post(api::create_asset))
        .route("/assets/:id", get(api::get_asset))
        .route("/assets/:id", put(api::update_asset))
        .route("/assets/:id", delete(api::delete_asset))
        .route("/assets/search", get(api::search_assets))
        // Relationship endpoints
        .route("/relationships", get(api::list_relationships))
        .route("/relationships", post(api::create_relationship))
        .route("/relationships/:id", delete(api::delete_relationship))
        .route("/assets/:id/relationships", get(api::get_asset_relationships))
        // Graph endpoints
        .route("/graph/neighbors/:id", get(api::get_neighbors))
        .route("/graph/path", get(api::find_path))
        .route("/graph/subgraph/:id", get(api::get_subgraph))
        // Impact analysis
        .route("/impact/:id", get(api::get_impact_analysis))
        .route("/impact/:id/downstream", get(api::get_downstream_impact))
        .route("/impact/:id/upstream", get(api::get_upstream_dependencies))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = std::env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8084".to_string())
        .parse()?;

    info!("Asset Registry Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Asset Registry Service shutdown complete");
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

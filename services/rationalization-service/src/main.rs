use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod scoring;
mod scenarios;
mod playbooks;
mod recommendations;

use api::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "rationalization_service=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sysilo:sysilo@localhost:5432/sysilo".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // AI service URL for recommendations
    let ai_service_url = std::env::var("AI_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());

    let state = AppState::new(pool, ai_service_url);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health checks
        .route("/health", get(api::health))
        .route("/ready", get(api::ready))
        // Applications
        .route("/applications", get(api::list_applications).post(api::create_application))
        .route("/applications/:id", get(api::get_application).put(api::update_application).delete(api::delete_application))
        .route("/applications/:id/scores", get(api::get_application_scores).post(api::set_application_score))
        .route("/applications/:id/dependencies", get(api::get_application_dependencies))
        .route("/applications/:id/impact", get(api::get_impact_analysis))
        // Scoring
        .route("/scoring/dimensions", get(api::list_scoring_dimensions).post(api::create_scoring_dimension))
        .route("/scoring/dimensions/:id", put(api::update_scoring_dimension))
        .route("/scoring/calculate/:id", post(api::calculate_time_quadrant))
        .route("/scoring/bulk-calculate", post(api::bulk_calculate_time_quadrants))
        // TIME Quadrant
        .route("/time/assessments", get(api::list_time_assessments))
        .route("/time/assessments/:id", get(api::get_time_assessment).put(api::override_time_assessment))
        .route("/time/summary", get(api::get_time_summary))
        // Scenarios
        .route("/scenarios", get(api::list_scenarios).post(api::create_scenario))
        .route("/scenarios/:id", get(api::get_scenario).put(api::update_scenario).delete(api::delete_scenario))
        .route("/scenarios/:id/analyze", post(api::analyze_scenario))
        .route("/scenarios/compare", post(api::compare_scenarios))
        // Playbooks
        .route("/playbooks", get(api::list_playbooks).post(api::create_playbook))
        .route("/playbooks/:id", get(api::get_playbook).put(api::update_playbook))
        .route("/playbooks/templates", get(api::list_playbook_templates))
        // Migration Projects
        .route("/projects", get(api::list_migration_projects).post(api::create_migration_project))
        .route("/projects/:id", get(api::get_migration_project).put(api::update_migration_project))
        .route("/projects/:id/progress", put(api::update_project_progress))
        // AI Recommendations
        .route("/recommendations", get(api::list_recommendations))
        .route("/recommendations/generate", post(api::generate_recommendations))
        .route("/recommendations/:id", get(api::get_recommendation).put(api::update_recommendation_status))
        // Analytics
        .route("/analytics/portfolio", get(api::get_portfolio_analytics))
        .route("/analytics/trends", get(api::get_score_trends))
        .route("/analytics/cost-analysis", get(api::get_cost_analysis))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8087);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Rationalization service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

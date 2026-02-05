use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::config::DatabaseConfig;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Database storage layer
pub struct Storage {
    pool: PgPool,
}

impl Storage {
    /// Create a new storage instance
    pub async fn new(config: &DatabaseConfig) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    /// Get the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<(), StorageError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Integration CRUD operations

    /// List integrations for a tenant
    pub async fn list_integrations(
        &self,
        tenant_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<IntegrationRow>, StorageError> {
        let rows: Vec<IntegrationRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, name, description, definition,
                version, status, schedule, config,
                created_at, updated_at
            FROM integrations
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get a single integration
    pub async fn get_integration(
        &self,
        tenant_id: &str,
        integration_id: Uuid,
    ) -> Result<IntegrationRow, StorageError> {
        let row: Option<IntegrationRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, name, description, definition,
                version, status, schedule, config,
                created_at, updated_at
            FROM integrations
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("Integration {}", integration_id)))
    }

    /// Create a new integration
    pub async fn create_integration(
        &self,
        tenant_id: &str,
        name: &str,
        description: Option<&str>,
        definition: serde_json::Value,
    ) -> Result<IntegrationRow, StorageError> {
        let row: IntegrationRow = sqlx::query_as(
            r#"
            INSERT INTO integrations (tenant_id, name, description, definition, status)
            VALUES ($1, $2, $3, $4, 'draft')
            RETURNING
                id, tenant_id, name, description, definition,
                version, status, schedule, config,
                created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(definition)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    // Integration Run operations

    /// Create a new integration run
    pub async fn create_run(
        &self,
        tenant_id: &str,
        integration_id: Uuid,
        integration_version: i32,
        trigger_type: &str,
    ) -> Result<RunRow, StorageError> {
        let row: RunRow = sqlx::query_as(
            r#"
            INSERT INTO integration_runs
                (tenant_id, integration_id, integration_version, trigger_type, status)
            VALUES ($1, $2, $3, $4, 'pending')
            RETURNING
                id, tenant_id, integration_id, integration_version, status,
                trigger_type, started_at, completed_at,
                error_message, metrics, created_at
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(integration_version)
        .bind(trigger_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get a run by ID
    pub async fn get_run(
        &self,
        tenant_id: &str,
        run_id: Uuid,
    ) -> Result<RunRow, StorageError> {
        let row: Option<RunRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, integration_id, integration_version, status,
                trigger_type, started_at, completed_at,
                error_message, metrics, created_at
            FROM integration_runs
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("Run {}", run_id)))
    }

    /// Update run status
    pub async fn update_run_status(
        &self,
        run_id: Uuid,
        status: &str,
        error_message: Option<&str>,
        metrics: Option<serde_json::Value>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE integration_runs
            SET
                status = $2,
                error_message = COALESCE($3, error_message),
                metrics = COALESCE($4, metrics),
                started_at = CASE WHEN $2 = 'running' AND started_at IS NULL THEN NOW() ELSE started_at END,
                completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status)
        .bind(error_message)
        .bind(metrics)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Database row for integrations
#[derive(Debug, FromRow)]
pub struct IntegrationRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub definition: serde_json::Value,
    pub version: i32,
    pub status: String,
    pub schedule: Option<serde_json::Value>,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database row for integration runs
#[derive(Debug, FromRow)]
pub struct RunRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub integration_version: i32,
    pub status: String,
    pub trigger_type: String,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub metrics: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

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

    // =============================================================================
    // Playbook CRUD operations
    // =============================================================================

    /// List playbooks for a tenant
    pub async fn list_playbooks(
        &self,
        tenant_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PlaybookRow>, StorageError> {
        let rows: Vec<PlaybookRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, trigger_type,
                   steps, variables, created_at, updated_at
            FROM playbooks
            WHERE tenant_id = $1::uuid
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

    /// Get a single playbook
    pub async fn get_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
    ) -> Result<PlaybookRow, StorageError> {
        let row: Option<PlaybookRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, description, trigger_type,
                   steps, variables, created_at, updated_at
            FROM playbooks
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("Playbook {}", playbook_id)))
    }

    /// Create a new playbook
    pub async fn create_playbook(
        &self,
        tenant_id: &str,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        steps: serde_json::Value,
        variables: serde_json::Value,
    ) -> Result<PlaybookRow, StorageError> {
        let row: PlaybookRow = sqlx::query_as(
            r#"
            INSERT INTO playbooks (tenant_id, name, description, trigger_type, steps, variables)
            VALUES ($1::uuid, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, description, trigger_type,
                      steps, variables, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(steps)
        .bind(variables)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Update a playbook
    pub async fn update_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        steps: serde_json::Value,
        variables: serde_json::Value,
    ) -> Result<PlaybookRow, StorageError> {
        let row: PlaybookRow = sqlx::query_as(
            r#"
            UPDATE playbooks
            SET name = $3, description = $4, trigger_type = $5,
                steps = $6, variables = $7, updated_at = NOW()
            WHERE tenant_id = $1::uuid AND id = $2
            RETURNING id, tenant_id, name, description, trigger_type,
                      steps, variables, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(steps)
        .bind(variables)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Delete a playbook
    pub async fn delete_playbook(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
    ) -> Result<(), StorageError> {
        let result = sqlx::query(
            r#"
            DELETE FROM playbooks
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Playbook {}", playbook_id)));
        }

        Ok(())
    }

    // =============================================================================
    // Playbook Run operations
    // =============================================================================

    /// Create a new playbook run
    pub async fn create_playbook_run(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        variables: serde_json::Value,
        step_states: serde_json::Value,
    ) -> Result<PlaybookRunRow, StorageError> {
        let row: PlaybookRunRow = sqlx::query_as(
            r#"
            INSERT INTO playbook_runs (tenant_id, playbook_id, variables, step_states, status)
            VALUES ($1::uuid, $2, $3, $4, 'pending')
            RETURNING id, playbook_id, tenant_id, status, variables,
                      step_states, started_at, completed_at
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(variables)
        .bind(step_states)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Get a playbook run by ID
    pub async fn get_playbook_run(
        &self,
        tenant_id: &str,
        run_id: Uuid,
    ) -> Result<PlaybookRunRow, StorageError> {
        let row: Option<PlaybookRunRow> = sqlx::query_as(
            r#"
            SELECT id, playbook_id, tenant_id, status, variables,
                   step_states, started_at, completed_at
            FROM playbook_runs
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("PlaybookRun {}", run_id)))
    }

    /// List runs for a playbook
    pub async fn list_playbook_runs(
        &self,
        tenant_id: &str,
        playbook_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PlaybookRunRow>, StorageError> {
        let rows: Vec<PlaybookRunRow> = sqlx::query_as(
            r#"
            SELECT id, playbook_id, tenant_id, status, variables,
                   step_states, started_at, completed_at
            FROM playbook_runs
            WHERE tenant_id = $1::uuid AND playbook_id = $2
            ORDER BY started_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(playbook_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Update playbook run status and step states
    pub async fn update_playbook_run(
        &self,
        run_id: Uuid,
        status: &str,
        step_states: serde_json::Value,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE playbook_runs
            SET status = $2, step_states = $3,
                completed_at = CASE
                    WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW()
                    ELSE completed_at
                END
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status)
        .bind(step_states)
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

/// Database row for playbooks
#[derive(Debug, FromRow)]
pub struct PlaybookRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub steps: serde_json::Value,
    pub variables: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database row for playbook runs
#[derive(Debug, FromRow)]
pub struct PlaybookRunRow {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub tenant_id: Uuid,
    pub status: String,
    pub variables: serde_json::Value,
    pub step_states: serde_json::Value,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

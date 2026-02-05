# Connections Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build full CRUD for connections so users can configure data sources, test connectivity, and use real connections in discovery.

**Architecture:** New `connections` module in integration-service with Axum handlers + Storage CRUD, following the exact same patterns as playbooks. Frontend ConnectionsPage rewritten from mock data to real API with create/edit modal and test button. Discovery modal wired to real connections list.

**Tech Stack:** Rust/Axum, sqlx (PostgreSQL), serde_json, React, TypeScript, React Query, Tailwind CSS

---

### Task 1: Database Migration + Types

**Files:**
- Create: `services/integration-service/migrations/20260204100000_create_connections.sql`
- Create: `services/integration-service/src/connections/mod.rs`
- Modify: `services/integration-service/src/main.rs` (add `mod connections;`)

**Step 1: Create migration**

Create `services/integration-service/migrations/20260204100000_create_connections.sql`:

```sql
-- Connections management schema
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    auth_type VARCHAR(50) NOT NULL DEFAULT 'credential',
    config JSONB NOT NULL DEFAULT '{}',
    credentials JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'untested',
    last_tested_at TIMESTAMPTZ,
    last_test_status VARCHAR(50),
    last_test_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_connections_tenant ON connections(tenant_id);
CREATE INDEX idx_connections_type ON connections(connector_type);
```

**Step 2: Create types module**

Create `services/integration-service/src/connections/mod.rs`:

```rust
pub mod api;

use serde::{Deserialize, Serialize};

/// Supported connector types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    Postgresql,
    Mysql,
    Snowflake,
    Oracle,
    Salesforce,
    RestApi,
}

impl std::fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgresql => write!(f, "postgresql"),
            Self::Mysql => write!(f, "mysql"),
            Self::Snowflake => write!(f, "snowflake"),
            Self::Oracle => write!(f, "oracle"),
            Self::Salesforce => write!(f, "salesforce"),
            Self::RestApi => write!(f, "rest_api"),
        }
    }
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Credential,
    Oauth,
    ApiKey,
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Credential => write!(f, "credential"),
            Self::Oauth => write!(f, "oauth"),
            Self::ApiKey => write!(f, "api_key"),
        }
    }
}

/// Validate that config contains required fields for the connector type.
/// Returns Ok(()) if valid, Err(message) if missing fields.
pub fn validate_config(connector_type: &ConnectorType, config: &serde_json::Value) -> Result<(), String> {
    let obj = config.as_object().ok_or("config must be a JSON object")?;

    let required_fields: &[&str] = match connector_type {
        ConnectorType::Postgresql => &["host", "port", "database"],
        ConnectorType::Mysql => &["host", "port", "database"],
        ConnectorType::Snowflake => &["account", "warehouse", "database"],
        ConnectorType::Oracle => &["host", "port", "service_name"],
        ConnectorType::Salesforce => &["instance_url"],
        ConnectorType::RestApi => &["base_url"],
    };

    let missing: Vec<&str> = required_fields
        .iter()
        .filter(|f| !obj.contains_key(**f))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("Missing required config fields for {}: {}", connector_type, missing.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_postgresql_valid() {
        let config = serde_json::json!({"host": "localhost", "port": 5432, "database": "mydb"});
        assert!(validate_config(&ConnectorType::Postgresql, &config).is_ok());
    }

    #[test]
    fn test_validate_config_postgresql_missing_field() {
        let config = serde_json::json!({"host": "localhost"});
        let err = validate_config(&ConnectorType::Postgresql, &config).unwrap_err();
        assert!(err.contains("port"));
        assert!(err.contains("database"));
    }

    #[test]
    fn test_validate_config_salesforce_valid() {
        let config = serde_json::json!({"instance_url": "https://myorg.salesforce.com"});
        assert!(validate_config(&ConnectorType::Salesforce, &config).is_ok());
    }

    #[test]
    fn test_validate_config_not_object() {
        let config = serde_json::json!("not an object");
        let err = validate_config(&ConnectorType::Postgresql, &config).unwrap_err();
        assert!(err.contains("JSON object"));
    }

    #[test]
    fn test_connector_type_serialization() {
        let ct = ConnectorType::RestApi;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"rest_api\"");
    }

    #[test]
    fn test_auth_type_serialization() {
        let at = AuthType::ApiKey;
        let json = serde_json::to_string(&at).unwrap();
        assert_eq!(json, "\"api_key\"");
    }
}
```

**Step 3: Add module declaration to main.rs**

In `services/integration-service/src/main.rs`, add `mod connections;` after `mod consumer;` (line 17):

```rust
mod connections;
```

**Step 4: Run tests and build**

```bash
cd services/integration-service && cargo test -- connections && cargo check
```

Expected: 6 tests pass, build succeeds.

**Step 5: Commit**

```bash
git add services/integration-service/migrations/20260204100000_create_connections.sql \
  services/integration-service/src/connections/mod.rs \
  services/integration-service/src/main.rs
git commit -m "feat(connections): add migration, types, and config validation"
```

---

### Task 2: Storage CRUD Methods

**Files:**
- Modify: `services/integration-service/src/storage/mod.rs`

**Step 1: Add ConnectionRow struct**

Add after `PlaybookRunRow` at the bottom of `storage/mod.rs`:

```rust
/// Database row for connections
#[derive(Debug, FromRow)]
pub struct ConnectionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub connector_type: String,
    pub auth_type: String,
    pub config: serde_json::Value,
    pub credentials: serde_json::Value,
    pub status: String,
    pub last_tested_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_test_status: Option<String>,
    pub last_test_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

**Step 2: Add CRUD methods**

Add a new section in the `impl Storage` block after the playbook run methods:

```rust
    // =============================================================================
    // Connection CRUD operations
    // =============================================================================

    /// List connections for a tenant
    pub async fn list_connections(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<ConnectionRow>, StorageError> {
        let rows: Vec<ConnectionRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, connector_type, auth_type,
                   config, credentials, status,
                   last_tested_at, last_test_status, last_test_error,
                   created_at, updated_at
            FROM connections
            WHERE tenant_id = $1::uuid
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get a single connection
    pub async fn get_connection(
        &self,
        tenant_id: &str,
        connection_id: Uuid,
    ) -> Result<ConnectionRow, StorageError> {
        let row: Option<ConnectionRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, name, connector_type, auth_type,
                   config, credentials, status,
                   last_tested_at, last_test_status, last_test_error,
                   created_at, updated_at
            FROM connections
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("Connection {}", connection_id)))
    }

    /// Create a new connection
    pub async fn create_connection(
        &self,
        tenant_id: &str,
        name: &str,
        connector_type: &str,
        auth_type: &str,
        config: serde_json::Value,
        credentials: serde_json::Value,
    ) -> Result<ConnectionRow, StorageError> {
        let row: ConnectionRow = sqlx::query_as(
            r#"
            INSERT INTO connections (tenant_id, name, connector_type, auth_type, config, credentials)
            VALUES ($1::uuid, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, connector_type, auth_type,
                      config, credentials, status,
                      last_tested_at, last_test_status, last_test_error,
                      created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(connector_type)
        .bind(auth_type)
        .bind(config)
        .bind(credentials)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Update a connection
    pub async fn update_connection(
        &self,
        tenant_id: &str,
        connection_id: Uuid,
        name: &str,
        config: serde_json::Value,
        credentials: Option<serde_json::Value>,
    ) -> Result<ConnectionRow, StorageError> {
        // If credentials provided, update them; otherwise keep existing
        let row: ConnectionRow = if let Some(creds) = credentials {
            sqlx::query_as(
                r#"
                UPDATE connections
                SET name = $3, config = $4, credentials = $5, updated_at = NOW()
                WHERE tenant_id = $1::uuid AND id = $2
                RETURNING id, tenant_id, name, connector_type, auth_type,
                          config, credentials, status,
                          last_tested_at, last_test_status, last_test_error,
                          created_at, updated_at
                "#,
            )
            .bind(tenant_id)
            .bind(connection_id)
            .bind(name)
            .bind(config)
            .bind(creds)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                UPDATE connections
                SET name = $3, config = $4, updated_at = NOW()
                WHERE tenant_id = $1::uuid AND id = $2
                RETURNING id, tenant_id, name, connector_type, auth_type,
                          config, credentials, status,
                          last_tested_at, last_test_status, last_test_error,
                          created_at, updated_at
                "#,
            )
            .bind(tenant_id)
            .bind(connection_id)
            .bind(name)
            .bind(config)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(row)
    }

    /// Delete a connection
    pub async fn delete_connection(
        &self,
        tenant_id: &str,
        connection_id: Uuid,
    ) -> Result<(), StorageError> {
        let result = sqlx::query(
            r#"
            DELETE FROM connections
            WHERE tenant_id = $1::uuid AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(connection_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Connection {}", connection_id)));
        }

        Ok(())
    }

    /// Update connection test status
    pub async fn update_connection_test_status(
        &self,
        connection_id: Uuid,
        status: &str,
        test_status: &str,
        test_error: Option<&str>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE connections
            SET status = $2, last_tested_at = NOW(),
                last_test_status = $3, last_test_error = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .bind(status)
        .bind(test_status)
        .bind(test_error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
```

**Step 3: Build**

```bash
cd services/integration-service && cargo check
```

Expected: compiles cleanly.

**Step 4: Commit**

```bash
git add services/integration-service/src/storage/mod.rs
git commit -m "feat(connections): add storage CRUD methods"
```

---

### Task 3: API Handlers

**Files:**
- Create: `services/integration-service/src/connections/api.rs`
- Modify: `services/integration-service/src/main.rs` (add routes)

**Step 1: Create API handlers**

Create `services/integration-service/src/connections/api.rs`:

```rust
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
use crate::connections::{validate_config, AuthType, ConnectorType};
use crate::middleware::TenantContext;
use crate::AppState;

// === Response types ===

#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub name: String,
    pub connector_type: String,
    pub auth_type: String,
    pub config: serde_json::Value,
    pub has_credentials: bool,
    pub status: String,
    pub last_tested_at: Option<String>,
    pub last_test_status: Option<String>,
    pub last_test_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionListResponse {
    pub connections: Vec<ConnectionResponse>,
    pub total: usize,
}

// === Request types ===

#[derive(Debug, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub connector_type: ConnectorType,
    pub auth_type: AuthType,
    pub config: serde_json::Value,
    #[serde(default)]
    pub credentials: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConnectionRequest {
    pub name: String,
    pub config: serde_json::Value,
    /// If omitted, existing credentials are kept
    pub credentials: Option<serde_json::Value>,
}

// === Handlers ===

/// GET /connections
pub async fn list_connections(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<ConnectionListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = state
        .storage
        .list_connections(&tenant_id)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let connections: Vec<ConnectionResponse> = rows
        .into_iter()
        .map(|r| ConnectionResponse {
            id: r.id,
            name: r.name,
            connector_type: r.connector_type,
            auth_type: r.auth_type,
            config: r.config,
            has_credentials: r.credentials != serde_json::json!({}),
            status: r.status,
            last_tested_at: r.last_tested_at.map(|t| t.to_rfc3339()),
            last_test_status: r.last_test_status,
            last_test_error: r.last_test_error,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        })
        .collect();

    let total = connections.len();

    Ok(Json(ConnectionListResponse { connections, total }))
}

/// POST /connections
pub async fn create_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<ConnectionResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Validate config for connector type
    validate_config(&req.connector_type, &req.config).map_err(|e| ApiError {
        error: "validation_error".to_string(),
        message: e,
    })?;

    let row = state
        .storage
        .create_connection(
            &tenant_id,
            &req.name,
            &req.connector_type.to_string(),
            &req.auth_type.to_string(),
            req.config,
            req.credentials,
        )
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(
        connection_id = %row.id,
        connector_type = %row.connector_type,
        "Connection created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ConnectionResponse {
            id: row.id,
            name: row.name,
            connector_type: row.connector_type,
            auth_type: row.auth_type,
            config: row.config,
            has_credentials: row.credentials != serde_json::json!({}),
            status: row.status,
            last_tested_at: row.last_tested_at.map(|t| t.to_rfc3339()),
            last_test_status: row.last_test_status,
            last_test_error: row.last_test_error,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }),
    ))
}

/// GET /connections/:id
pub async fn get_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let row = state
        .storage
        .get_connection(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(ConnectionResponse {
        id: row.id,
        name: row.name,
        connector_type: row.connector_type,
        auth_type: row.auth_type,
        config: row.config,
        has_credentials: row.credentials != serde_json::json!({}),
        status: row.status,
        last_tested_at: row.last_tested_at.map(|t| t.to_rfc3339()),
        last_test_status: row.last_test_status,
        last_test_error: row.last_test_error,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

/// PUT /connections/:id
pub async fn update_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnectionRequest>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get existing to validate config against its connector_type
    let existing = state
        .storage
        .get_connection(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    let connector_type: ConnectorType =
        serde_json::from_value(serde_json::json!(existing.connector_type)).map_err(|e| ApiError {
            error: "invalid_connector_type".to_string(),
            message: e.to_string(),
        })?;

    validate_config(&connector_type, &req.config).map_err(|e| ApiError {
        error: "validation_error".to_string(),
        message: e,
    })?;

    let row = state
        .storage
        .update_connection(&tenant_id, id, &req.name, req.config, req.credentials)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(ConnectionResponse {
        id: row.id,
        name: row.name,
        connector_type: row.connector_type,
        auth_type: row.auth_type,
        config: row.config,
        has_credentials: row.credentials != serde_json::json!({}),
        status: row.status,
        last_tested_at: row.last_tested_at.map(|t| t.to_rfc3339()),
        last_test_status: row.last_test_status,
        last_test_error: row.last_test_error,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

/// DELETE /connections/:id
pub async fn delete_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    state
        .storage
        .delete_connection(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(connection_id = %id, "Connection deleted");

    Ok(StatusCode::NO_CONTENT)
}

/// POST /connections/:id/test
pub async fn test_connection(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the connection
    let conn = state
        .storage
        .get_connection(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // For now, validate config shape as a basic "test"
    // Real connectivity testing will be added when agents support it
    let connector_type: ConnectorType =
        serde_json::from_value(serde_json::json!(conn.connector_type)).map_err(|e| ApiError {
            error: "invalid_connector_type".to_string(),
            message: e.to_string(),
        })?;

    let (status, test_status, test_error) = match validate_config(&connector_type, &conn.config) {
        Ok(()) => ("active", "success", None),
        Err(e) => ("error", "failure", Some(e)),
    };

    state
        .storage
        .update_connection_test_status(id, status, test_status, test_error.as_deref())
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // Re-fetch to get updated fields
    let row = state
        .storage
        .get_connection(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    Ok(Json(ConnectionResponse {
        id: row.id,
        name: row.name,
        connector_type: row.connector_type,
        auth_type: row.auth_type,
        config: row.config,
        has_credentials: row.credentials != serde_json::json!({}),
        status: row.status,
        last_tested_at: row.last_tested_at.map(|t| t.to_rfc3339()),
        last_test_status: row.last_test_status,
        last_test_error: row.last_test_error,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}
```

**Step 2: Register routes in main.rs**

In `services/integration-service/src/main.rs`, add import and routes.

Add import after the playbooks_api import (around line 26):
```rust
use crate::connections::api as connections_api;
```

Add routes in the `protected_routes` builder, after the discovery endpoints (after line 148):
```rust
        // Connection endpoints
        .route("/connections", get(connections_api::list_connections))
        .route("/connections", post(connections_api::create_connection))
        .route("/connections/:id", get(connections_api::get_connection))
        .route("/connections/:id", put(connections_api::update_connection))
        .route("/connections/:id", delete(connections_api::delete_connection))
        .route("/connections/:id/test", post(connections_api::test_connection))
```

**Step 3: Build and test**

```bash
cd services/integration-service && cargo test && cargo check
```

Expected: All existing tests pass + build succeeds.

**Step 4: Commit**

```bash
git add services/integration-service/src/connections/api.rs \
  services/integration-service/src/main.rs
git commit -m "feat(connections): add REST API handlers and routes"
```

---

### Task 4: Frontend API Service + Hooks

**Files:**
- Create: `packages/frontend/web-app/src/services/connections.ts`
- Create: `packages/frontend/web-app/src/hooks/useConnections.ts`

**Step 1: Create API service**

Create `packages/frontend/web-app/src/services/connections.ts`:

```typescript
import { apiFetch } from './api';

const DEV_TENANT_ID = 'dev-tenant';

export type ConnectorType = 'postgresql' | 'mysql' | 'snowflake' | 'oracle' | 'salesforce' | 'rest_api';
export type AuthType = 'credential' | 'oauth' | 'api_key';

export interface Connection {
  id: string;
  name: string;
  connector_type: ConnectorType;
  auth_type: AuthType;
  config: Record<string, unknown>;
  has_credentials: boolean;
  status: string;
  last_tested_at?: string;
  last_test_status?: string;
  last_test_error?: string;
  created_at: string;
  updated_at: string;
}

export interface ConnectionListResponse {
  connections: Connection[];
  total: number;
}

export interface CreateConnectionRequest {
  name: string;
  connector_type: ConnectorType;
  auth_type: AuthType;
  config: Record<string, unknown>;
  credentials?: Record<string, unknown>;
}

export interface UpdateConnectionRequest {
  name: string;
  config: Record<string, unknown>;
  credentials?: Record<string, unknown>;
}

const headers = {
  'Content-Type': 'application/json',
  'X-Tenant-ID': DEV_TENANT_ID,
};

export async function listConnections(): Promise<Connection[]> {
  const resp = await apiFetch<ConnectionListResponse>('/connections', { headers });
  return resp.connections;
}

export async function getConnection(id: string): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}`, { headers });
}

export async function createConnection(request: CreateConnectionRequest): Promise<Connection> {
  return apiFetch<Connection>('/connections', {
    method: 'POST',
    headers,
    body: JSON.stringify(request),
  });
}

export async function updateConnection(id: string, request: UpdateConnectionRequest): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}`, {
    method: 'PUT',
    headers,
    body: JSON.stringify(request),
  });
}

export async function deleteConnection(id: string): Promise<void> {
  await apiFetch(`/connections/${id}`, {
    method: 'DELETE',
    headers,
  });
}

export async function testConnection(id: string): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}/test`, {
    method: 'POST',
    headers,
  });
}

/// Connector type metadata for UI display
export const CONNECTOR_TYPES: Record<ConnectorType, { label: string; authType: AuthType; configFields: string[] }> = {
  postgresql: { label: 'PostgreSQL', authType: 'credential', configFields: ['host', 'port', 'database', 'ssl_mode'] },
  mysql: { label: 'MySQL', authType: 'credential', configFields: ['host', 'port', 'database'] },
  snowflake: { label: 'Snowflake', authType: 'credential', configFields: ['account', 'warehouse', 'database', 'schema'] },
  oracle: { label: 'Oracle', authType: 'credential', configFields: ['host', 'port', 'service_name'] },
  salesforce: { label: 'Salesforce', authType: 'oauth', configFields: ['instance_url', 'api_version'] },
  rest_api: { label: 'REST API', authType: 'api_key', configFields: ['base_url', 'headers'] },
};
```

**Step 2: Create React Query hooks**

Create `packages/frontend/web-app/src/hooks/useConnections.ts`:

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listConnections,
  createConnection,
  updateConnection,
  deleteConnection,
  testConnection,
  type CreateConnectionRequest,
  type UpdateConnectionRequest,
} from '../services/connections';

export function useConnections() {
  return useQuery({
    queryKey: ['connections'],
    queryFn: listConnections,
    staleTime: 30_000,
  });
}

export function useCreateConnection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateConnectionRequest) => createConnection(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
    },
  });
}

export function useUpdateConnection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, request }: { id: string; request: UpdateConnectionRequest }) =>
      updateConnection(id, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
    },
  });
}

export function useDeleteConnection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteConnection(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
    },
  });
}

export function useTestConnection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => testConnection(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
    },
  });
}
```

**Step 3: Build**

```bash
cd packages/frontend/web-app && npx tsc --noEmit
```

Expected: Type checks pass.

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/services/connections.ts \
  packages/frontend/web-app/src/hooks/useConnections.ts
git commit -m "feat(connections): add frontend API service and React Query hooks"
```

---

### Task 5: Rewrite ConnectionsPage

**Files:**
- Modify: `packages/frontend/web-app/src/pages/ConnectionsPage.tsx`

**Step 1: Rewrite ConnectionsPage with real API, create modal, and test button**

This is the USER CONTRIBUTION task. The page needs:
- Real data from `useConnections()` hook (not hardcoded)
- A "New Connection" modal with: connector type picker, name field, dynamic config fields based on type, optional credentials
- Test button that calls `useTestConnection()` and shows result
- Delete with confirmation
- Loading/error/empty states

The rewrite replaces the existing 72-line mock page with a full-featured page. Use the `frontend-design` skill for this component since it's the most substantial UI piece.

**Step 2: Build**

```bash
cd packages/frontend/web-app && npx tsc --noEmit
```

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/pages/ConnectionsPage.tsx
git commit -m "feat(connections): rewrite ConnectionsPage with real API and create modal"
```

---

### Task 6: Wire Discovery Modal to Real Connections

**Files:**
- Modify: `packages/frontend/web-app/src/services/discovery.ts`

**Step 1: Replace stubbed listConnections**

In `packages/frontend/web-app/src/services/discovery.ts`, replace the stubbed `listConnections` function and the `Connection` interface. Import from the real connections service instead:

Remove the `Connection` interface (lines 16-21) and the `listConnections` function (lines 43-65).

Replace with a re-export:

```typescript
// Re-export from connections service (replaces stub)
export { listConnections } from './connections';
export type { Connection } from './connections';
```

Keep all other exports (DiscoveryRequest, DiscoveryResponse, runDiscovery, MockDiscoveryRequest, MockDiscoveryResponse, triggerMockDiscovery) unchanged.

**Step 2: Build**

```bash
cd packages/frontend/web-app && npx tsc --noEmit
```

Expected: Type checks pass. The `useConnections()` hook in `useDiscovery.ts` already imports `listConnections` from `../services/discovery.js` — it will now get the real version.

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/services/discovery.ts
git commit -m "feat(connections): wire discovery modal to real connections API"
```

---

### Task 7: Build Verification

**Step 1: Full backend build and test**

```bash
cd services/integration-service && cargo build && cargo test
```

Expected: All tests pass including new connection config validation tests.

**Step 2: Full frontend type check**

```bash
cd packages/frontend/web-app && npx tsc --noEmit
```

Expected: No type errors.

**Step 3: Final commit if any remaining changes**

If there are any fixup changes needed.

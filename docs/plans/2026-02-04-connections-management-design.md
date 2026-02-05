# Connections Management Design

**Date:** 2026-02-04
**Status:** Approved

---

## Overview

Connections are the credential store for external systems — databases, SaaS apps, APIs. They're referenced by discovery runs and integrations. This feature replaces the hardcoded mock data in the frontend with a full CRUD backend, wires the ConnectionsPage to real APIs, and connects the discovery modal to real connection data.

## Data Model

```sql
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    auth_type VARCHAR(50) NOT NULL,
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

### Supported Connector Types

| Type | Auth | Config fields |
|------|------|--------------|
| `postgresql` | credential | host, port, database, ssl_mode |
| `mysql` | credential | host, port, database |
| `snowflake` | credential | account, warehouse, database, schema |
| `oracle` | credential | host, port, service_name |
| `salesforce` | oauth | instance_url, api_version |
| `rest_api` | api_key | base_url, headers |

### Credential Handling

- `credentials` JSONB stores passwords, tokens, API keys
- Never returned in GET/LIST responses
- API returns `has_credentials: true` flag instead

## API Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/connections` | List all connections for tenant |
| `POST` | `/connections` | Create a new connection |
| `GET` | `/connections/:id` | Get connection details (no secrets) |
| `PUT` | `/connections/:id` | Update connection config/credentials |
| `DELETE` | `/connections/:id` | Delete a connection |
| `POST` | `/connections/:id/test` | Test connectivity and update status |

### Response Shape

```json
{
  "id": "uuid",
  "name": "Production PostgreSQL",
  "connector_type": "postgresql",
  "auth_type": "credential",
  "config": { "host": "db.example.com", "port": 5432, "database": "prod" },
  "has_credentials": true,
  "status": "active",
  "last_tested_at": "2026-02-04T...",
  "last_test_status": "success"
}
```

## Implementation Components

### Backend (integration-service)

| File | Responsibility |
|------|---------------|
| `migrations/20260204100000_create_connections.sql` | Table + indexes |
| `src/connections/mod.rs` | Types, ConnectorType enum, config validation |
| `src/connections/api.rs` | 6 Axum handlers |
| `src/storage/mod.rs` | CRUD methods for connections |
| `src/main.rs` | Register routes + mod declaration |

### Frontend

| File | Responsibility |
|------|---------------|
| `src/services/connections.ts` | API client (create, list, get, update, delete, test) |
| `src/hooks/useConnections.ts` | React Query hooks |
| `src/pages/ConnectionsPage.tsx` | Rewrite: real API, create/edit modal, test button |
| `src/services/discovery.ts` | Replace stubbed listConnections with real import |

## Scope Boundaries (YAGNI)

- No actual database/SaaS connectivity testing (mock validation)
- No OAuth flow UI (store tokens via API)
- No connector plugin framework
- No credential encryption at rest (infra concern)

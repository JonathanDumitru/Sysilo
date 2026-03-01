import { apiFetch } from './api';

const DEV_TENANT_ID = 'dev-tenant';

export type ConnectorType = 'postgresql' | 'mysql' | 'snowflake' | 'oracle' | 'salesforce' | 'rest_api';
export type AuthType = 'credential' | 'oauth' | 'api_key';
export type ConnectionStatus = 'draft' | 'tested' | 'active' | 'error' | 'untested';
export type ConnectionTestStatus = 'success' | 'failure';

export interface Connection {
  id: string;
  name: string;
  connector_type: ConnectorType;
  auth_type: AuthType;
  config: Record<string, unknown>;
  has_credentials: boolean;
  status: ConnectionStatus;
  last_tested_at?: string;
  last_test_status?: ConnectionTestStatus;
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
  desired_status?: 'active';
}

const tenantHeaders = { 'X-Tenant-ID': DEV_TENANT_ID };

export async function listConnections(): Promise<Connection[]> {
  const resp = await apiFetch<ConnectionListResponse>('/connections', {
    headers: tenantHeaders,
  });
  return resp.connections;
}

export async function getConnection(id: string): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}`, {
    headers: tenantHeaders,
  });
}

export async function createConnection(request: CreateConnectionRequest): Promise<Connection> {
  return apiFetch<Connection>('/connections', {
    method: 'POST',
    headers: tenantHeaders,
    body: JSON.stringify(request),
  });
}

export async function updateConnection(id: string, request: UpdateConnectionRequest): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}`, {
    method: 'PUT',
    headers: tenantHeaders,
    body: JSON.stringify(request),
  });
}

export async function activateConnection(connection: Connection): Promise<Connection> {
  return updateConnection(connection.id, {
    name: connection.name,
    config: connection.config,
    desired_status: 'active',
  });
}

export async function deleteConnection(id: string): Promise<void> {
  await apiFetch(`/connections/${id}`, {
    method: 'DELETE',
    headers: tenantHeaders,
  });
}

export async function testConnection(id: string): Promise<Connection> {
  return apiFetch<Connection>(`/connections/${id}/test`, {
    method: 'POST',
    headers: tenantHeaders,
  });
}

/** Connector type metadata for UI display */
export const CONNECTOR_TYPES: Record<ConnectorType, { label: string; authType: AuthType; configFields: string[] }> = {
  postgresql: { label: 'PostgreSQL', authType: 'credential', configFields: ['host', 'port', 'database', 'ssl_mode'] },
  mysql: { label: 'MySQL', authType: 'credential', configFields: ['host', 'port', 'database'] },
  snowflake: { label: 'Snowflake', authType: 'credential', configFields: ['account', 'warehouse', 'database', 'schema'] },
  oracle: { label: 'Oracle', authType: 'credential', configFields: ['host', 'port', 'service_name'] },
  salesforce: { label: 'Salesforce', authType: 'oauth', configFields: ['instance_url', 'api_version'] },
  rest_api: { label: 'REST API', authType: 'api_key', configFields: ['base_url', 'headers'] },
};

import { getAuthContextHeaders } from '../config/env';
import { apiClient, GATEWAY_CONNECTIONS_BASE_PATH } from './api';
import {
  SUPPORTED_CONNECTORS,
  type ConnectorAuthType,
  type SupportedConnectorSpec,
  type SupportedConnectorType,
} from '../../../../sdk/typescript/src/index.ts';

export type ConnectorType = SupportedConnectorType;
export type AuthType = ConnectorAuthType;
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

const connectionHeaders = getAuthContextHeaders();

export async function listConnections(): Promise<Connection[]> {
  const resp = await apiClient.request<ConnectionListResponse>(GATEWAY_CONNECTIONS_BASE_PATH, {
    headers: connectionHeaders,
  });
  return resp.connections;
}

export async function getConnection(id: string): Promise<Connection> {
  return apiClient.request<Connection>(`${GATEWAY_CONNECTIONS_BASE_PATH}/${id}`, {
    headers: connectionHeaders,
  });
}

export async function createConnection(request: CreateConnectionRequest): Promise<Connection> {
  return apiClient.request<Connection>(GATEWAY_CONNECTIONS_BASE_PATH, {
    method: 'POST',
    headers: connectionHeaders,
    body: JSON.stringify(request),
  });
}

export async function updateConnection(id: string, request: UpdateConnectionRequest): Promise<Connection> {
  return apiClient.request<Connection>(`${GATEWAY_CONNECTIONS_BASE_PATH}/${id}`, {
    method: 'PUT',
    headers: connectionHeaders,
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
  await apiClient.request(`${GATEWAY_CONNECTIONS_BASE_PATH}/${id}`, {
    method: 'DELETE',
    headers: connectionHeaders,
  });
}

export async function testConnection(id: string): Promise<Connection> {
  return apiClient.request<Connection>(`${GATEWAY_CONNECTIONS_BASE_PATH}/${id}/test`, {
    method: 'POST',
    headers: connectionHeaders,
  });
}

/** Connector type metadata for UI display */
export type ConnectorUiMetadata = Pick<
  SupportedConnectorSpec,
  'label' | 'authType' | 'authModes' | 'configFields' | 'requiresCredentialReplacementOnEdit' | 'capabilities'
>;

export const CONNECTOR_TYPES: Record<ConnectorType, ConnectorUiMetadata> = SUPPORTED_CONNECTORS.reduce(
  (acc, spec) => {
    acc[spec.connectorType] = {
      label: spec.label,
      authType: spec.authType,
      authModes: spec.authModes,
      configFields: spec.configFields,
      requiresCredentialReplacementOnEdit: spec.requiresCredentialReplacementOnEdit,
      capabilities: spec.capabilities,
    };
    return acc;
  },
  {} as Record<ConnectorType, ConnectorUiMetadata>
);

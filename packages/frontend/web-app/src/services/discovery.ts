import { apiFetch } from './api.js';

export interface DiscoveryRequest {
  connection_id: string;
  discovery_type?: 'full' | 'incremental';
  resource_types?: string[];
}

export interface DiscoveryResponse {
  run_id: string;
  task_id: string;
  status: string;
  message: string;
}

export interface Connection {
  id: string;
  name: string;
  connector_type: string;
  status: string;
}

const DEV_TENANT_ID = 'dev-tenant';

/**
 * Start a discovery run against a connection
 */
export async function runDiscovery(request: DiscoveryRequest): Promise<DiscoveryResponse> {
  return apiFetch<DiscoveryResponse>('/discovery/run', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-ID': DEV_TENANT_ID,
    },
    body: JSON.stringify(request),
  });
}

/**
 * List available connections for discovery
 * TODO: Replace with real API when connections service is ready
 */
export async function listConnections(): Promise<Connection[]> {
  // Stub data until connections API is implemented
  return [
    {
      id: '00000000-0000-0000-0000-000000000001',
      name: 'Production PostgreSQL',
      connector_type: 'postgresql',
      status: 'active',
    },
    {
      id: '00000000-0000-0000-0000-000000000002',
      name: 'Salesforce CRM',
      connector_type: 'salesforce',
      status: 'active',
    },
    {
      id: '00000000-0000-0000-0000-000000000003',
      name: 'AWS S3 Data Lake',
      connector_type: 's3',
      status: 'active',
    },
  ];
}

// =============================================================================
// Development/Mock endpoints
// =============================================================================

export interface MockDiscoveryRequest {
  connection_id: string;
  asset_count?: number;
}

export interface MockDiscoveryResponse {
  message: string;
  assets_created: number;
}

/**
 * Trigger mock discovery to generate fake assets (dev only)
 * This bypasses Kafka and directly creates assets in the asset-service
 */
export async function triggerMockDiscovery(
  request: MockDiscoveryRequest
): Promise<MockDiscoveryResponse> {
  return apiFetch<MockDiscoveryResponse>('/dev/mock-discovery', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-ID': DEV_TENANT_ID,
    },
    body: JSON.stringify(request),
  });
}

import { apiFetch } from './api.js';

export { listConnections } from './connections';
export type { Connection } from './connections';

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

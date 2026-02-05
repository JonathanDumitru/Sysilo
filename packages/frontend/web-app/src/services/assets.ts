import { apiFetch } from './api';

// Default tenant ID for development
const DEV_TENANT_ID = '00000000-0000-0000-0000-000000000001';

// Asset types matching backend enum
export type AssetType =
  | 'application'
  | 'service'
  | 'database'
  | 'api'
  | 'data_store'
  | 'integration'
  | 'infrastructure'
  | 'platform'
  | 'tool';

// Asset status matching backend enum
export type AssetStatus =
  | 'active'
  | 'deprecated'
  | 'sunset'
  | 'planned'
  | 'under_review';

export interface Asset {
  id: string;
  tenant_id: string;
  name: string;
  asset_type: AssetType;
  status: AssetStatus;
  description?: string;
  owner?: string;
  team?: string;
  vendor?: string;
  version?: string;
  documentation_url?: string;
  repository_url?: string;
  metadata?: Record<string, unknown>;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface ListAssetsResponse {
  assets: Asset[];
  total: number;
}

export interface ListAssetsParams {
  tenant_id?: string;
  asset_type?: AssetType;
  status?: AssetStatus;
  limit?: number;
  offset?: number;
}

export async function listAssets(
  params: ListAssetsParams = {}
): Promise<ListAssetsResponse> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', params.tenant_id || DEV_TENANT_ID);

  if (params.asset_type) searchParams.set('asset_type', params.asset_type);
  if (params.status) searchParams.set('status', params.status);
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.offset) searchParams.set('offset', params.offset.toString());

  return apiFetch<ListAssetsResponse>(`/assets?${searchParams.toString()}`);
}

export async function getAsset(id: string, tenantId?: string): Promise<Asset> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', tenantId || DEV_TENANT_ID);

  return apiFetch<Asset>(`/assets/${id}?${searchParams.toString()}`);
}

export async function searchAssets(
  query: string,
  tenantId?: string
): Promise<Asset[]> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', tenantId || DEV_TENANT_ID);
  searchParams.set('q', query);

  return apiFetch<Asset[]>(`/assets/search?${searchParams.toString()}`);
}

export interface CreateAssetRequest {
  tenant_id: string;
  name: string;
  asset_type: AssetType;
  status?: AssetStatus;
  description?: string;
  owner?: string;
  team?: string;
  vendor?: string;
  version?: string;
  documentation_url?: string;
  repository_url?: string;
  metadata?: Record<string, unknown>;
  tags?: string[];
}

export async function createAsset(request: CreateAssetRequest): Promise<Asset> {
  return apiFetch<Asset>('/assets', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

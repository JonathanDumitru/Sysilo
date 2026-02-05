import { useQuery } from '@tanstack/react-query';

import {
  listAssets,
  getAsset,
  searchAssets,
  type ListAssetsParams,
  type Asset,
} from '../services/assets.js';

export function useAssets(params: ListAssetsParams = {}) {
  return useQuery({
    queryKey: ['assets', params],
    queryFn: () => listAssets(params),
  });
}

export function useAsset(id: string, tenantId?: string) {
  return useQuery({
    queryKey: ['asset', id, tenantId],
    queryFn: () => getAsset(id, tenantId),
    enabled: !!id,
  });
}

export function useAssetSearch(query: string, tenantId?: string) {
  return useQuery({
    queryKey: ['assets', 'search', query, tenantId],
    queryFn: () => searchAssets(query, tenantId),
    enabled: query.length >= 2,
  });
}

export type { Asset, ListAssetsParams };

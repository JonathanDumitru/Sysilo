import { useQuery } from '@tanstack/react-query';

import {
  listAssets,
  getAsset,
  searchAssets,
  type ListAssetsParams,
  type Asset,
} from '../services/assets.js';

export function useAssets(params: ListAssetsParams = {}, enablePolling = false) {
  return useQuery({
    queryKey: ['assets', params],
    queryFn: () => listAssets(params),
    staleTime: 10_000, // 10 seconds
    refetchInterval: enablePolling ? 5_000 : false, // Poll every 5s when enabled
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

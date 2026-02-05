import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listConnections,
  runDiscovery,
  type DiscoveryRequest,
} from '../services/discovery.js';

/**
 * Hook to list available connections
 */
export function useConnections() {
  return useQuery({
    queryKey: ['connections'],
    queryFn: listConnections,
    staleTime: 30_000, // 30 seconds
  });
}

/**
 * Hook to trigger a discovery run
 */
export function useRunDiscovery() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: DiscoveryRequest) => runDiscovery(request),
    onSuccess: () => {
      // Invalidate assets query to trigger refresh
      // Assets will appear as they're discovered
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

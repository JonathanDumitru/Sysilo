import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listConnections,
  runDiscovery,
  triggerMockDiscovery,
  getDiscoveryRuns,
  type DiscoveryRequest,
  type MockDiscoveryRequest,
  type DiscoveryRun,
} from '../services/discovery.js';

/**
 * Hook to list available connections
 */
export function useConnections() {
  return useQuery({
    queryKey: ['connections'],
    queryFn: listConnections,
    staleTime: 30_000,
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
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

/**
 * Hook to trigger mock discovery (dev only)
 */
export function useMockDiscovery() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: MockDiscoveryRequest) => triggerMockDiscovery(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

/**
 * Hook to poll discovery run statuses.
 * Polls every 3 seconds while any run is in a non-terminal state.
 */
export function useDiscoveryRuns(runIds: string[]) {
  const queryClient = useQueryClient();

  return useQuery({
    queryKey: ['discovery-runs', ...runIds],
    queryFn: () => getDiscoveryRuns(runIds),
    enabled: runIds.length > 0,
    refetchInterval: (query) => {
      const runs = query.state.data as DiscoveryRun[] | undefined;
      if (!runs) return 3000;
      const allTerminal = runs.every(
        (r) => r.status === 'completed' || r.status === 'failed'
      );
      if (allTerminal) {
        // Invalidate assets one final time
        queryClient.invalidateQueries({ queryKey: ['assets'] });
        return false; // Stop polling
      }
      return 3000;
    },
  });
}

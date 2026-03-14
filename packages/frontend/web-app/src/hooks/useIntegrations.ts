import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listIntegrations,
  getIntegration,
  createIntegration,
  runIntegration,
  getRun,
  type CreateIntegrationRequest,
  type IntegrationRun,
} from '../services/integrations';

const INTEGRATIONS_QUERY_KEY = ['integrations'] as const;

export function useIntegrations() {
  return useQuery({
    queryKey: INTEGRATIONS_QUERY_KEY,
    queryFn: listIntegrations,
    staleTime: 30_000,
  });
}

export function useIntegration(id: string | undefined) {
  return useQuery({
    queryKey: [...INTEGRATIONS_QUERY_KEY, id],
    queryFn: () => getIntegration(id!),
    enabled: !!id,
  });
}

export function useCreateIntegration() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateIntegrationRequest) => createIntegration(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: INTEGRATIONS_QUERY_KEY });
    },
  });
}

export function useRunIntegration() {
  return useMutation({
    mutationFn: (id: string) => runIntegration(id),
  });
}

export function useIntegrationRun(runId: string | undefined) {
  return useQuery({
    queryKey: ['integration-runs', runId],
    queryFn: () => getRun(runId!),
    enabled: !!runId,
    refetchInterval: (query) => {
      const data = query.state.data as IntegrationRun | undefined;
      if (data && (data.status === 'completed' || data.status === 'failed' || data.status === 'cancelled')) {
        return false;
      }
      return 5_000;
    },
  });
}

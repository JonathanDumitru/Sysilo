import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listPlaybooks,
  getPlaybook,
  createPlaybook,
  updatePlaybook,
  deletePlaybook,
  runPlaybook,
  listPlaybookRuns,
  getPlaybookRun,
  approveRun,
  rejectRun,
  type CreatePlaybookRequest,
  type RunPlaybookRequest,
  type Playbook,
  type PlaybookSummary,
  type PlaybookRun,
} from '../services/playbooks';

// Query keys
export const playbookKeys = {
  all: ['playbooks'] as const,
  list: () => [...playbookKeys.all, 'list'] as const,
  detail: (id: string) => [...playbookKeys.all, 'detail', id] as const,
  runs: (playbookId: string) => [...playbookKeys.all, 'runs', playbookId] as const,
  run: (runId: string) => [...playbookKeys.all, 'run', runId] as const,
};

// List playbooks
export function usePlaybooks() {
  return useQuery({
    queryKey: playbookKeys.list(),
    queryFn: listPlaybooks,
  });
}

// Get single playbook
export function usePlaybook(id: string) {
  return useQuery({
    queryKey: playbookKeys.detail(id),
    queryFn: () => getPlaybook(id),
    enabled: !!id,
  });
}

// Create playbook
export function useCreatePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreatePlaybookRequest) => createPlaybook(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
    },
  });
}

// Update playbook
export function useUpdatePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, request }: { id: string; request: CreatePlaybookRequest }) =>
      updatePlaybook(id, request),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
      queryClient.invalidateQueries({ queryKey: playbookKeys.detail(data.id) });
    },
  });
}

// Delete playbook
export function useDeletePlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deletePlaybook(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
    },
  });
}

// Run playbook
export function useRunPlaybook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, request }: { id: string; request?: RunPlaybookRequest }) =>
      runPlaybook(id, request),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(variables.id) });
    },
  });
}

// List runs for a playbook
export function usePlaybookRuns(playbookId: string) {
  return useQuery({
    queryKey: playbookKeys.runs(playbookId),
    queryFn: () => listPlaybookRuns(playbookId),
    enabled: !!playbookId,
  });
}

// Get single run with polling for active runs
export function usePlaybookRun(runId: string, refetchInterval?: number) {
  return useQuery({
    queryKey: playbookKeys.run(runId),
    queryFn: () => getPlaybookRun(runId),
    enabled: !!runId,
    refetchInterval: (query) => {
      const data = query.state.data;
      // Only poll if run is active
      if (data && ['pending', 'running', 'waiting_approval'].includes(data.status)) {
        return refetchInterval ?? 2000;
      }
      return false;
    },
  });
}

// Approve run
export function useApproveRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (runId: string) => approveRun(runId),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.run(data.id) });
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(data.playbook_id) });
    },
  });
}

// Reject run
export function useRejectRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (runId: string) => rejectRun(runId),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.run(data.id) });
      queryClient.invalidateQueries({ queryKey: playbookKeys.runs(data.playbook_id) });
    },
  });
}

// Re-export types for convenience
export type { Playbook, PlaybookSummary, PlaybookRun, CreatePlaybookRequest, RunPlaybookRequest };

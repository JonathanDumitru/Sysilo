import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listConnections,
  createConnection,
  updateConnection,
  activateConnection,
  deleteConnection,
  testConnection,
  type Connection,
  type CreateConnectionRequest,
  type UpdateConnectionRequest,
} from '../services/connections';

const CONNECTIONS_QUERY_KEY = ['connections'] as const;

export function useConnections() {
  return useQuery({
    queryKey: CONNECTIONS_QUERY_KEY,
    queryFn: listConnections,
    staleTime: 30_000,
  });
}

export function useCreateConnection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateConnectionRequest) => createConnection(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CONNECTIONS_QUERY_KEY });
    },
  });
}

export function useUpdateConnection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, request }: { id: string; request: UpdateConnectionRequest }) =>
      updateConnection(id, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CONNECTIONS_QUERY_KEY });
    },
  });
}

export function useDeleteConnection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: deleteConnection,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CONNECTIONS_QUERY_KEY });
    },
  });
}

export function useTestConnection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: testConnection,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CONNECTIONS_QUERY_KEY });
    },
  });
}

export function useActivateConnection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (connection: Connection) => activateConnection(connection),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CONNECTIONS_QUERY_KEY });
    },
  });
}

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listAlertInstances,
  acknowledgeAlert,
  resolveAlert,
  listAlertRules,
  createAlertRule,
  listIncidents,
  getIncident,
  createIncident,
  resolveIncident,
  listIncidentEvents,
  addIncidentEvent,
  getMetricAggregations,
  listNotificationChannels,
  type AlertStatus,
  type AlertSeverity,
  type IncidentStatus,
  type IncidentSeverity,
  type CreateAlertRuleRequest,
  type CreateIncidentRequest,
  type AddIncidentEventRequest,
  type MetricAggregationParams,
} from '../services/operations';

// --- Query Keys ---

const ALERT_INSTANCES_KEY = ['alert-instances'] as const;
const ALERT_RULES_KEY = ['alert-rules'] as const;
const INCIDENTS_KEY = ['incidents'] as const;
const INCIDENT_EVENTS_KEY = ['incident-events'] as const;
const METRIC_AGGREGATIONS_KEY = ['metric-aggregations'] as const;
const NOTIFICATION_CHANNELS_KEY = ['notification-channels'] as const;

// --- Alert Instances ---

export function useAlertInstances(status?: AlertStatus, severity?: AlertSeverity) {
  return useQuery({
    queryKey: [...ALERT_INSTANCES_KEY, status, severity],
    queryFn: () => listAlertInstances(status, severity),
    refetchInterval: 10_000,
  });
}

export function useAcknowledgeAlert() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => acknowledgeAlert(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ALERT_INSTANCES_KEY });
    },
  });
}

export function useResolveAlert() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => resolveAlert(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ALERT_INSTANCES_KEY });
    },
  });
}

// --- Alert Rules ---

export function useAlertRules() {
  return useQuery({
    queryKey: ALERT_RULES_KEY,
    queryFn: () => listAlertRules(),
  });
}

export function useCreateAlertRule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (rule: CreateAlertRuleRequest) => createAlertRule(rule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ALERT_RULES_KEY });
    },
  });
}

// --- Incidents ---

export function useIncidents(status?: IncidentStatus, severity?: IncidentSeverity) {
  return useQuery({
    queryKey: [...INCIDENTS_KEY, status, severity],
    queryFn: () => listIncidents(status, severity),
    refetchInterval: 15_000,
  });
}

export function useIncident(id: string) {
  return useQuery({
    queryKey: [...INCIDENTS_KEY, id],
    queryFn: () => getIncident(id),
    enabled: !!id,
  });
}

export function useCreateIncident() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateIncidentRequest) => createIncident(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: INCIDENTS_KEY });
    },
  });
}

export function useResolveIncident() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => resolveIncident(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: INCIDENTS_KEY });
    },
  });
}

// --- Incident Events ---

export function useIncidentEvents(incidentId: string) {
  return useQuery({
    queryKey: [...INCIDENT_EVENTS_KEY, incidentId],
    queryFn: () => listIncidentEvents(incidentId),
    enabled: !!incidentId,
  });
}

export function useAddIncidentEvent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ incidentId, data }: { incidentId: string; data: AddIncidentEventRequest }) =>
      addIncidentEvent(incidentId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: INCIDENT_EVENTS_KEY });
    },
  });
}

// --- Metrics ---

export function useMetricAggregations(params: MetricAggregationParams) {
  return useQuery({
    queryKey: [...METRIC_AGGREGATIONS_KEY, params],
    queryFn: () => getMetricAggregations(params),
    staleTime: 30_000,
  });
}

// --- Notification Channels ---

export function useNotificationChannels() {
  return useQuery({
    queryKey: NOTIFICATION_CHANNELS_KEY,
    queryFn: listNotificationChannels,
  });
}

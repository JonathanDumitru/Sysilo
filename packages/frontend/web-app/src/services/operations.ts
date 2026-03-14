import { getAuthContextHeaders } from '../config/env';
import { apiFetch } from './api';

// --- Types ---

export type AlertSeverity = 'critical' | 'high' | 'medium' | 'low' | 'info';
export type AlertStatus = 'firing' | 'resolved' | 'acknowledged' | 'silenced';

export interface AlertRule {
  id: string;
  tenant_id: string;
  name: string;
  metric_name: string;
  condition: 'gt' | 'lt' | 'eq' | 'gte' | 'lte' | 'ne';
  threshold: number;
  severity: AlertSeverity;
  enabled: boolean;
  notification_channels: string[];
  created_at: string;
  updated_at: string;
}

export interface AlertInstance {
  id: string;
  tenant_id: string;
  rule_id: string;
  rule_name: string;
  metric_name: string;
  metric_value: number;
  threshold: number;
  condition: string;
  severity: AlertSeverity;
  status: AlertStatus;
  acknowledged_by?: string;
  fired_at: string;
  acknowledged_at?: string;
  resolved_at?: string;
}

export type IncidentSeverity = 'critical' | 'high' | 'medium' | 'low' | 'info';
export type IncidentStatus = 'open' | 'acknowledged' | 'investigating' | 'resolved' | 'closed';

export interface Incident {
  id: string;
  tenant_id: string;
  title: string;
  description: string;
  severity: IncidentSeverity;
  status: IncidentStatus;
  assignee_id?: string;
  created_at: string;
  acknowledged_at?: string;
  resolved_at?: string;
  closed_at?: string;
}

export interface IncidentEvent {
  id: string;
  incident_id: string;
  event_type: string;
  description: string;
  actor_id?: string;
  created_at: string;
}

export interface Metric {
  id: string;
  tenant_id: string;
  resource_id: string;
  resource_type: string;
  metric_name: string;
  value: number;
  unit: string;
  tags: Record<string, string>;
  timestamp: string;
}

export interface MetricAggregation {
  metric_name: string;
  bucket: string;
  min_value: number;
  max_value: number;
  avg_value: number;
  count: number;
}

export interface NotificationChannel {
  id: string;
  tenant_id: string;
  name: string;
  channel_type: 'email' | 'slack' | 'webhook' | 'pagerduty' | 'teams' | 'opsgenie';
  config: Record<string, unknown>;
  enabled: boolean;
  severity_filter: AlertSeverity[];
  created_at: string;
}

// --- API Response Wrapper ---

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

function unwrap<T>(response: ApiResponse<T> | T): T {
  if (response && typeof response === 'object' && 'success' in response) {
    return (response as ApiResponse<T>).data!;
  }
  return response as T;
}

// --- Headers ---

const opsHeaders = getAuthContextHeaders();

// --- Alert Instances ---

export async function listAlertInstances(
  status?: AlertStatus,
  severity?: AlertSeverity,
  limit?: number
): Promise<AlertInstance[]> {
  const params = new URLSearchParams();
  if (status) params.set('status', status);
  if (severity) params.set('severity', severity);
  if (limit) params.set('limit', limit.toString());

  const query = params.toString();
  const url = `/ops/alerts/instances${query ? `?${query}` : ''}`;

  const resp = await apiFetch<ApiResponse<AlertInstance[]> | AlertInstance[]>(url, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export async function acknowledgeAlert(id: string): Promise<AlertInstance> {
  const resp = await apiFetch<ApiResponse<AlertInstance> | AlertInstance>(
    `/ops/alerts/instances/${id}/ack`,
    {
      method: 'POST',
      headers: opsHeaders,
    }
  );
  return unwrap(resp);
}

export async function resolveAlert(id: string): Promise<AlertInstance> {
  const resp = await apiFetch<ApiResponse<AlertInstance> | AlertInstance>(
    `/ops/alerts/instances/${id}/resolve`,
    {
      method: 'POST',
      headers: opsHeaders,
    }
  );
  return unwrap(resp);
}

// --- Alert Rules ---

export async function listAlertRules(enabledOnly?: boolean): Promise<AlertRule[]> {
  const params = new URLSearchParams();
  if (enabledOnly !== undefined) params.set('enabled_only', enabledOnly.toString());

  const query = params.toString();
  const url = `/ops/alerts/rules${query ? `?${query}` : ''}`;

  const resp = await apiFetch<ApiResponse<AlertRule[]> | AlertRule[]>(url, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export interface CreateAlertRuleRequest {
  name: string;
  metric_name: string;
  condition: 'gt' | 'lt' | 'eq' | 'gte' | 'lte' | 'ne';
  threshold: number;
  severity: AlertSeverity;
  enabled?: boolean;
  notification_channels?: string[];
}

export async function createAlertRule(rule: CreateAlertRuleRequest): Promise<AlertRule> {
  const resp = await apiFetch<ApiResponse<AlertRule> | AlertRule>('/ops/alerts/rules', {
    method: 'POST',
    headers: opsHeaders,
    body: JSON.stringify(rule),
  });
  return unwrap(resp);
}

// --- Incidents ---

export async function listIncidents(
  status?: IncidentStatus,
  severity?: IncidentSeverity,
  limit?: number,
  offset?: number
): Promise<Incident[]> {
  const params = new URLSearchParams();
  if (status) params.set('status', status);
  if (severity) params.set('severity', severity);
  if (limit) params.set('limit', limit.toString());
  if (offset) params.set('offset', offset.toString());

  const query = params.toString();
  const url = `/ops/incidents${query ? `?${query}` : ''}`;

  const resp = await apiFetch<ApiResponse<Incident[]> | Incident[]>(url, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export async function getIncident(id: string): Promise<Incident> {
  const resp = await apiFetch<ApiResponse<Incident> | Incident>(`/ops/incidents/${id}`, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export interface CreateIncidentRequest {
  title: string;
  description: string;
  severity: IncidentSeverity;
  assignee_id?: string;
}

export async function createIncident(data: CreateIncidentRequest): Promise<Incident> {
  const resp = await apiFetch<ApiResponse<Incident> | Incident>('/ops/incidents', {
    method: 'POST',
    headers: opsHeaders,
    body: JSON.stringify(data),
  });
  return unwrap(resp);
}

export async function resolveIncident(id: string): Promise<Incident> {
  const resp = await apiFetch<ApiResponse<Incident> | Incident>(
    `/ops/incidents/${id}/resolve`,
    {
      method: 'POST',
      headers: opsHeaders,
    }
  );
  return unwrap(resp);
}

// --- Incident Events ---

export async function listIncidentEvents(incidentId: string): Promise<IncidentEvent[]> {
  const resp = await apiFetch<ApiResponse<IncidentEvent[]> | IncidentEvent[]>(
    `/ops/incidents/${incidentId}/events`,
    {
      headers: opsHeaders,
    }
  );
  return unwrap(resp);
}

export interface AddIncidentEventRequest {
  event_type: string;
  description: string;
  actor_id?: string;
}

export async function addIncidentEvent(
  incidentId: string,
  data: AddIncidentEventRequest
): Promise<IncidentEvent> {
  const resp = await apiFetch<ApiResponse<IncidentEvent> | IncidentEvent>(
    `/ops/incidents/${incidentId}/events`,
    {
      method: 'POST',
      headers: opsHeaders,
      body: JSON.stringify(data),
    }
  );
  return unwrap(resp);
}

// --- Metrics ---

export interface QueryMetricsParams {
  resource_id?: string;
  resource_type?: string;
  metric_name?: string;
  start_time?: string;
  end_time?: string;
  limit?: number;
}

export async function queryMetrics(params: QueryMetricsParams): Promise<Metric[]> {
  const searchParams = new URLSearchParams();
  if (params.resource_id) searchParams.set('resource_id', params.resource_id);
  if (params.resource_type) searchParams.set('resource_type', params.resource_type);
  if (params.metric_name) searchParams.set('metric_name', params.metric_name);
  if (params.start_time) searchParams.set('start_time', params.start_time);
  if (params.end_time) searchParams.set('end_time', params.end_time);
  if (params.limit) searchParams.set('limit', params.limit.toString());

  const query = searchParams.toString();
  const url = `/ops/metrics${query ? `?${query}` : ''}`;

  const resp = await apiFetch<ApiResponse<Metric[]> | Metric[]>(url, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export interface MetricAggregationParams {
  metric_name?: string;
  resource_type?: string;
  start_time?: string;
  end_time?: string;
  bucket_interval?: string;
}

export async function getMetricAggregations(
  params: MetricAggregationParams
): Promise<MetricAggregation[]> {
  const searchParams = new URLSearchParams();
  if (params.metric_name) searchParams.set('metric_name', params.metric_name);
  if (params.resource_type) searchParams.set('resource_type', params.resource_type);
  if (params.start_time) searchParams.set('start_time', params.start_time);
  if (params.end_time) searchParams.set('end_time', params.end_time);
  if (params.bucket_interval) searchParams.set('bucket_interval', params.bucket_interval);

  const query = searchParams.toString();
  const url = `/ops/metrics/aggregations${query ? `?${query}` : ''}`;

  const resp = await apiFetch<ApiResponse<MetricAggregation[]> | MetricAggregation[]>(url, {
    headers: opsHeaders,
  });
  return unwrap(resp);
}

export interface IngestMetricsRequest {
  resource_id: string;
  resource_type: string;
  metric_name: string;
  value: number;
  unit: string;
  tags?: Record<string, string>;
}

export async function ingestMetrics(metrics: IngestMetricsRequest[]): Promise<void> {
  await apiFetch<ApiResponse<void> | void>('/ops/metrics', {
    method: 'POST',
    headers: opsHeaders,
    body: JSON.stringify(metrics),
  });
}

// --- Notification Channels ---

export async function listNotificationChannels(): Promise<NotificationChannel[]> {
  const resp = await apiFetch<ApiResponse<NotificationChannel[]> | NotificationChannel[]>(
    '/ops/notifications/channels',
    {
      headers: opsHeaders,
    }
  );
  return unwrap(resp);
}

export interface CreateNotificationChannelRequest {
  name: string;
  channel_type: NotificationChannel['channel_type'];
  config: Record<string, unknown>;
  enabled?: boolean;
  severity_filter?: AlertSeverity[];
}

export async function createNotificationChannel(
  data: CreateNotificationChannelRequest
): Promise<NotificationChannel> {
  const resp = await apiFetch<ApiResponse<NotificationChannel> | NotificationChannel>(
    '/ops/notifications/channels',
    {
      method: 'POST',
      headers: opsHeaders,
      body: JSON.stringify(data),
    }
  );
  return unwrap(resp);
}

export async function testNotificationChannel(id: string): Promise<void> {
  await apiFetch<ApiResponse<void> | void>(`/ops/notifications/test/${id}`, {
    method: 'POST',
    headers: opsHeaders,
  });
}

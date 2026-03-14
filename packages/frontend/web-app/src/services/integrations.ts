import { getAuthContextHeaders } from '../config/env';
import { apiFetch } from './api';

// --- Types ---

export type IntegrationStatus = 'active' | 'inactive' | 'draft' | 'error';

export interface IntegrationSummary {
  id: string;
  name: string;
  description: string;
  status: IntegrationStatus;
  created_at: string;
}

export interface IntegrationListResponse {
  integrations: IntegrationSummary[];
  total: number;
}

export interface IntegrationDetail {
  id: string;
  name: string;
  description: string;
  definition: Record<string, unknown>;
  version: number;
  status: IntegrationStatus;
  created_at: string;
}

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface IntegrationRun {
  id: string;
  integration_id: string;
  status: RunStatus;
  trigger_type: string;
  started_at: string;
  completed_at?: string;
  error_message?: string;
  metrics: Record<string, unknown> & {
    records_read?: number;
    records_written?: number;
    bytes_processed?: number;
    duration_ms?: number;
  };
}

export interface CreateIntegrationRequest {
  name: string;
  description: string;
  definition: Record<string, unknown>;
}

// --- API functions ---

const INTEGRATIONS_BASE_PATH = '/integrations/integrations';
const RUNS_BASE_PATH = '/integrations/runs';

const integrationHeaders = getAuthContextHeaders();

export async function listIntegrations(): Promise<IntegrationListResponse> {
  return apiFetch<IntegrationListResponse>(INTEGRATIONS_BASE_PATH, {
    headers: integrationHeaders,
  });
}

export async function getIntegration(id: string): Promise<IntegrationDetail> {
  return apiFetch<IntegrationDetail>(`${INTEGRATIONS_BASE_PATH}/${id}`, {
    headers: integrationHeaders,
  });
}

export async function createIntegration(data: CreateIntegrationRequest): Promise<IntegrationDetail> {
  return apiFetch<IntegrationDetail>(INTEGRATIONS_BASE_PATH, {
    method: 'POST',
    headers: integrationHeaders,
    body: JSON.stringify(data),
  });
}

export async function runIntegration(id: string): Promise<IntegrationRun> {
  return apiFetch<IntegrationRun>(`${INTEGRATIONS_BASE_PATH}/${id}/run`, {
    method: 'POST',
    headers: integrationHeaders,
  });
}

export async function getRun(id: string): Promise<IntegrationRun> {
  return apiFetch<IntegrationRun>(`${RUNS_BASE_PATH}/${id}`, {
    headers: integrationHeaders,
  });
}

export async function cancelRun(id: string): Promise<IntegrationRun> {
  return apiFetch<IntegrationRun>(`${RUNS_BASE_PATH}/${id}/cancel`, {
    method: 'POST',
    headers: integrationHeaders,
  });
}

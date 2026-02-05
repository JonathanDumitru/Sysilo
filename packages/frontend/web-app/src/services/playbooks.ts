import { apiFetch } from './api';

// Types matching backend
export type TriggerType = 'manual' | 'scheduled' | 'webhook' | 'event';
export type StepType = 'integration' | 'webhook' | 'wait' | 'condition' | 'approval';
export type RunStatus = 'pending' | 'running' | 'waiting_approval' | 'completed' | 'failed' | 'cancelled';
export type StepStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped';

export interface Variable {
  name: string;
  var_type: string;
  required: boolean;
  default_value?: string;
}

export interface Step {
  id: string;
  step_type: StepType;
  name: string;
  config: Record<string, unknown>;
  on_success: string[];
  on_failure: string[];
}

export interface StepState {
  step_id: string;
  status: StepStatus;
  started_at?: string;
  completed_at?: string;
  output?: Record<string, unknown>;
  error?: string;
}

export interface Playbook {
  id: string;
  name: string;
  description?: string;
  trigger_type: TriggerType;
  steps: Step[];
  variables: Variable[];
  created_at: string;
  updated_at: string;
}

export interface PlaybookSummary {
  id: string;
  name: string;
  description?: string;
  trigger_type: TriggerType;
  step_count: number;
  created_at: string;
  updated_at: string;
}

export interface PlaybookRun {
  id: string;
  playbook_id: string;
  status: RunStatus;
  variables: Record<string, unknown>;
  step_states: StepState[];
  started_at: string;
  completed_at?: string;
}

export interface ListPlaybooksResponse {
  playbooks: PlaybookSummary[];
  total: number;
}

export interface ListRunsResponse {
  runs: PlaybookRun[];
}

// API functions - Use /integrations prefix for the integration-service backend

export async function listPlaybooks(): Promise<ListPlaybooksResponse> {
  return apiFetch<ListPlaybooksResponse>('/integrations/playbooks');
}

export async function getPlaybook(id: string): Promise<Playbook> {
  return apiFetch<Playbook>(`/integrations/playbooks/${id}`);
}

export interface CreatePlaybookRequest {
  name: string;
  description?: string;
  trigger_type?: TriggerType;
  steps?: Step[];
  variables?: Variable[];
}

export async function createPlaybook(request: CreatePlaybookRequest): Promise<Playbook> {
  return apiFetch<Playbook>('/integrations/playbooks', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export async function updatePlaybook(id: string, request: CreatePlaybookRequest): Promise<Playbook> {
  return apiFetch<Playbook>(`/integrations/playbooks/${id}`, {
    method: 'PUT',
    body: JSON.stringify(request),
  });
}

export async function deletePlaybook(id: string): Promise<void> {
  await apiFetch(`/integrations/playbooks/${id}`, {
    method: 'DELETE',
  });
}

export interface RunPlaybookRequest {
  variables?: Record<string, unknown>;
}

export async function runPlaybook(id: string, request: RunPlaybookRequest = {}): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbooks/${id}/run`, {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export async function listPlaybookRuns(playbookId: string): Promise<ListRunsResponse> {
  return apiFetch<ListRunsResponse>(`/integrations/playbooks/${playbookId}/runs`);
}

export async function getPlaybookRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbook-runs/${runId}`);
}

export async function approveRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbook-runs/${runId}/approve`, {
    method: 'POST',
  });
}

export async function rejectRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbook-runs/${runId}/reject`, {
    method: 'POST',
  });
}

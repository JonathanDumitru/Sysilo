import { getAuthContextHeaders } from '../config/env';
import { apiFetch } from './api';

// --- Types ---

export type PolicyScope = 'integration' | 'connection' | 'agent' | 'asset' | 'data_entity' | 'all';
export type EnforcementMode = 'enforce' | 'warn' | 'audit';
export type PolicySeverity = 'critical' | 'high' | 'medium' | 'low' | 'info';

export interface Policy {
  id: string;
  tenant_id: string;
  name: string;
  description: string;
  rego_policy: string;
  scope: PolicyScope;
  enforcement_mode: EnforcementMode;
  severity: PolicySeverity;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface PolicyViolation {
  id: string;
  tenant_id: string;
  policy_id: string;
  resource_type: string;
  resource_id: string;
  details: Record<string, unknown>;
  status: 'open' | 'resolved';
  resolved_at?: string;
  resolved_by?: string;
  resolution_note?: string;
  created_at: string;
}

export interface PolicyEvaluationResult {
  policy_id: string;
  policy_name: string;
  passed: boolean;
  violations: string[];
  enforcement_mode: EnforcementMode;
  severity: PolicySeverity;
}

export interface ApprovalWorkflow {
  id: string;
  tenant_id: string;
  name: string;
  description: string;
  trigger_conditions: Record<string, unknown>;
  stages: unknown;
  auto_approve_conditions: unknown;
  enabled: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface ApprovalRequest {
  id: string;
  tenant_id: string;
  workflow_id: string;
  resource_type: string;
  resource_id: string;
  resource_snapshot: unknown;
  requester_id: string;
  current_stage: string;
  status: 'pending' | 'approved' | 'rejected' | 'cancelled' | 'expired';
  auto_approved: boolean;
  created_at: string;
  updated_at: string;
  completed_at?: string;
}

export interface ApprovalDecision {
  id: string;
  request_id: string;
  stage: string;
  approver_id: string;
  decision: 'approved' | 'rejected';
  comment?: string;
  decided_at: string;
}

export interface ApprovalRequestWithWorkflow {
  request: ApprovalRequest;
  workflow_name: string;
  current_stage_name: string;
  decisions: ApprovalDecision[];
}

export interface AuditEntry {
  id: string;
  tenant_id: string;
  actor_id: string;
  actor_type: string;
  actor_name: string;
  action: string;
  resource_type: string;
  resource_id: string;
  resource_name: string;
  before_state: unknown;
  after_state: unknown;
  metadata: Record<string, unknown>;
  ip_address: string;
  user_agent: string;
  timestamp: string;
  hash: string;
}

export interface AuditStats {
  total_entries: number;
  entries_today: number;
  unique_actors: number;
  top_actions: { action: string; count: number }[];
  top_resources: { resource_type: string; count: number }[];
}

export interface ComplianceFramework {
  id: string;
  name: string;
  description: string;
  version: string;
  controls: unknown;
  created_at: string;
  updated_at: string;
}

export interface AssessmentResult {
  framework_id: string;
  framework_name: string;
  total_controls: number;
  compliant: number;
  non_compliant: number;
  partial: number;
  not_assessed: number;
  not_applicable: number;
  compliance_score: number;
  assessed_at: string;
}

export interface Standard {
  id: string;
  tenant_id: string;
  name: string;
  category: string;
  description: string;
  rules: unknown;
  examples: unknown;
  version: string;
  is_active: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
}

// --- ApiResponse wrapper ---

interface ApiResponse<T> {
  success: boolean;
  data: T;
  error?: string;
}

async function unwrap<T>(promise: Promise<ApiResponse<T>>): Promise<T> {
  const response = await promise;
  if (!response.success) {
    throw new Error(response.error || 'Unknown governance API error');
  }
  return response.data;
}

// --- Shared headers ---

const governanceHeaders = getAuthContextHeaders();

// --- Policy APIs ---

export async function listPolicies(scope?: PolicyScope, enabledOnly?: boolean): Promise<Policy[]> {
  const params = new URLSearchParams();
  if (scope) params.set('scope', scope);
  if (enabledOnly !== undefined) params.set('enabled_only', String(enabledOnly));
  const query = params.toString();
  return unwrap(
    apiFetch<ApiResponse<Policy[]>>(`/governance/policies${query ? `?${query}` : ''}`, {
      headers: governanceHeaders,
    })
  );
}

export async function createPolicy(data: Omit<Policy, 'id' | 'tenant_id' | 'created_at' | 'updated_at'>): Promise<Policy> {
  return unwrap(
    apiFetch<ApiResponse<Policy>>('/governance/policies', {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify(data),
    })
  );
}

export async function updatePolicy(id: string, data: Partial<Omit<Policy, 'id' | 'tenant_id' | 'created_at' | 'updated_at'>>): Promise<Policy> {
  return unwrap(
    apiFetch<ApiResponse<Policy>>(`/governance/policies/${id}`, {
      method: 'PUT',
      headers: governanceHeaders,
      body: JSON.stringify(data),
    })
  );
}

export async function deletePolicy(id: string): Promise<void> {
  await unwrap(
    apiFetch<ApiResponse<null>>(`/governance/policies/${id}`, {
      method: 'DELETE',
      headers: governanceHeaders,
    })
  );
}

export async function evaluatePolicies(data: { resource_type: string; resource_id: string; resource_data: Record<string, unknown> }): Promise<PolicyEvaluationResult[]> {
  return unwrap(
    apiFetch<ApiResponse<PolicyEvaluationResult[]>>('/governance/policies/evaluate', {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify(data),
    })
  );
}

// --- Violation APIs ---

export async function listViolations(status?: string, policyId?: string, limit?: number, offset?: number): Promise<PolicyViolation[]> {
  const params = new URLSearchParams();
  if (status) params.set('status', status);
  if (policyId) params.set('policy_id', policyId);
  if (limit !== undefined) params.set('limit', String(limit));
  if (offset !== undefined) params.set('offset', String(offset));
  const query = params.toString();
  return unwrap(
    apiFetch<ApiResponse<PolicyViolation[]>>(`/governance/policies/violations${query ? `?${query}` : ''}`, {
      headers: governanceHeaders,
    })
  );
}

export async function resolveViolation(id: string, note: string): Promise<PolicyViolation> {
  return unwrap(
    apiFetch<ApiResponse<PolicyViolation>>(`/governance/policies/violations/${id}/resolve`, {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify({ resolution_note: note }),
    })
  );
}

// --- Approval Workflow APIs ---

export async function listApprovalWorkflows(): Promise<ApprovalWorkflow[]> {
  return unwrap(
    apiFetch<ApiResponse<ApprovalWorkflow[]>>('/governance/approvals/workflows', {
      headers: governanceHeaders,
    })
  );
}

export async function createApprovalWorkflow(data: Omit<ApprovalWorkflow, 'id' | 'tenant_id' | 'created_at' | 'updated_at'>): Promise<ApprovalWorkflow> {
  return unwrap(
    apiFetch<ApiResponse<ApprovalWorkflow>>('/governance/approvals/workflows', {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify(data),
    })
  );
}

// --- Approval Request APIs ---

export async function listApprovalRequests(status?: string, limit?: number, offset?: number): Promise<ApprovalRequestWithWorkflow[]> {
  const params = new URLSearchParams();
  if (status) params.set('status', status);
  if (limit !== undefined) params.set('limit', String(limit));
  if (offset !== undefined) params.set('offset', String(offset));
  const query = params.toString();
  return unwrap(
    apiFetch<ApiResponse<ApprovalRequestWithWorkflow[]>>(`/governance/approvals/requests${query ? `?${query}` : ''}`, {
      headers: governanceHeaders,
    })
  );
}

export async function getApprovalRequest(id: string): Promise<ApprovalRequestWithWorkflow> {
  return unwrap(
    apiFetch<ApiResponse<ApprovalRequestWithWorkflow>>(`/governance/approvals/requests/${id}`, {
      headers: governanceHeaders,
    })
  );
}

export async function createApprovalRequest(data: { workflow_id: string; resource_type: string; resource_id: string; resource_snapshot: unknown }): Promise<ApprovalRequest> {
  return unwrap(
    apiFetch<ApiResponse<ApprovalRequest>>('/governance/approvals/requests', {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify(data),
    })
  );
}

export async function decideApprovalRequest(id: string, decision: 'approved' | 'rejected', comment?: string): Promise<ApprovalDecision> {
  return unwrap(
    apiFetch<ApiResponse<ApprovalDecision>>(`/governance/approvals/requests/${id}/decide`, {
      method: 'POST',
      headers: governanceHeaders,
      body: JSON.stringify({ decision, comment }),
    })
  );
}

// --- Audit APIs ---

export async function queryAuditLog(params: {
  start_time?: string;
  end_time?: string;
  actor_id?: string;
  action?: string;
  resource_type?: string;
  resource_id?: string;
  limit?: number;
  offset?: number;
}): Promise<AuditEntry[]> {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) searchParams.set(key, String(value));
  }
  const query = searchParams.toString();
  return unwrap(
    apiFetch<ApiResponse<AuditEntry[]>>(`/governance/audit${query ? `?${query}` : ''}`, {
      headers: governanceHeaders,
    })
  );
}

export async function getAuditStats(): Promise<AuditStats> {
  return unwrap(
    apiFetch<ApiResponse<AuditStats>>('/governance/audit/stats', {
      headers: governanceHeaders,
    })
  );
}

export async function exportAuditLog(startTime: string, endTime: string): Promise<Blob> {
  const params = new URLSearchParams({ start_time: startTime, end_time: endTime });
  return apiFetch<Blob>(`/governance/audit/export?${params.toString()}`, {
    headers: governanceHeaders,
  });
}

// --- Compliance APIs ---

export async function getComplianceSummary(): Promise<AssessmentResult[]> {
  return unwrap(
    apiFetch<ApiResponse<AssessmentResult[]>>('/governance/compliance/frameworks/summary', {
      headers: governanceHeaders,
    })
  );
}

export async function runComplianceAssessment(frameworkId: string): Promise<AssessmentResult> {
  return unwrap(
    apiFetch<ApiResponse<AssessmentResult>>(`/governance/compliance/frameworks/${frameworkId}/assess`, {
      method: 'POST',
      headers: governanceHeaders,
    })
  );
}

// --- Standards APIs ---

export async function listStandards(category?: string, activeOnly?: boolean): Promise<Standard[]> {
  const params = new URLSearchParams();
  if (category) params.set('category', category);
  if (activeOnly !== undefined) params.set('active_only', String(activeOnly));
  const query = params.toString();
  return unwrap(
    apiFetch<ApiResponse<Standard[]>>(`/governance/standards${query ? `?${query}` : ''}`, {
      headers: governanceHeaders,
    })
  );
}

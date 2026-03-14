import { apiFetch } from './api';
import { getAuthContextHeaders } from '../config/env';

// Types matching rationalization-service backend

export type TimeQuadrant = 'tolerate' | 'invest' | 'migrate' | 'eliminate';

export interface Application {
  id: string;
  tenant_id: string;
  asset_id: string;
  name: string;
  description?: string;
  vendor?: string;
  version?: string;
  business_capability?: string;
  business_unit?: string;
  application_type?: string;
  criticality?: string;
  lifecycle_stage?: string;
  go_live_date?: string;
  sunset_date?: string;
  business_owner_id?: string;
  technical_owner_id?: string;
  license_cost?: number;
  infrastructure_cost?: number;
  support_cost?: number;
  development_cost?: number;
  total_cost?: number;
  technology_stack?: string[];
  hosting_model?: string;
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface TimeAssessment {
  id: string;
  tenant_id: string;
  application_id: string;
  quadrant: TimeQuadrant;
  business_value_score: number;
  technical_health_score: number;
  is_override: boolean;
  override_reason?: string;
  recommended_actions?: string[];
  assessed_by?: string;
  assessed_at: string;
}

export interface TimeSummary {
  tolerate: number;
  invest: number;
  migrate: number;
  eliminate: number;
  total_applications: number;
}

export interface Scenario {
  id: string;
  tenant_id: string;
  name: string;
  description?: string;
  scenario_type?: string;
  status?: string;
  affected_applications?: string[];
  assumptions?: string;
  current_state?: string;
  projected_state?: string;
  implementation_cost?: number;
  annual_savings?: number;
  payback_months?: number;
  npv?: number;
  roi_percent?: number;
  risk_level?: string;
  risk_factors?: string[];
  estimated_duration_months?: number;
  proposed_start_date?: string;
  created_by?: string;
  created_at: string;
  updated_at: string;
}

export interface Recommendation {
  id: string;
  tenant_id: string;
  application_id?: string;
  scenario_id?: string;
  recommendation_type?: string;
  title: string;
  summary?: string;
  detailed_analysis?: string;
  confidence_score?: number;
  reasoning?: string;
  supporting_data?: Record<string, unknown>;
  estimated_savings?: number;
  estimated_effort?: string;
  risk_assessment?: string;
  status?: string;
  user_feedback?: string;
  generated_at: string;
  reviewed_by?: string;
  reviewed_at?: string;
}

export interface PortfolioAnalytics {
  total_applications: number;
  by_lifecycle: Record<string, number>;
  by_criticality: Record<string, number>;
  by_quadrant: Record<string, number>;
  total_cost: number;
  avg_health_score: number;
  avg_value_score: number;
}

// Request params

export interface ListApplicationsParams {
  lifecycle_stage?: string;
  criticality?: string;
  business_unit?: string;
}

export interface CreateScenarioRequest {
  name: string;
  description?: string;
  scenario_type?: string;
  affected_applications?: string[];
  assumptions?: string;
  estimated_duration_months?: number;
  proposed_start_date?: string;
}

// Helper to build headers with tenant context
function authHeaders(): Record<string, string> {
  return getAuthContextHeaders();
}

// Helper to build query string with tenant_id
function buildParams(extra: Record<string, string | undefined> = {}): string {
  const headers = getAuthContextHeaders();
  const params = new URLSearchParams();
  params.set('tenant_id', headers['X-Tenant-ID']);

  for (const [key, value] of Object.entries(extra)) {
    if (value) params.set(key, value);
  }

  return params.toString();
}

// =============================================================================
// Applications
// =============================================================================

export async function listApplications(
  params: ListApplicationsParams = {}
): Promise<Application[]> {
  const qs = buildParams({
    lifecycle_stage: params.lifecycle_stage,
    criticality: params.criticality,
    business_unit: params.business_unit,
  });
  return apiFetch<Application[]>(`/rationalization/applications?${qs}`, {
    headers: authHeaders(),
  });
}

export async function getApplication(id: string): Promise<Application> {
  const qs = buildParams();
  return apiFetch<Application>(`/rationalization/applications/${id}?${qs}`, {
    headers: authHeaders(),
  });
}

// =============================================================================
// TIME Assessments
// =============================================================================

export async function getTimeSummary(): Promise<TimeSummary> {
  const qs = buildParams();
  return apiFetch<TimeSummary>(`/rationalization/time/summary?${qs}`, {
    headers: authHeaders(),
  });
}

export async function listTimeAssessments(): Promise<TimeAssessment[]> {
  const qs = buildParams();
  return apiFetch<TimeAssessment[]>(`/rationalization/time/assessments?${qs}`, {
    headers: authHeaders(),
  });
}

export async function getTimeAssessment(id: string): Promise<TimeAssessment> {
  const qs = buildParams();
  return apiFetch<TimeAssessment>(`/rationalization/time/assessments/${id}?${qs}`, {
    headers: authHeaders(),
  });
}

// =============================================================================
// Scoring
// =============================================================================

export async function calculateTimeScore(applicationId: string): Promise<TimeAssessment> {
  const qs = buildParams();
  return apiFetch<TimeAssessment>(`/rationalization/scoring/calculate/${applicationId}?${qs}`, {
    method: 'POST',
    headers: authHeaders(),
  });
}

export async function bulkCalculateScores(): Promise<TimeAssessment[]> {
  const qs = buildParams();
  return apiFetch<TimeAssessment[]>(`/rationalization/scoring/bulk-calculate?${qs}`, {
    method: 'POST',
    headers: authHeaders(),
  });
}

// =============================================================================
// Scenarios
// =============================================================================

export async function listScenarios(): Promise<Scenario[]> {
  const qs = buildParams();
  return apiFetch<Scenario[]>(`/rationalization/scenarios?${qs}`, {
    headers: authHeaders(),
  });
}

export async function getScenario(id: string): Promise<Scenario> {
  const qs = buildParams();
  return apiFetch<Scenario>(`/rationalization/scenarios/${id}?${qs}`, {
    headers: authHeaders(),
  });
}

export async function createScenario(request: CreateScenarioRequest): Promise<Scenario> {
  const qs = buildParams();
  return apiFetch<Scenario>(`/rationalization/scenarios?${qs}`, {
    method: 'POST',
    headers: authHeaders(),
    body: JSON.stringify(request),
  });
}

export async function analyzeScenario(id: string): Promise<Scenario> {
  const qs = buildParams();
  return apiFetch<Scenario>(`/rationalization/scenarios/${id}/analyze?${qs}`, {
    method: 'POST',
    headers: authHeaders(),
  });
}

// =============================================================================
// Recommendations
// =============================================================================

export async function listRecommendations(): Promise<Recommendation[]> {
  const qs = buildParams();
  return apiFetch<Recommendation[]>(`/rationalization/recommendations?${qs}`, {
    headers: authHeaders(),
  });
}

export async function generateRecommendations(): Promise<Recommendation[]> {
  const qs = buildParams();
  return apiFetch<Recommendation[]>(`/rationalization/recommendations/generate?${qs}`, {
    method: 'POST',
    headers: authHeaders(),
  });
}

// =============================================================================
// Analytics
// =============================================================================

export async function getPortfolioAnalytics(): Promise<PortfolioAnalytics> {
  const qs = buildParams();
  return apiFetch<PortfolioAnalytics>(`/rationalization/analytics/portfolio?${qs}`, {
    headers: authHeaders(),
  });
}

export async function getCostAnalysis(): Promise<Record<string, unknown>> {
  const qs = buildParams();
  return apiFetch<Record<string, unknown>>(`/rationalization/analytics/cost-analysis?${qs}`, {
    headers: authHeaders(),
  });
}

export async function getScoreTrends(): Promise<Record<string, unknown>> {
  const qs = buildParams();
  return apiFetch<Record<string, unknown>>(`/rationalization/analytics/trends?${qs}`, {
    headers: authHeaders(),
  });
}

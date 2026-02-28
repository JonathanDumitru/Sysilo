import { apiFetch } from './api';

// Types

export interface Plan {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  price_cents: number;
  billing_interval: string;
  is_active: boolean;
  limits: PlanLimits;
  features: PlanFeatures;
}

export interface PlanLimits {
  max_users: number;
  max_integrations: number;
  max_connections: number;
  max_playbooks: number;
  max_runs_per_month: number;
  max_agents: number;
  audit_retention_days: number;
}

export interface PlanFeatures {
  governance_enabled: boolean;
  governance_level?: string;
  compliance_enabled: boolean;
  rationalization_enabled: boolean;
  ai_enabled: boolean;
  ai_level?: string;
  advanced_ops_enabled: boolean;
  ops_level?: string;
}

export interface TenantPlan {
  tenant_id: string;
  plan_id?: string;
  plan_status: string;
  trial_ends_at?: string;
  billing_email?: string;
  stripe_customer_id?: string;
  stripe_subscription_id?: string;
  plan?: Plan;
}

export interface UsageResponse {
  period: {
    id: string;
    period_start: string;
    period_end: string;
    integration_runs: number;
    active_users: number;
    data_bytes_processed: number;
  };
  resources: Record<string, number>;
}

// API calls

export async function getCurrentPlan(): Promise<TenantPlan> {
  return apiFetch<TenantPlan>('/api/v1/plan');
}

export async function getPlanUsage(): Promise<UsageResponse> {
  return apiFetch<UsageResponse>('/api/v1/plan/usage');
}

export async function listPlans(): Promise<{ plans: Plan[] }> {
  return apiFetch<{ plans: Plan[] }>('/api/v1/plans');
}

export async function createCheckoutSession(planName: string): Promise<{ checkout_url: string; session_id: string }> {
  return apiFetch('/api/v1/billing/checkout', {
    method: 'POST',
    body: JSON.stringify({
      plan_name: planName,
      success_url: `${window.location.origin}/settings?tab=billing&status=success`,
      cancel_url: `${window.location.origin}/settings?tab=billing&status=cancelled`,
    }),
  });
}

export async function createPortalSession(): Promise<{ portal_url: string }> {
  return apiFetch('/api/v1/billing/portal', { method: 'POST' });
}

export async function getSubscription(): Promise<TenantPlan> {
  return apiFetch<TenantPlan>('/api/v1/billing/subscription');
}

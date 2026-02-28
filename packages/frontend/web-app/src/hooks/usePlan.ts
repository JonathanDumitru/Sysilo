import { useQuery } from '@tanstack/react-query';
import { getCurrentPlan, getPlanUsage, type TenantPlan, type PlanFeatures, type PlanLimits } from '../services/billing';

export const planKeys = {
  all: ['plan'] as const,
  current: () => [...planKeys.all, 'current'] as const,
  usage: () => [...planKeys.all, 'usage'] as const,
};

export function usePlan() {
  const { data, isLoading, error } = useQuery({
    queryKey: planKeys.current(),
    queryFn: getCurrentPlan,
    staleTime: 60_000,
  });

  const plan = data?.plan;
  const features: PlanFeatures = plan?.features ?? {
    governance_enabled: false,
    compliance_enabled: false,
    rationalization_enabled: false,
    ai_enabled: false,
    advanced_ops_enabled: false,
  };
  const limits: PlanLimits = plan?.limits ?? {
    max_users: 5,
    max_integrations: 10,
    max_connections: 5,
    max_playbooks: 5,
    max_runs_per_month: 500,
    max_agents: 1,
    audit_retention_days: 30,
  };

  const hasFeature = (key: keyof PlanFeatures): boolean => {
    return !!features[key];
  };

  const isUnlimited = (val: number): boolean => val < 0;

  const planName = plan?.display_name ?? 'Trial';
  const planStatus = data?.plan_status ?? 'trial';
  const isTrial = planStatus === 'trial';
  const isSuspended = planStatus === 'suspended';
  const trialEndsAt = data?.trial_ends_at ? new Date(data.trial_ends_at) : null;

  const trialDaysLeft = trialEndsAt
    ? Math.max(0, Math.ceil((trialEndsAt.getTime() - Date.now()) / (1000 * 60 * 60 * 24)))
    : null;

  return {
    data,
    plan,
    planName,
    planStatus,
    features,
    limits,
    hasFeature,
    isUnlimited,
    isTrial,
    isSuspended,
    trialEndsAt,
    trialDaysLeft,
    isLoading,
    error,
  };
}

export function usePlanUsage() {
  return useQuery({
    queryKey: planKeys.usage(),
    queryFn: getPlanUsage,
    staleTime: 30_000,
  });
}

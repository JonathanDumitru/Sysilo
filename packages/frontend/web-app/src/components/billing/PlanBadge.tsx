import { usePlan } from '../../hooks/usePlan';

const planColors: Record<string, string> = {
  trial: 'bg-amber-100 text-amber-700',
  team: 'bg-blue-100 text-blue-700',
  business: 'bg-purple-100 text-purple-700',
  enterprise: 'bg-emerald-100 text-emerald-700',
};

export function PlanBadge() {
  const { plan, planStatus } = usePlan();
  const name = plan?.name ?? 'trial';
  const displayName = plan?.display_name ?? 'Trial';
  const colorClass = planColors[name] || planColors.trial;

  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${colorClass}`}>
      {displayName}
      {planStatus === 'trial' && ' Trial'}
    </span>
  );
}

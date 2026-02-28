import { usePlan, usePlanUsage } from '../../hooks/usePlan';

interface UsageMeterBarProps {
  label: string;
  current: number;
  max: number;
}

function UsageMeterBar({ label, current, max }: UsageMeterBarProps) {
  const isUnlimited = max < 0;
  const percentage = isUnlimited ? 0 : Math.min(100, (current / max) * 100);
  const isWarning = !isUnlimited && percentage >= 80;
  const isCritical = !isUnlimited && percentage >= 95;

  return (
    <div>
      <div className="flex items-center justify-between text-sm mb-1">
        <span className="text-gray-600">{label}</span>
        <span className="font-medium text-gray-900">
          {current} / {isUnlimited ? '∞' : max}
        </span>
      </div>
      {!isUnlimited && (
        <div className="h-2 bg-gray-100 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all ${
              isCritical ? 'bg-red-500' : isWarning ? 'bg-amber-500' : 'bg-primary-500'
            }`}
            style={{ width: `${percentage}%` }}
          />
        </div>
      )}
    </div>
  );
}

export function UsageMeter() {
  const { limits, isUnlimited } = usePlan();
  const { data: usage, isLoading } = usePlanUsage();

  if (isLoading || !usage) return null;

  const resources = usage.resources;

  return (
    <div className="space-y-3">
      <UsageMeterBar label="Integrations" current={resources.integrations ?? 0} max={limits.max_integrations} />
      <UsageMeterBar label="Connections" current={resources.connections ?? 0} max={limits.max_connections} />
      <UsageMeterBar label="Users" current={resources.users ?? 0} max={limits.max_users} />
      <UsageMeterBar label="Runs this month" current={resources.runs_this_month ?? 0} max={limits.max_runs_per_month} />
      <UsageMeterBar label="Agents" current={resources.agents ?? 0} max={limits.max_agents} />
    </div>
  );
}

import { Workflow, Bot, Database, AlertTriangle, TrendingUp } from 'lucide-react';

interface FloatingStatsPanelProps {
  totalAssets: number;
  activeIntegrations: number;
  runningAgents: number;
  openAlerts: number;
}

interface StatCard {
  label: string;
  value: number;
  icon: React.ElementType;
  trend: string;
  trendUp: boolean;
}

export function FloatingStatsPanel({
  totalAssets,
  activeIntegrations,
  runningAgents,
  openAlerts,
}: FloatingStatsPanelProps) {
  const cards: StatCard[] = [
    {
      label: 'Assets',
      value: totalAssets,
      icon: Database,
      trend: '+2 this week',
      trendUp: true,
    },
    {
      label: 'Integrations',
      value: activeIntegrations,
      icon: Workflow,
      trend: 'All active',
      trendUp: true,
    },
    {
      label: 'Agents',
      value: runningAgents,
      icon: Bot,
      trend: `${runningAgents} online`,
      trendUp: runningAgents > 0,
    },
    {
      label: 'Alerts',
      value: openAlerts,
      icon: AlertTriangle,
      trend: openAlerts > 0 ? `${openAlerts} open` : 'None',
      trendUp: openAlerts === 0,
    },
  ];

  return (
    <div className="glass-panel p-3 grid grid-cols-2 gap-2 min-w-[240px]">
      {cards.map((card) => {
        const Icon = card.icon;
        return (
          <div
            key={card.label}
            className="glass-card px-3 py-2.5 flex flex-col gap-1"
          >
            <div className="flex items-center justify-between">
              <Icon className="w-3.5 h-3.5 text-gray-400" />
              <div className="flex items-center gap-1 text-[10px]">
                <TrendingUp
                  className={`w-2.5 h-2.5 ${
                    card.trendUp ? 'text-status-healthy' : 'text-status-critical'
                  }`}
                />
                <span className={card.trendUp ? 'text-status-healthy' : 'text-status-critical'}>
                  {card.trend}
                </span>
              </div>
            </div>
            <div className="text-xl font-bold text-gray-100 leading-none">
              {card.value}
            </div>
            <div className="text-[10px] text-gray-500 font-medium uppercase tracking-wider">
              {card.label}
            </div>
          </div>
        );
      })}
    </div>
  );
}

import { useEffect, useCallback } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Shield,
  Clock,
  Zap,
  CheckCircle,
  Eye,
  Users,
  X,
} from 'lucide-react';
import { useAlertInstances, useAcknowledgeAlert, useResolveAlert } from '../../hooks/useOperations';
import { useApprovalRequests, useDecideApproval } from '../../hooks/useGovernance';
import { useStatusBar } from '../../hooks/useStatusBar';

type TabKey = 'critical' | 'warnings' | 'governance';

const tabs: { key: TabKey; label: string; icon: React.ElementType }[] = [
  { key: 'critical', label: 'Critical', icon: AlertCircle },
  { key: 'warnings', label: 'Warnings', icon: AlertTriangle },
  { key: 'governance', label: 'Governance', icon: Shield },
];

function formatTimeAgo(dateStr: string) {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

export function TriageDrawer() {
  const { isDrawerOpen, activeTab, setActiveTab, closeDrawer } = useStatusBar();

  const { data: allAlerts } = useAlertInstances();
  const { data: pendingApprovals } = useApprovalRequests('pending');
  const acknowledgeAlert = useAcknowledgeAlert();
  const resolveAlertMutation = useResolveAlert();
  const decideApproval = useDecideApproval();

  // Escape key handler
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isDrawerOpen) {
        closeDrawer();
      }
    },
    [isDrawerOpen, closeDrawer]
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  if (!isDrawerOpen) return null;

  const firingAlerts = (allAlerts ?? []).filter((a) => a.status === 'firing');
  const criticalAlerts = firingAlerts.filter((a) => a.severity === 'critical');
  const warningAlerts = firingAlerts.filter(
    (a) => a.severity === 'medium' || a.severity === 'high' || a.severity === 'low'
  );
  const governanceItems = pendingApprovals ?? [];

  const tabCounts: Record<TabKey, number> = {
    critical: criticalAlerts.length,
    warnings: warningAlerts.length,
    governance: governanceItems.length,
  };

  return (
    <div className="relative z-20 w-full">
      <div className="max-h-[50vh] bg-surface-raised/95 backdrop-blur-sm border-t border-surface-border rounded-t-xl overflow-hidden flex flex-col animate-in slide-in-from-bottom duration-200">
        {/* Header with tabs */}
        <div className="flex items-center justify-between px-4 pt-3 pb-0">
          <div className="flex items-center gap-1">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              const count = tabCounts[tab.key];
              const isActive = activeTab === tab.key;
              return (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key)}
                  className={`flex items-center gap-2 px-3 py-2 text-xs font-medium rounded-t-lg transition-colors ${
                    isActive
                      ? 'bg-surface-overlay text-gray-200 border-b-2 border-primary-500'
                      : 'text-gray-400 hover:text-gray-300 hover:bg-surface-overlay/50'
                  }`}
                >
                  <Icon className="w-3.5 h-3.5" />
                  {tab.label}
                  {count > 0 && (
                    <span
                      className={`px-1.5 py-0.5 rounded-full text-[10px] font-bold ${
                        tab.key === 'critical'
                          ? 'bg-red-500/20 text-red-400'
                          : tab.key === 'warnings'
                            ? 'bg-amber-500/20 text-amber-400'
                            : 'bg-blue-500/20 text-blue-400'
                      }`}
                    >
                      {count}
                    </span>
                  )}
                </button>
              );
            })}
          </div>
          <button
            onClick={closeDrawer}
            className="p-1.5 text-gray-500 hover:text-gray-300 rounded-lg hover:bg-surface-overlay/50"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {activeTab === 'critical' && (
            <AlertList
              alerts={criticalAlerts}
              emptyLabel="No critical alerts"
              onAcknowledge={(id) => acknowledgeAlert.mutate(id)}
              onResolve={(id) => resolveAlertMutation.mutate(id)}
            />
          )}
          {activeTab === 'warnings' && (
            <AlertList
              alerts={warningAlerts}
              emptyLabel="No warning alerts"
              onAcknowledge={(id) => acknowledgeAlert.mutate(id)}
              onResolve={(id) => resolveAlertMutation.mutate(id)}
            />
          )}
          {activeTab === 'governance' && (
            <GovernanceList
              items={governanceItems}
              emptyLabel="No governance pending"
              onApprove={(id) => decideApproval.mutate({ id, decision: 'approved' })}
              onReject={(id) => decideApproval.mutate({ id, decision: 'rejected' })}
            />
          )}
        </div>
      </div>
    </div>
  );
}

// --- Sub-components ---

interface AlertListProps {
  alerts: Array<{
    id: string;
    rule_name: string;
    severity: string;
    metric_name: string;
    metric_value: number;
    threshold: number;
    fired_at: string;
    status: string;
  }>;
  emptyLabel: string;
  onAcknowledge: (id: string) => void;
  onResolve: (id: string) => void;
}

function AlertList({ alerts, emptyLabel, onAcknowledge, onResolve }: AlertListProps) {
  if (alerts.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-gray-500">
        <CheckCircle className="w-8 h-8 mb-2 text-green-500/50" />
        <p className="text-sm">{emptyLabel}</p>
      </div>
    );
  }

  return (
    <>
      {alerts.map((alert) => {
        const severityColor =
          alert.severity === 'critical' ? 'text-red-400' : 'text-amber-400';
        const severityIcon =
          alert.severity === 'critical' ? AlertCircle : AlertTriangle;
        const Icon = severityIcon;

        return (
          <div
            key={alert.id}
            className="flex items-start gap-3 p-3 bg-surface-overlay/50 rounded-lg border border-surface-border hover:border-surface-border/80 transition-colors"
          >
            <Icon className={`w-4 h-4 mt-0.5 flex-shrink-0 ${severityColor}`} />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-gray-200 truncate">
                  {alert.rule_name}
                </span>
                <span className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-orange-500/10 text-orange-400">
                  <Zap className="w-2.5 h-2.5" />
                  blast radius
                </span>
              </div>
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <span className="truncate">
                  {alert.metric_name}: {alert.metric_value} (threshold: {alert.threshold})
                </span>
                <span className="flex items-center gap-1 flex-shrink-0">
                  <Clock className="w-3 h-3" />
                  {formatTimeAgo(alert.fired_at)}
                </span>
              </div>
            </div>
            <div className="flex items-center gap-1.5 flex-shrink-0">
              <button
                onClick={() => onAcknowledge(alert.id)}
                className="px-2 py-1 text-[11px] font-medium rounded bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 transition-colors"
              >
                <span className="flex items-center gap-1">
                  <Eye className="w-3 h-3" />
                  Ack
                </span>
              </button>
              <button
                onClick={() => onResolve(alert.id)}
                className="px-2 py-1 text-[11px] font-medium rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors"
              >
                <span className="flex items-center gap-1">
                  <CheckCircle className="w-3 h-3" />
                  Resolve
                </span>
              </button>
            </div>
          </div>
        );
      })}
    </>
  );
}

interface GovernanceItem {
  id: string;
  title?: string;
  description?: string;
  resource_type?: string;
  created_at?: string;
  status?: string;
}

interface GovernanceListProps {
  items: GovernanceItem[];
  emptyLabel: string;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
}

function GovernanceList({ items, emptyLabel, onApprove, onReject }: GovernanceListProps) {
  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-gray-500">
        <Shield className="w-8 h-8 mb-2 text-blue-500/50" />
        <p className="text-sm">{emptyLabel}</p>
      </div>
    );
  }

  return (
    <>
      {items.map((item) => (
        <div
          key={item.id}
          className="flex items-start gap-3 p-3 bg-surface-overlay/50 rounded-lg border border-surface-border hover:border-surface-border/80 transition-colors"
        >
          <Shield className="w-4 h-4 mt-0.5 flex-shrink-0 text-blue-400" />
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-sm font-medium text-gray-200 truncate">
                {item.title || item.description || `Approval ${item.id.slice(0, 8)}`}
              </span>
              {item.resource_type && (
                <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-blue-500/10 text-blue-400">
                  {item.resource_type}
                </span>
              )}
            </div>
            <div className="flex items-center gap-3 text-xs text-gray-500">
              {item.description && (
                <span className="truncate">{item.description}</span>
              )}
              {item.created_at && (
                <span className="flex items-center gap-1 flex-shrink-0">
                  <Clock className="w-3 h-3" />
                  {formatTimeAgo(item.created_at)}
                </span>
              )}
            </div>
          </div>
          <div className="flex items-center gap-1.5 flex-shrink-0">
            <button
              onClick={() => onApprove(item.id)}
              className="px-2 py-1 text-[11px] font-medium rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors"
            >
              Approve
            </button>
            <button
              onClick={() => onReject(item.id)}
              className="px-2 py-1 text-[11px] font-medium rounded bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors"
            >
              Reject
            </button>
            <button className="px-2 py-1 text-[11px] font-medium rounded bg-surface-overlay text-gray-400 hover:bg-surface-border transition-colors">
              <span className="flex items-center gap-1">
                <Users className="w-3 h-3" />
                Delegate
              </span>
            </button>
          </div>
        </div>
      ))}
    </>
  );
}

import { useEffect, useRef, useCallback } from 'react';
import {
  X,
  Workflow,
  Link2,
  Database,
  Bot,
  Play,
  GitBranch,
  ShieldCheck,
  Sparkles,
  Clock,
  AlertTriangle,
  BarChart3,
} from 'lucide-react';
import type {
  TopologyNodeData,
  NodeStatus,
  GovernanceStatus,
  TimeQuadrant,
} from '../../hooks/useTopologyGraph.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const TYPE_ICONS: Record<string, React.ElementType> = {
  integration: Workflow,
  connection: Link2,
  asset: Database,
  agent: Bot,
};

const STATUS_LABEL: Record<NodeStatus, { text: string; cls: string }> = {
  healthy: { text: 'Healthy', cls: 'bg-status-healthy/15 text-status-healthy' },
  warning: { text: 'Warning', cls: 'bg-status-warning/15 text-status-warning' },
  critical: { text: 'Critical', cls: 'bg-status-critical/15 text-status-critical' },
  inactive: { text: 'Inactive', cls: 'bg-gray-600/20 text-gray-400' },
};

const GOV_LABEL: Record<GovernanceStatus, { text: string; cls: string }> = {
  compliant: { text: 'Compliant', cls: 'text-status-governance' },
  violation: { text: 'Violation', cls: 'text-status-critical' },
  uncovered: { text: 'Uncovered', cls: 'text-gray-400' },
};

const TIME_LABEL: Record<TimeQuadrant, { text: string; cls: string }> = {
  invest: { text: 'Invest', cls: 'text-status-healthy' },
  tolerate: { text: 'Tolerate', cls: 'text-status-info' },
  migrate: { text: 'Migrate', cls: 'text-status-warning' },
  eliminate: { text: 'Eliminate', cls: 'text-status-critical' },
};

// ---------------------------------------------------------------------------
// Quick action buttons
// ---------------------------------------------------------------------------

interface QuickAction {
  label: string;
  icon: React.ElementType;
}

function getQuickActions(type: string): QuickAction[] {
  switch (type) {
    case 'integration':
      return [
        { label: 'Run Integration', icon: Play },
        { label: 'View Lineage', icon: GitBranch },
        { label: 'Check Policy', icon: ShieldCheck },
      ];
    case 'agent':
      return [
        { label: 'View Logs', icon: BarChart3 },
        { label: 'Restart Agent', icon: Play },
      ];
    case 'asset':
      return [
        { label: 'View Lineage', icon: GitBranch },
        { label: 'Check Policy', icon: ShieldCheck },
      ];
    case 'connection':
      return [
        { label: 'Test Connection', icon: Play },
        { label: 'View Lineage', icon: GitBranch },
      ];
    default:
      return [];
  }
}

// ---------------------------------------------------------------------------
// AI placeholder summaries
// ---------------------------------------------------------------------------

const AI_SUMMARIES: Record<string, string> = {
  integration:
    'This integration is performing within expected parameters. Data throughput has been consistent over the past 24 hours with no anomalies detected.',
  connection:
    'Connection latency is within acceptable thresholds. Last health check passed all diagnostics. Consider upgrading driver version for improved performance.',
  asset:
    'This data asset has 3 downstream consumers and is refreshed every 15 minutes. Data quality score is 94% based on the latest validation run.',
  agent:
    'Agent resource utilization is nominal. Memory usage at 42%, CPU at 18%. Next scheduled maintenance window is in 6 days.',
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface NodeDetailDrawerProps {
  node: TopologyNodeData | null;
  onClose: () => void;
}

export function NodeDetailDrawer({ node, onClose }: NodeDetailDrawerProps) {
  const panelRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    },
    [onClose],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Click outside to close
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as HTMLElement)) {
        onClose();
      }
    },
    [onClose],
  );

  if (!node) return null;

  const Icon = TYPE_ICONS[node.type] ?? Workflow;
  const statusInfo = STATUS_LABEL[node.status];
  const govInfo = GOV_LABEL[node.governanceStatus ?? 'uncovered'];
  const timeInfo = TIME_LABEL[node.timeQuadrant ?? 'tolerate'];
  const quickActions = getQuickActions(node.type);
  const aiSummary = AI_SUMMARIES[node.type] ?? AI_SUMMARIES.integration;

  return (
    <div
      className="fixed inset-0 z-50 flex justify-end"
      onClick={handleBackdropClick}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/30 backdrop-blur-sm" />

      {/* Panel */}
      <div
        ref={panelRef}
        className="relative w-full max-w-md h-full glass-panel-strong rounded-l-2xl border-l border-surface-border-strong overflow-y-auto animate-[slideInRight_0.25s_ease-out]"
      >
        {/* Header */}
        <div className="sticky top-0 z-10 flex items-center justify-between px-6 py-4 border-b border-surface-border bg-surface-overlay/90 backdrop-blur-glass">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-surface-raised border border-surface-border">
              <Icon className="w-5 h-5 text-gray-300" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-gray-100">{node.name}</h2>
              <p className="text-xs text-gray-500 capitalize">{node.type}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-surface-raised text-gray-400 hover:text-gray-200 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-5 space-y-6">
          {/* Status row */}
          <div className="grid grid-cols-2 gap-3">
            <div className="glass-card px-3 py-2.5">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Status</div>
              <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${statusInfo.cls}`}>
                {statusInfo.text}
              </span>
            </div>
            <div className="glass-card px-3 py-2.5">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Last Activity</div>
              <div className="flex items-center gap-1 text-xs text-gray-300">
                <Clock className="w-3 h-3 text-gray-500" />
                {node.lastActivity ?? 'Unknown'}
              </div>
            </div>
          </div>

          {/* Metrics row */}
          <div className="grid grid-cols-3 gap-3">
            <div className="glass-card px-3 py-2.5 text-center">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Errors</div>
              <div className="flex items-center justify-center gap-1">
                <AlertTriangle className="w-3 h-3 text-gray-500" />
                <span className={`text-sm font-bold ${(node.errorCount ?? 0) > 0 ? 'text-status-critical' : 'text-gray-300'}`}>
                  {node.errorCount ?? 0}
                </span>
              </div>
            </div>
            <div className="glass-card px-3 py-2.5 text-center">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">TIME</div>
              <span className={`text-sm font-bold ${timeInfo.cls}`}>
                {timeInfo.text}
              </span>
            </div>
            <div className="glass-card px-3 py-2.5 text-center">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Governance</div>
              <span className={`text-sm font-bold ${govInfo.cls}`}>
                {govInfo.text}
              </span>
            </div>
          </div>

          {/* TIME score bar */}
          {node.timeScore !== undefined && (
            <div className="glass-card px-4 py-3">
              <div className="flex items-center justify-between mb-1.5">
                <div className="text-xs text-gray-400 font-medium">TIME Score</div>
                <div className="text-xs font-bold text-gray-200">{node.timeScore}/100</div>
              </div>
              <div className="h-1.5 rounded-full bg-surface-base overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-500"
                  style={{
                    width: `${node.timeScore}%`,
                    background:
                      node.timeScore >= 70
                        ? '#3FB950'
                        : node.timeScore >= 50
                        ? '#58A6FF'
                        : node.timeScore >= 30
                        ? '#D29922'
                        : '#F85149',
                  }}
                />
              </div>
            </div>
          )}

          {/* Description */}
          {node.description && (
            <div>
              <div className="text-xs text-gray-500 uppercase tracking-wider mb-1.5">Description</div>
              <p className="text-sm text-gray-300 leading-relaxed">{node.description}</p>
            </div>
          )}

          {/* Quick actions */}
          <div>
            <div className="text-xs text-gray-500 uppercase tracking-wider mb-2">Quick Actions</div>
            <div className="flex flex-wrap gap-2">
              {quickActions.map((action) => {
                const ActionIcon = action.icon;
                return (
                  <button
                    key={action.label}
                    className="flex items-center gap-1.5 px-3 py-1.5 glass-card text-xs font-medium text-gray-300 hover:text-gray-100 hover:border-primary-500/30 transition-all"
                  >
                    <ActionIcon className="w-3.5 h-3.5" />
                    {action.label}
                  </button>
                );
              })}
            </div>
          </div>

          {/* AI summary */}
          <div className="glass-card px-4 py-3 border-status-ai/20">
            <div className="flex items-center gap-1.5 mb-2">
              <Sparkles className="w-3.5 h-3.5 text-status-ai" />
              <span className="text-xs font-medium text-status-ai">AI Summary</span>
            </div>
            <p className="text-xs text-gray-400 leading-relaxed">{aiSummary}</p>
          </div>
        </div>
      </div>
    </div>
  );
}

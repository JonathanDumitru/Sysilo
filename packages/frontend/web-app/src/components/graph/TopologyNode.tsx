import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import {
  Workflow,
  Link2,
  Database,
  Bot,
} from 'lucide-react';
import type { TopologyNodeData, NodeStatus } from '../../hooks/useTopologyGraph.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const TYPE_ICONS: Record<string, React.ElementType> = {
  integration: Workflow,
  connection: Link2,
  asset: Database,
  agent: Bot,
};

const STATUS_DOT: Record<NodeStatus, string> = {
  healthy: 'bg-status-healthy shadow-[0_0_6px_rgba(63,185,80,0.6)]',
  warning: 'bg-status-warning shadow-[0_0_6px_rgba(210,153,34,0.6)]',
  critical: 'bg-status-critical shadow-[0_0_6px_rgba(248,81,73,0.6)] animate-pulse',
  inactive: 'bg-gray-500',
};

/** Shape class per node type */
function shapeClass(type: string): string {
  switch (type) {
    case 'agent':
      return 'rounded-full'; // circle
    case 'asset':
      return 'rounded-lg [clip-path:polygon(25%_0%,75%_0%,100%_50%,75%_100%,25%_100%,0%_50%)]'; // hexagon via clip-path
    case 'integration':
    case 'connection':
    default:
      return 'rounded-xl'; // rounded rect
  }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const TopologyNode = memo(function TopologyNode({ data, selected }: NodeProps) {
  const d = data as TopologyNodeData & { _borderColor?: string; _lens?: string };
  const Icon = TYPE_ICONS[d.type] ?? Workflow;
  const borderColor = (d._borderColor as string) ?? 'rgba(255,255,255,0.08)';
  const isHex = d.type === 'asset';

  const glowShadow =
    d.status === 'critical'
      ? '0 0 20px rgba(248,81,73,0.25)'
      : d.status === 'warning'
      ? '0 0 14px rgba(210,153,34,0.18)'
      : 'none';

  return (
    <div
      className={`relative group ${isHex ? '' : shapeClass(d.type)}`}
      style={{ minWidth: isHex ? 120 : undefined }}
    >
      {/* Handles */}
      <Handle type="target" position={Position.Left} className="!bg-gray-500 !border-surface-raised !w-2.5 !h-2.5" />
      <Handle type="source" position={Position.Right} className="!bg-gray-500 !border-surface-raised !w-2.5 !h-2.5" />

      {/* Card body */}
      <div
        className={`px-4 py-3 backdrop-blur-[16px] bg-surface-raised/80 border min-w-[140px] transition-all duration-200 ${
          isHex ? '' : shapeClass(d.type)
        } ${selected ? 'ring-2 ring-primary-400 shadow-glow' : ''}`}
        style={{
          borderColor,
          boxShadow: glowShadow,
          ...(isHex
            ? { clipPath: 'polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%)' }
            : {}),
        }}
      >
        <div className={`flex items-center gap-2.5 ${isHex ? 'justify-center' : ''}`}>
          {/* Icon */}
          <div
            className="p-1.5 rounded-lg bg-surface-overlay/70 border border-surface-border flex-shrink-0"
          >
            <Icon className="w-4 h-4 text-gray-300" />
          </div>

          {/* Name & type */}
          {!isHex && (
            <div className="flex-1 min-w-0">
              <div className="font-medium text-sm text-gray-200 truncate leading-tight">
                {d.name}
              </div>
              <div className="text-[10px] text-gray-500 capitalize">{d.type}</div>
            </div>
          )}

          {/* Status dot */}
          <div className={`w-2 h-2 rounded-full flex-shrink-0 ${STATUS_DOT[d.status]}`} />
        </div>

        {/* Hexagon shows name below icon row */}
        {isHex && (
          <div className="mt-1 text-center">
            <div className="text-xs text-gray-200 font-medium truncate">{d.name}</div>
          </div>
        )}
      </div>

      {/* Hover tooltip (mini AI summary) */}
      <div className="absolute left-1/2 -translate-x-1/2 -top-10 opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-150 z-50 whitespace-nowrap">
        <div className="glass-panel-strong px-3 py-1.5 text-[11px] text-gray-300 max-w-[220px] truncate">
          {d.description ?? d.name} — {d.lastActivity ?? 'No recent activity'}
        </div>
      </div>
    </div>
  );
});

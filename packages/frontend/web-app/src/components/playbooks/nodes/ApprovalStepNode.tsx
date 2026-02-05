import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { UserCheck } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const ApprovalStepNode = memo(function ApprovalStepNode({
  data,
  selected,
}: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;
  const approvers = (nodeData.config?.approvers as string[]) ?? [];
  const timeout = nodeData.config?.timeout_hours as number | undefined;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-yellow-100 rounded border border-yellow-200">
          <UserCheck className="w-4 h-4 text-yellow-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Approval</div>
        </div>
      </div>

      <div className="mt-2 space-y-1">
        {approvers.length > 0 ? (
          <div className="text-xs text-gray-500 truncate">
            {approvers.length} approver{approvers.length !== 1 ? 's' : ''}
          </div>
        ) : (
          <div className="text-xs text-gray-400">No approvers set</div>
        )}
        {timeout && (
          <div className="text-xs text-gray-500">
            Timeout: {timeout}h
          </div>
        )}
      </div>

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});

import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { GitBranch } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const ConditionStepNode = memo(function ConditionStepNode({
  data,
  selected,
}: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;
  const expression = (nodeData.config?.expression as string) ?? 'No condition set';

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-purple-100 rounded border border-purple-200">
          <GitBranch className="w-4 h-4 text-purple-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Condition</div>
        </div>
      </div>

      <div className="mt-2 text-xs text-gray-500 truncate font-mono" title={expression}>
        {expression}
      </div>

      {/* True branch - positioned at top 30% */}
      <Handle
        type="source"
        position={Position.Right}
        id="true"
        style={{ top: '30%' }}
        className="!bg-green-500"
      />

      {/* False branch - positioned at bottom 70% */}
      <Handle
        type="source"
        position={Position.Right}
        id="false"
        style={{ top: '70%' }}
        className="!bg-red-500"
      />

      {/* Labels for the handles */}
      <div className="absolute right-[-8px] top-[30%] translate-y-[-50%] translate-x-full text-xs text-green-600 font-medium pl-2">
        true
      </div>
      <div className="absolute right-[-8px] top-[70%] translate-y-[-50%] translate-x-full text-xs text-red-600 font-medium pl-2">
        false
      </div>
    </div>
  );
});

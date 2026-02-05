import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Workflow } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const IntegrationStepNode = memo(function IntegrationStepNode({
  data,
  selected,
}: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;
  const integrationName = (nodeData.config?.integration_id as string) ?? 'Not configured';

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-indigo-100 rounded border border-indigo-200">
          <Workflow className="w-4 h-4 text-indigo-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Integration</div>
        </div>
      </div>

      <div className="mt-2 text-xs text-gray-500 truncate" title={integrationName}>
        {integrationName}
      </div>

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});

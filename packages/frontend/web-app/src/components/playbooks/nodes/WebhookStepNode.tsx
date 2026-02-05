import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Globe } from 'lucide-react';
import { type StepNodeData, statusStyles } from './index';

export const WebhookStepNode = memo(function WebhookStepNode({
  data,
  selected,
}: NodeProps) {
  const nodeData = data as StepNodeData;
  const statusStyle = statusStyles[nodeData.status ?? 'pending'] ?? statusStyles.pending;
  const url = (nodeData.config?.url as string) ?? 'No URL configured';
  const method = (nodeData.config?.method as string) ?? 'POST';

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[180px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-orange-100 rounded border border-orange-200">
          <Globe className="w-4 h-4 text-orange-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">Webhook</div>
        </div>
      </div>

      <div className="mt-2 flex items-center gap-1.5">
        <span className="text-xs font-medium text-orange-600 bg-orange-50 px-1.5 py-0.5 rounded">
          {method}
        </span>
        <span className="text-xs text-gray-500 truncate flex-1" title={url}>
          {url}
        </span>
      </div>

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});

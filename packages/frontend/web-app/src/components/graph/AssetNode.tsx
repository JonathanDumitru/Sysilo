import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Database, Server, Workflow, Globe } from 'lucide-react';

export interface AssetNodeData extends Record<string, unknown> {
  id: string;
  name: string;
  asset_type: string;
  status: string;
  vendor?: string;
  tags: string[];
}

const typeIcons: Record<string, React.ElementType> = {
  database: Database,
  application: Server,
  service: Server,
  api: Workflow,
  integration: Workflow,
  default: Globe,
};

const statusColors: Record<string, string> = {
  active: 'border-green-400 bg-green-50',
  deprecated: 'border-yellow-400 bg-yellow-50',
  default: 'border-gray-300 bg-white',
};

export const AssetNode = memo(function AssetNode({ data, selected }: NodeProps) {
  const nodeData = data as AssetNodeData;
  const Icon = typeIcons[nodeData.asset_type] ?? typeIcons.default;
  const statusStyle = statusColors[nodeData.status] ?? statusColors.default;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[160px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-white rounded border border-gray-200">
          <Icon className="w-4 h-4 text-gray-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{nodeData.name}</div>
          <div className="text-xs text-gray-500">{nodeData.asset_type}</div>
        </div>
      </div>

      {nodeData.vendor && (
        <div className="mt-2 text-xs text-gray-500 truncate">{nodeData.vendor}</div>
      )}

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
});

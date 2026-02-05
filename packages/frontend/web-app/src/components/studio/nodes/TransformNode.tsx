import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Shuffle, Filter, Calculator, GitMerge } from 'lucide-react';

const transformIcons: Record<string, React.ElementType> = {
  map: Shuffle,
  filter: Filter,
  aggregate: Calculator,
  join: GitMerge,
};

interface TransformNodeData extends Record<string, unknown> {
  label: string;
  transformType: string;
  config: Record<string, unknown>;
}

export const TransformNode = memo(function TransformNode({ data, selected }: NodeProps) {
  const nodeData = data as TransformNodeData;
  const Icon = transformIcons[nodeData.transformType] || Shuffle;

  return (
    <div
      className={`bg-white rounded-lg border-2 shadow-sm min-w-[180px] ${
        selected ? 'border-primary-500' : 'border-amber-300'
      }`}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Left}
        className="!w-3 !h-3 !bg-amber-500 !border-2 !border-white"
      />

      {/* Header */}
      <div className="bg-amber-50 px-3 py-2 rounded-t-md border-b border-amber-100">
        <div className="flex items-center gap-2">
          <div className="p-1 bg-amber-100 rounded">
            <Icon className="w-4 h-4 text-amber-600" />
          </div>
          <span className="text-xs font-semibold text-amber-700 uppercase">Transform</span>
        </div>
      </div>

      {/* Content */}
      <div className="px-3 py-3">
        <p className="text-sm font-medium text-gray-900">{nodeData.label}</p>
        <p className="text-xs text-gray-500 mt-1">{nodeData.transformType}</p>
      </div>

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Right}
        className="!w-3 !h-3 !bg-amber-500 !border-2 !border-white"
      />
    </div>
  );
});

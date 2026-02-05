import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Database, Cloud, FileText } from 'lucide-react';

const connectorIcons: Record<string, React.ElementType> = {
  postgresql: Database,
  mysql: Database,
  mongodb: Database,
  salesforce: Cloud,
  hubspot: Cloud,
  s3: FileText,
};

interface SourceNodeData extends Record<string, unknown> {
  label: string;
  connector: string;
  config: Record<string, unknown>;
}

export const SourceNode = memo(function SourceNode({ data, selected }: NodeProps) {
  const nodeData = data as SourceNodeData;
  const Icon = connectorIcons[nodeData.connector] || Database;

  return (
    <div
      className={`bg-white rounded-lg border-2 shadow-sm min-w-[180px] ${
        selected ? 'border-primary-500' : 'border-emerald-300'
      }`}
    >
      {/* Header */}
      <div className="bg-emerald-50 px-3 py-2 rounded-t-md border-b border-emerald-100">
        <div className="flex items-center gap-2">
          <div className="p-1 bg-emerald-100 rounded">
            <Icon className="w-4 h-4 text-emerald-600" />
          </div>
          <span className="text-xs font-semibold text-emerald-700 uppercase">Source</span>
        </div>
      </div>

      {/* Content */}
      <div className="px-3 py-3">
        <p className="text-sm font-medium text-gray-900">{nodeData.label}</p>
        <p className="text-xs text-gray-500 mt-1">{nodeData.connector}</p>
      </div>

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Right}
        className="!w-3 !h-3 !bg-emerald-500 !border-2 !border-white"
      />
    </div>
  );
});

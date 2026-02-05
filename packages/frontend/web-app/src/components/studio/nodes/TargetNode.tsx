import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';
import { Database, Cloud, FileText, Send } from 'lucide-react';

const connectorIcons: Record<string, React.ElementType> = {
  snowflake: Database,
  bigquery: Database,
  postgresql: Database,
  salesforce: Cloud,
  slack: Send,
  s3: FileText,
};

interface TargetNodeData {
  label: string;
  connector: string;
  config: Record<string, unknown>;
}

export const TargetNode = memo(({ data, selected }: NodeProps<TargetNodeData>) => {
  const Icon = connectorIcons[data.connector] || Database;

  return (
    <div
      className={`bg-white rounded-lg border-2 shadow-sm min-w-[180px] ${
        selected ? 'border-primary-500' : 'border-blue-300'
      }`}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Left}
        className="!w-3 !h-3 !bg-blue-500 !border-2 !border-white"
      />

      {/* Header */}
      <div className="bg-blue-50 px-3 py-2 rounded-t-md border-b border-blue-100">
        <div className="flex items-center gap-2">
          <div className="p-1 bg-blue-100 rounded">
            <Icon className="w-4 h-4 text-blue-600" />
          </div>
          <span className="text-xs font-semibold text-blue-700 uppercase">Target</span>
        </div>
      </div>

      {/* Content */}
      <div className="px-3 py-3">
        <p className="text-sm font-medium text-gray-900">{data.label}</p>
        <p className="text-xs text-gray-500 mt-1">{data.connector}</p>
      </div>
    </div>
  );
});

TargetNode.displayName = 'TargetNode';

import { Database, Shuffle, Target, Filter, Calculator, GitMerge, Cloud, FileText } from 'lucide-react';

interface NodeTypeItem {
  type: string;
  label: string;
  icon: React.ElementType;
  category: 'source' | 'transform' | 'target';
  data: Record<string, unknown>;
}

const nodeTypes: NodeTypeItem[] = [
  // Sources
  {
    type: 'source',
    label: 'PostgreSQL',
    icon: Database,
    category: 'source',
    data: { label: 'PostgreSQL Source', connector: 'postgresql', config: {} },
  },
  {
    type: 'source',
    label: 'MySQL',
    icon: Database,
    category: 'source',
    data: { label: 'MySQL Source', connector: 'mysql', config: {} },
  },
  {
    type: 'source',
    label: 'Salesforce',
    icon: Cloud,
    category: 'source',
    data: { label: 'Salesforce Source', connector: 'salesforce', config: {} },
  },
  {
    type: 'source',
    label: 'S3',
    icon: FileText,
    category: 'source',
    data: { label: 'S3 Source', connector: 's3', config: {} },
  },

  // Transforms
  {
    type: 'transform',
    label: 'Map Fields',
    icon: Shuffle,
    category: 'transform',
    data: { label: 'Field Mapping', transformType: 'map', config: {} },
  },
  {
    type: 'transform',
    label: 'Filter',
    icon: Filter,
    category: 'transform',
    data: { label: 'Filter Records', transformType: 'filter', config: {} },
  },
  {
    type: 'transform',
    label: 'Aggregate',
    icon: Calculator,
    category: 'transform',
    data: { label: 'Aggregate', transformType: 'aggregate', config: {} },
  },
  {
    type: 'transform',
    label: 'Join',
    icon: GitMerge,
    category: 'transform',
    data: { label: 'Join', transformType: 'join', config: {} },
  },

  // Targets
  {
    type: 'target',
    label: 'Snowflake',
    icon: Database,
    category: 'target',
    data: { label: 'Snowflake Target', connector: 'snowflake', config: {} },
  },
  {
    type: 'target',
    label: 'BigQuery',
    icon: Database,
    category: 'target',
    data: { label: 'BigQuery Target', connector: 'bigquery', config: {} },
  },
  {
    type: 'target',
    label: 'PostgreSQL',
    icon: Database,
    category: 'target',
    data: { label: 'PostgreSQL Target', connector: 'postgresql', config: {} },
  },
  {
    type: 'target',
    label: 'S3',
    icon: FileText,
    category: 'target',
    data: { label: 'S3 Target', connector: 's3', config: {} },
  },
];

const categoryColors = {
  source: 'bg-emerald-50 border-emerald-200 text-emerald-700 hover:bg-emerald-100',
  transform: 'bg-amber-50 border-amber-200 text-amber-700 hover:bg-amber-100',
  target: 'bg-blue-50 border-blue-200 text-blue-700 hover:bg-blue-100',
};

const categoryLabels = {
  source: 'Sources',
  transform: 'Transforms',
  target: 'Targets',
};

export function NodeToolbox() {
  const onDragStart = (event: React.DragEvent, item: NodeTypeItem) => {
    event.dataTransfer.setData('application/reactflow', item.type);
    event.dataTransfer.setData('application/nodedata', JSON.stringify(item.data));
    event.dataTransfer.effectAllowed = 'move';
  };

  const groupedNodes = nodeTypes.reduce((acc, item) => {
    if (!acc[item.category]) acc[item.category] = [];
    acc[item.category].push(item);
    return acc;
  }, {} as Record<string, NodeTypeItem[]>);

  return (
    <div className="w-64 bg-white border-r border-gray-200 overflow-y-auto">
      <div className="p-4 border-b border-gray-100">
        <h3 className="text-sm font-semibold text-gray-900">Components</h3>
        <p className="text-xs text-gray-500 mt-1">Drag nodes to the canvas</p>
      </div>

      <div className="p-3 space-y-4">
        {(['source', 'transform', 'target'] as const).map((category) => (
          <div key={category}>
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
              {categoryLabels[category]}
            </h4>
            <div className="space-y-1">
              {groupedNodes[category]?.map((item) => (
                <div
                  key={`${item.type}-${item.label}`}
                  draggable
                  onDragStart={(e) => onDragStart(e, item)}
                  className={`flex items-center gap-2 px-3 py-2 rounded-lg border cursor-grab active:cursor-grabbing transition-colors ${categoryColors[category]}`}
                >
                  <item.icon className="w-4 h-4" />
                  <span className="text-sm font-medium">{item.label}</span>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

import { Workflow, Globe, Clock, GitBranch, UserCheck } from 'lucide-react';

const stepTypes = [
  {
    type: 'integration',
    label: 'Integration',
    description: 'Run an existing integration',
    icon: Workflow,
    color: 'indigo',
    defaultData: {
      name: 'Run Integration',
      config: { integration_id: '' },
    },
  },
  {
    type: 'webhook',
    label: 'Webhook',
    description: 'HTTP request to external URL',
    icon: Globe,
    color: 'orange',
    defaultData: {
      name: 'HTTP Request',
      config: { url: '', method: 'POST', headers: {}, body: '' },
    },
  },
  {
    type: 'wait',
    label: 'Wait',
    description: 'Pause for a duration',
    icon: Clock,
    color: 'gray',
    defaultData: {
      name: 'Wait',
      config: { duration_seconds: 60 },
    },
  },
  {
    type: 'condition',
    label: 'Condition',
    description: 'Branch based on expression',
    icon: GitBranch,
    color: 'purple',
    defaultData: {
      name: 'Check Condition',
      config: { expression: '' },
    },
  },
  {
    type: 'approval',
    label: 'Approval',
    description: 'Wait for manual approval',
    icon: UserCheck,
    color: 'yellow',
    defaultData: {
      name: 'Require Approval',
      config: { message: 'Please approve to continue' },
    },
  },
];

const colorMap: Record<string, string> = {
  indigo: 'bg-indigo-50 border-indigo-200 hover:border-indigo-300',
  orange: 'bg-orange-50 border-orange-200 hover:border-orange-300',
  gray: 'bg-gray-50 border-gray-200 hover:border-gray-300',
  purple: 'bg-purple-50 border-purple-200 hover:border-purple-300',
  yellow: 'bg-yellow-50 border-yellow-200 hover:border-yellow-300',
};

const iconColorMap: Record<string, string> = {
  indigo: 'text-indigo-600',
  orange: 'text-orange-600',
  gray: 'text-gray-600',
  purple: 'text-purple-600',
  yellow: 'text-yellow-600',
};

export function PlaybookToolbox() {
  const onDragStart = (
    event: React.DragEvent,
    stepType: string,
    defaultData: Record<string, unknown>
  ) => {
    event.dataTransfer.setData('application/reactflow', stepType);
    event.dataTransfer.setData('application/nodedata', JSON.stringify(defaultData));
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className="w-64 bg-white border-r border-gray-200 p-4 overflow-y-auto">
      <h3 className="text-sm font-semibold text-gray-900 mb-3">Step Types</h3>
      <p className="text-xs text-gray-500 mb-4">Drag steps onto the canvas to build your playbook</p>

      <div className="space-y-2">
        {stepTypes.map((step) => {
          const Icon = step.icon;
          return (
            <div
              key={step.type}
              draggable
              onDragStart={(e) => onDragStart(e, step.type, step.defaultData)}
              className={`p-3 rounded-lg border-2 cursor-grab active:cursor-grabbing transition-colors ${colorMap[step.color]}`}
            >
              <div className="flex items-center gap-2">
                <Icon className={`w-4 h-4 ${iconColorMap[step.color]}`} />
                <span className="text-sm font-medium text-gray-900">{step.label}</span>
              </div>
              <p className="text-xs text-gray-500 mt-1">{step.description}</p>
            </div>
          );
        })}
      </div>
    </div>
  );
}

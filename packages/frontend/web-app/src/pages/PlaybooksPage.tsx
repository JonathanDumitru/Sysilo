import { useState } from 'react';
import {
  Plus,
  Search,
  BookOpen,
  ChevronRight,
  X,
  Edit,
  Copy,
  Play,
} from 'lucide-react';

// Mock data
const playbooks = [
  {
    id: '1',
    name: 'Cloud Rehosting Playbook',
    strategy: 'rehost',
    description: 'Lift-and-shift migration to cloud infrastructure with minimal changes',
    isTemplate: true,
    version: 2,
    phases: [
      {
        id: '1',
        name: 'Assessment',
        description: 'Evaluate application readiness for cloud migration',
        tasks: [
          { id: '1', name: 'Infrastructure dependency mapping', estimatedHours: 8 },
          { id: '2', name: 'Network requirements analysis', estimatedHours: 4 },
          { id: '3', name: 'Security compliance check', estimatedHours: 6 },
        ],
      },
      {
        id: '2',
        name: 'Planning',
        description: 'Develop detailed migration plan',
        tasks: [
          { id: '4', name: 'Create migration timeline', estimatedHours: 4 },
          { id: '5', name: 'Define rollback procedures', estimatedHours: 4 },
          { id: '6', name: 'Resource provisioning plan', estimatedHours: 6 },
        ],
      },
      {
        id: '3',
        name: 'Execution',
        description: 'Execute the migration',
        tasks: [
          { id: '7', name: 'Provision cloud resources', estimatedHours: 8 },
          { id: '8', name: 'Data migration', estimatedHours: 16 },
          { id: '9', name: 'Application deployment', estimatedHours: 8 },
        ],
      },
      {
        id: '4',
        name: 'Validation',
        description: 'Verify migration success',
        tasks: [
          { id: '10', name: 'Functional testing', estimatedHours: 12 },
          { id: '11', name: 'Performance testing', estimatedHours: 8 },
          { id: '12', name: 'User acceptance testing', estimatedHours: 16 },
        ],
      },
    ],
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-10T00:00:00Z',
  },
  {
    id: '2',
    name: 'Application Retirement Playbook',
    strategy: 'retire',
    description: 'Safely decommission applications with data preservation and user transition',
    isTemplate: true,
    version: 3,
    phases: [
      {
        id: '1',
        name: 'Discovery',
        description: 'Identify dependencies and stakeholders',
        tasks: [
          { id: '1', name: 'Stakeholder identification', estimatedHours: 4 },
          { id: '2', name: 'Dependency analysis', estimatedHours: 8 },
          { id: '3', name: 'Data inventory', estimatedHours: 6 },
        ],
      },
      {
        id: '2',
        name: 'Planning',
        description: 'Create retirement and transition plan',
        tasks: [
          { id: '4', name: 'User communication plan', estimatedHours: 4 },
          { id: '5', name: 'Data archival strategy', estimatedHours: 6 },
          { id: '6', name: 'Alternative solution mapping', estimatedHours: 8 },
        ],
      },
      {
        id: '3',
        name: 'Transition',
        description: 'Move users and data to alternatives',
        tasks: [
          { id: '7', name: 'User training on alternatives', estimatedHours: 16 },
          { id: '8', name: 'Data migration to archive', estimatedHours: 12 },
          { id: '9', name: 'Integration updates', estimatedHours: 8 },
        ],
      },
      {
        id: '4',
        name: 'Decommission',
        description: 'Safely shut down the application',
        tasks: [
          { id: '10', name: 'Disable user access', estimatedHours: 2 },
          { id: '11', name: 'System shutdown', estimatedHours: 4 },
          { id: '12', name: 'Documentation and closure', estimatedHours: 4 },
        ],
      },
    ],
    createdAt: '2024-01-05T00:00:00Z',
    updatedAt: '2024-01-12T00:00:00Z',
  },
  {
    id: '3',
    name: 'Platform Modernization Playbook',
    strategy: 'refactor',
    description: 'Re-architect applications for cloud-native capabilities',
    isTemplate: true,
    version: 1,
    phases: [
      {
        id: '1',
        name: 'Architecture Analysis',
        description: 'Analyze current architecture and define target state',
        tasks: [
          { id: '1', name: 'Current state documentation', estimatedHours: 16 },
          { id: '2', name: 'Target architecture design', estimatedHours: 24 },
          { id: '3', name: 'Gap analysis', estimatedHours: 8 },
        ],
      },
      {
        id: '2',
        name: 'Refactoring',
        description: 'Implement architectural changes',
        tasks: [
          { id: '4', name: 'Microservices decomposition', estimatedHours: 80 },
          { id: '5', name: 'API modernization', estimatedHours: 40 },
          { id: '6', name: 'Database optimization', estimatedHours: 32 },
        ],
      },
    ],
    createdAt: '2024-01-08T00:00:00Z',
    updatedAt: '2024-01-08T00:00:00Z',
  },
];

const strategyColors: Record<string, string> = {
  rehost: 'bg-blue-100 text-blue-700 border-blue-200',
  replatform: 'bg-purple-100 text-purple-700 border-purple-200',
  refactor: 'bg-indigo-100 text-indigo-700 border-indigo-200',
  replace: 'bg-orange-100 text-orange-700 border-orange-200',
  retire: 'bg-red-100 text-red-700 border-red-200',
  retain: 'bg-gray-100 text-gray-700 border-gray-200',
};

const strategyDescriptions: Record<string, string> = {
  rehost: 'Lift and shift - minimal changes',
  replatform: 'Lift and optimize - some cloud optimizations',
  refactor: 'Re-architect - significant changes for cloud-native',
  replace: 'Replace with SaaS or new solution',
  retire: 'Decommission the application',
  retain: 'Keep as-is with no changes',
};

export function PlaybooksPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [strategyFilter, setStrategyFilter] = useState<string>('all');
  const [selectedPlaybook, setSelectedPlaybook] = useState<typeof playbooks[0] | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString();
  };

  const getTotalHours = (playbook: typeof playbooks[0]) => {
    return playbook.phases.reduce(
      (sum, phase) => sum + phase.tasks.reduce((taskSum, task) => taskSum + task.estimatedHours, 0),
      0
    );
  };

  const getTotalTasks = (playbook: typeof playbooks[0]) => {
    return playbook.phases.reduce((sum, phase) => sum + phase.tasks.length, 0);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Migration Playbooks</h1>
          <p className="text-gray-500">Standardized templates for application rationalization</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Playbook
        </button>
      </div>

      {/* Strategy Overview */}
      <div className="grid grid-cols-6 gap-3">
        {Object.entries(strategyColors).map(([strategy, color]) => (
          <button
            key={strategy}
            onClick={() => setStrategyFilter(strategyFilter === strategy ? 'all' : strategy)}
            className={`p-3 rounded-xl border-2 text-left transition-all ${
              strategyFilter === strategy
                ? color + ' ring-2 ring-offset-1'
                : 'bg-white border-gray-100 hover:border-gray-200'
            }`}
          >
            <p className="text-sm font-semibold capitalize mb-1">{strategy}</p>
            <p className="text-xs text-gray-500 line-clamp-2">{strategyDescriptions[strategy]}</p>
          </button>
        ))}
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search playbooks..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <select className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
          <option value="all">All Types</option>
          <option value="template">Templates</option>
          <option value="custom">Custom</option>
        </select>
      </div>

      {/* Playbooks Grid */}
      <div className="grid grid-cols-2 gap-6">
        {/* Playbooks List */}
        <div className="space-y-4">
          {playbooks
            .filter((p) => strategyFilter === 'all' || p.strategy === strategyFilter)
            .map((playbook) => (
              <div
                key={playbook.id}
                onClick={() => setSelectedPlaybook(playbook)}
                className={`bg-white rounded-xl p-5 shadow-sm border cursor-pointer transition-all ${
                  selectedPlaybook?.id === playbook.id
                    ? 'border-primary-500 ring-2 ring-primary-100'
                    : 'border-gray-100 hover:border-gray-200'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-3">
                    <div className="p-2 bg-primary-50 rounded-lg">
                      <BookOpen className="w-5 h-5 text-primary-600" />
                    </div>
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="font-semibold text-gray-900">{playbook.name}</h3>
                        {playbook.isTemplate && (
                          <span className="text-xs px-1.5 py-0.5 bg-purple-100 text-purple-700 rounded">
                            Template
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-500 line-clamp-2">{playbook.description}</p>
                    </div>
                  </div>
                  <ChevronRight className="w-5 h-5 text-gray-400 flex-shrink-0" />
                </div>

                <div className="flex items-center gap-4 mt-4 pt-3 border-t border-gray-100">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full border capitalize ${strategyColors[playbook.strategy]}`}
                  >
                    {playbook.strategy}
                  </span>
                  <span className="text-xs text-gray-500">{playbook.phases.length} phases</span>
                  <span className="text-xs text-gray-500">{getTotalTasks(playbook)} tasks</span>
                  <span className="text-xs text-gray-500">{getTotalHours(playbook)}h est.</span>
                  <span className="text-xs text-gray-400 ml-auto">v{playbook.version}</span>
                </div>
              </div>
            ))}
        </div>

        {/* Detail Panel */}
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 h-fit sticky top-6 max-h-[calc(100vh-8rem)] overflow-y-auto">
          {selectedPlaybook ? (
            <div>
              <div className="p-6 border-b border-gray-100 sticky top-0 bg-white z-10">
                <div className="flex items-center justify-between mb-3">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full border capitalize ${strategyColors[selectedPlaybook.strategy]}`}
                  >
                    {selectedPlaybook.strategy}
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-gray-500">v{selectedPlaybook.version}</span>
                    <button className="p-1.5 text-gray-400 hover:text-gray-600 rounded">
                      <Edit className="w-4 h-4" />
                    </button>
                    <button className="p-1.5 text-gray-400 hover:text-gray-600 rounded">
                      <Copy className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <h2 className="text-xl font-semibold text-gray-900 mb-2">
                  {selectedPlaybook.name}
                </h2>
                <p className="text-sm text-gray-600">{selectedPlaybook.description}</p>

                {/* Summary Stats */}
                <div className="grid grid-cols-3 gap-3 mt-4">
                  <div className="p-2 bg-gray-50 rounded-lg text-center">
                    <p className="text-lg font-semibold text-gray-900">
                      {selectedPlaybook.phases.length}
                    </p>
                    <p className="text-xs text-gray-500">Phases</p>
                  </div>
                  <div className="p-2 bg-gray-50 rounded-lg text-center">
                    <p className="text-lg font-semibold text-gray-900">
                      {getTotalTasks(selectedPlaybook)}
                    </p>
                    <p className="text-xs text-gray-500">Tasks</p>
                  </div>
                  <div className="p-2 bg-gray-50 rounded-lg text-center">
                    <p className="text-lg font-semibold text-gray-900">
                      {getTotalHours(selectedPlaybook)}h
                    </p>
                    <p className="text-xs text-gray-500">Est. Hours</p>
                  </div>
                </div>
              </div>

              {/* Phases */}
              <div className="p-6">
                <h3 className="text-sm font-medium text-gray-900 mb-4">Phases & Tasks</h3>
                <div className="space-y-4">
                  {selectedPlaybook.phases.map((phase, phaseIndex) => (
                    <div key={phase.id} className="border border-gray-100 rounded-lg overflow-hidden">
                      <div className="p-3 bg-gray-50 border-b border-gray-100">
                        <div className="flex items-center gap-2">
                          <span className="flex items-center justify-center w-6 h-6 bg-primary-100 text-primary-700 text-xs font-medium rounded-full">
                            {phaseIndex + 1}
                          </span>
                          <div>
                            <p className="text-sm font-medium text-gray-900">{phase.name}</p>
                            <p className="text-xs text-gray-500">{phase.description}</p>
                          </div>
                        </div>
                      </div>
                      <div className="divide-y divide-gray-50">
                        {phase.tasks.map((task) => (
                          <div
                            key={task.id}
                            className="px-3 py-2 flex items-center justify-between"
                          >
                            <div className="flex items-center gap-2">
                              <div className="w-4 h-4 rounded border border-gray-300" />
                              <span className="text-sm text-gray-700">{task.name}</span>
                            </div>
                            <span className="text-xs text-gray-500">{task.estimatedHours}h</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Actions */}
              <div className="p-6 border-t border-gray-100 sticky bottom-0 bg-white">
                <div className="flex items-center justify-between text-xs text-gray-500 mb-4">
                  <span>Created {formatDate(selectedPlaybook.createdAt)}</span>
                  <span>Updated {formatDate(selectedPlaybook.updatedAt)}</span>
                </div>
                <div className="flex gap-2">
                  <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                    <Play className="w-4 h-4" />
                    Start Project
                  </button>
                  <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50">
                    <Copy className="w-4 h-4" />
                    Duplicate
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-8 text-center text-gray-500">
              <BookOpen className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>Select a playbook to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Create Playbook Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Playbook</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  placeholder="e.g., Custom Migration Playbook"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Strategy</label>
                <select className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
                  {Object.keys(strategyColors).map((strategy) => (
                    <option key={strategy} value={strategy} className="capitalize">
                      {strategy.charAt(0).toUpperCase() + strategy.slice(1)} - {strategyDescriptions[strategy]}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  rows={3}
                  placeholder="Describe the playbook and when it should be used"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Start From
                </label>
                <div className="space-y-2">
                  <label className="flex items-center gap-3 p-3 border border-gray-200 rounded-lg cursor-pointer hover:bg-gray-50">
                    <input type="radio" name="startFrom" value="blank" className="text-primary-600" />
                    <div>
                      <p className="text-sm font-medium text-gray-900">Blank Playbook</p>
                      <p className="text-xs text-gray-500">Start from scratch</p>
                    </div>
                  </label>
                  <label className="flex items-center gap-3 p-3 border border-gray-200 rounded-lg cursor-pointer hover:bg-gray-50">
                    <input type="radio" name="startFrom" value="template" defaultChecked className="text-primary-600" />
                    <div>
                      <p className="text-sm font-medium text-gray-900">Copy from Template</p>
                      <p className="text-xs text-gray-500">Use an existing playbook as a starting point</p>
                    </div>
                  </label>
                </div>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Playbook
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

import { useState } from 'react';
import {
  Plus,
  Search,
  MoreHorizontal,
  Calendar,
  Users,
  CheckCircle,
  Clock,
  AlertCircle,
  Play,
  Pause,
  ChevronRight,
  X,
  ArrowRight,
} from 'lucide-react';

// Mock data
const projects = [
  {
    id: '1',
    name: 'Cloud Migration Wave 1',
    description: 'First wave of cloud migration for critical business applications',
    application: { id: '1', name: 'Core ERP System' },
    playbook: { id: '1', name: 'Cloud Rehosting Playbook' },
    status: 'in_progress',
    progress: 65,
    startDate: '2024-01-15',
    targetDate: '2024-03-15',
    owner: 'John Doe',
    team: ['John Doe', 'Jane Smith', 'Mike Johnson'],
    currentPhase: 'Execution',
    phases: [
      { name: 'Assessment', status: 'completed', progress: 100 },
      { name: 'Planning', status: 'completed', progress: 100 },
      { name: 'Execution', status: 'in_progress', progress: 60 },
      { name: 'Validation', status: 'pending', progress: 0 },
    ],
    recentActivity: [
      { id: '1', action: 'Completed data migration', user: 'Jane Smith', time: '2 hours ago' },
      { id: '2', action: 'Started application deployment', user: 'Mike Johnson', time: '4 hours ago' },
      { id: '3', action: 'Approved rollback plan', user: 'John Doe', time: '1 day ago' },
    ],
  },
  {
    id: '2',
    name: 'Legacy CRM Retirement',
    description: 'Decommission legacy CRM and transition users to new platform',
    application: { id: '2', name: 'Legacy CRM' },
    playbook: { id: '2', name: 'Application Retirement Playbook' },
    status: 'in_progress',
    progress: 40,
    startDate: '2024-01-20',
    targetDate: '2024-02-28',
    owner: 'Sarah Williams',
    team: ['Sarah Williams', 'Tom Brown'],
    currentPhase: 'Planning',
    phases: [
      { name: 'Discovery', status: 'completed', progress: 100 },
      { name: 'Planning', status: 'in_progress', progress: 60 },
      { name: 'Transition', status: 'pending', progress: 0 },
      { name: 'Decommission', status: 'pending', progress: 0 },
    ],
    recentActivity: [
      { id: '1', action: 'Created user communication plan', user: 'Sarah Williams', time: '3 hours ago' },
      { id: '2', action: 'Completed data inventory', user: 'Tom Brown', time: '1 day ago' },
    ],
  },
  {
    id: '3',
    name: 'HR System Modernization',
    description: 'Refactor HR system for cloud-native deployment',
    application: { id: '3', name: 'HR Management System' },
    playbook: { id: '3', name: 'Platform Modernization Playbook' },
    status: 'planned',
    progress: 0,
    startDate: '2024-02-01',
    targetDate: '2024-05-01',
    owner: 'Emily Chen',
    team: ['Emily Chen'],
    currentPhase: 'Not Started',
    phases: [
      { name: 'Architecture Analysis', status: 'pending', progress: 0 },
      { name: 'Refactoring', status: 'pending', progress: 0 },
    ],
    recentActivity: [],
  },
  {
    id: '4',
    name: 'Email Server Migration',
    description: 'Migrate on-premise email to cloud provider',
    application: { id: '6', name: 'Email Server' },
    playbook: { id: '1', name: 'Cloud Rehosting Playbook' },
    status: 'completed',
    progress: 100,
    startDate: '2023-12-01',
    targetDate: '2024-01-10',
    owner: 'David Lee',
    team: ['David Lee', 'Lisa Park'],
    currentPhase: 'Completed',
    phases: [
      { name: 'Assessment', status: 'completed', progress: 100 },
      { name: 'Planning', status: 'completed', progress: 100 },
      { name: 'Execution', status: 'completed', progress: 100 },
      { name: 'Validation', status: 'completed', progress: 100 },
    ],
    recentActivity: [
      { id: '1', action: 'Project marked as complete', user: 'David Lee', time: '5 days ago' },
    ],
  },
];

const statusConfig: Record<string, { color: string; icon: typeof CheckCircle; label: string }> = {
  planned: { color: 'bg-gray-700/50 text-gray-300', icon: Clock, label: 'Planned' },
  in_progress: { color: 'bg-blue-900/40 text-blue-400', icon: Play, label: 'In Progress' },
  on_hold: { color: 'bg-yellow-900/40 text-yellow-400', icon: Pause, label: 'On Hold' },
  completed: { color: 'bg-green-900/40 text-green-400', icon: CheckCircle, label: 'Completed' },
  cancelled: { color: 'bg-red-900/40 text-red-400', icon: AlertCircle, label: 'Cancelled' },
};

export function ProjectsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [selectedProject, setSelectedProject] = useState<typeof projects[0] | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const getProgressColor = (progress: number, status: string) => {
    if (status === 'completed') return 'bg-green-900/300';
    if (progress >= 75) return 'bg-green-900/300';
    if (progress >= 50) return 'bg-blue-900/300';
    if (progress >= 25) return 'bg-yellow-900/300';
    return 'bg-gray-300';
  };

  const filteredProjects = projects.filter((project) => {
    const matchesSearch =
      project.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      project.application.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = statusFilter === 'all' || project.status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  // Summary counts
  const statusCounts = projects.reduce((acc, project) => {
    acc[project.status] = (acc[project.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Migration Projects</h1>
          <p className="text-gray-500">Track and manage active rationalization initiatives</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          New Project
        </button>
      </div>

      {/* Status Summary */}
      <div className="grid grid-cols-5 gap-4">
        {Object.entries(statusConfig).map(([status, config]) => {
          const StatusIcon = config.icon;
          const count = statusCounts[status] || 0;
          return (
            <button
              key={status}
              onClick={() => setStatusFilter(statusFilter === status ? 'all' : status)}
              className={`p-4 rounded-xl border-2 text-left transition-all ${
                statusFilter === status
                  ? config.color + ' border-current'
                  : 'bg-surface-raised/80 border-surface-border hover:border-surface-border'
              }`}
            >
              <div className="flex items-center justify-between mb-2">
                <StatusIcon className="w-5 h-5" />
                <span className="text-2xl font-bold">{count}</span>
              </div>
              <p className="text-sm font-medium">{config.label}</p>
            </button>
          );
        })}
      </div>

      {/* Search & Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search projects..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 glass-input text-sm"
          />
        </div>
        <select className="px-3 py-2 glass-input text-sm">
          <option value="all">All Owners</option>
          <option value="john">John Doe</option>
          <option value="sarah">Sarah Williams</option>
        </select>
      </div>

      {/* Projects Grid */}
      <div className="grid grid-cols-2 gap-6">
        {/* Projects List */}
        <div className="space-y-4">
          {filteredProjects.map((project) => {
            const StatusIcon = statusConfig[project.status].icon;
            return (
              <div
                key={project.id}
                onClick={() => setSelectedProject(project)}
                className={`glass-panel p-5 cursor-pointer transition-all ${
                  selectedProject?.id === project.id
                    ? 'border-primary-500 ring-2 ring-primary-100'
                    : 'border-surface-border hover:border-surface-border'
                }`}
              >
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-white">{project.name}</h3>
                      <span
                        className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${statusConfig[project.status].color}`}
                      >
                        <StatusIcon className="w-3 h-3" />
                        {statusConfig[project.status].label}
                      </span>
                    </div>
                    <p className="text-sm text-gray-500">{project.application.name}</p>
                  </div>
                  <ChevronRight className="w-5 h-5 text-gray-400 flex-shrink-0" />
                </div>

                {/* Progress Bar */}
                <div className="mb-3">
                  <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                    <span>{project.currentPhase}</span>
                    <span>{project.progress}%</span>
                  </div>
                  <div className="w-full h-2 bg-surface-overlay rounded-full overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all ${getProgressColor(
                        project.progress,
                        project.status
                      )}`}
                      style={{ width: `${project.progress}%` }}
                    />
                  </div>
                </div>

                {/* Footer */}
                <div className="flex items-center justify-between text-xs text-gray-500">
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-1">
                      <Calendar className="w-3.5 h-3.5" />
                      <span>{formatDate(project.targetDate)}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Users className="w-3.5 h-3.5" />
                      <span>{project.team.length}</span>
                    </div>
                  </div>
                  <span>{project.owner}</span>
                </div>
              </div>
            );
          })}
        </div>

        {/* Detail Panel */}
        <div className="glass-panel h-fit sticky top-6 max-h-[calc(100vh-8rem)] overflow-y-auto">
          {selectedProject ? (
            <div>
              <div className="p-6 border-b border-surface-border sticky top-0 bg-surface-raised/80 z-10">
                <div className="flex items-center justify-between mb-3">
                  <span
                    className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${statusConfig[selectedProject.status].color}`}
                  >
                    {statusConfig[selectedProject.status].label}
                  </span>
                  <button className="p-1 text-gray-400 hover:text-gray-300">
                    <MoreHorizontal className="w-5 h-5" />
                  </button>
                </div>
                <h2 className="text-xl font-semibold text-white mb-1">
                  {selectedProject.name}
                </h2>
                <p className="text-sm text-gray-400 mb-3">{selectedProject.description}</p>

                {/* Key Info */}
                <div className="grid grid-cols-2 gap-3">
                  <div className="p-3 bg-surface-overlay/50 rounded-lg">
                    <p className="text-xs text-gray-500 mb-1">Application</p>
                    <p className="text-sm font-medium text-white">
                      {selectedProject.application.name}
                    </p>
                  </div>
                  <div className="p-3 bg-surface-overlay/50 rounded-lg">
                    <p className="text-xs text-gray-500 mb-1">Playbook</p>
                    <p className="text-sm font-medium text-white">
                      {selectedProject.playbook.name}
                    </p>
                  </div>
                </div>
              </div>

              {/* Progress Overview */}
              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">Progress</h3>
                <div className="flex items-center gap-3 mb-4">
                  <div className="flex-1">
                    <div className="w-full h-3 bg-surface-overlay rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all ${getProgressColor(
                          selectedProject.progress,
                          selectedProject.status
                        )}`}
                        style={{ width: `${selectedProject.progress}%` }}
                      />
                    </div>
                  </div>
                  <span className="text-lg font-bold text-white">{selectedProject.progress}%</span>
                </div>

                {/* Phase Progress */}
                <div className="space-y-2">
                  {selectedProject.phases.map((phase, index) => (
                    <div key={phase.name} className="flex items-center gap-3">
                      <div
                        className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium ${
                          phase.status === 'completed'
                            ? 'bg-green-900/40 text-green-400'
                            : phase.status === 'in_progress'
                            ? 'bg-blue-900/40 text-blue-400'
                            : 'bg-surface-overlay text-gray-500'
                        }`}
                      >
                        {phase.status === 'completed' ? (
                          <CheckCircle className="w-4 h-4" />
                        ) : (
                          index + 1
                        )}
                      </div>
                      <div className="flex-1">
                        <div className="flex items-center justify-between">
                          <span className="text-sm text-white">{phase.name}</span>
                          <span className="text-xs text-gray-500">{phase.progress}%</span>
                        </div>
                        <div className="w-full h-1.5 bg-surface-overlay rounded-full overflow-hidden mt-1">
                          <div
                            className={`h-full rounded-full ${
                              phase.status === 'completed'
                                ? 'bg-green-900/300'
                                : phase.status === 'in_progress'
                                ? 'bg-blue-900/300'
                                : 'bg-gray-700'
                            }`}
                            style={{ width: `${phase.progress}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Timeline */}
              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">Timeline</h3>
                <div className="flex items-center gap-3">
                  <div className="flex-1 p-3 bg-surface-overlay/50 rounded-lg">
                    <p className="text-xs text-gray-500 mb-1">Start Date</p>
                    <p className="text-sm font-medium text-white">
                      {formatDate(selectedProject.startDate)}
                    </p>
                  </div>
                  <ArrowRight className="w-5 h-5 text-gray-300" />
                  <div className="flex-1 p-3 bg-surface-overlay/50 rounded-lg">
                    <p className="text-xs text-gray-500 mb-1">Target Date</p>
                    <p className="text-sm font-medium text-white">
                      {formatDate(selectedProject.targetDate)}
                    </p>
                  </div>
                </div>
              </div>

              {/* Team */}
              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">Team</h3>
                <div className="flex items-center gap-2 flex-wrap">
                  {selectedProject.team.map((member, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 px-3 py-1.5 bg-surface-overlay/50 rounded-full"
                    >
                      <div className="w-6 h-6 rounded-full bg-primary-100 flex items-center justify-center">
                        <span className="text-xs font-medium text-primary-700">
                          {member.split(' ').map((n) => n[0]).join('')}
                        </span>
                      </div>
                      <span className="text-sm text-gray-300">{member}</span>
                      {member === selectedProject.owner && (
                        <span className="text-xs bg-primary-100 text-primary-700 px-1.5 py-0.5 rounded">
                          Owner
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              </div>

              {/* Recent Activity */}
              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">Recent Activity</h3>
                {selectedProject.recentActivity.length > 0 ? (
                  <div className="space-y-3">
                    {selectedProject.recentActivity.map((activity) => (
                      <div key={activity.id} className="flex items-start gap-3">
                        <div className="w-2 h-2 rounded-full bg-primary-900/300 mt-1.5" />
                        <div>
                          <p className="text-sm text-white">{activity.action}</p>
                          <p className="text-xs text-gray-500">
                            {activity.user} • {activity.time}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500">No recent activity</p>
                )}
              </div>

              {/* Actions */}
              <div className="p-6 sticky bottom-0 bg-surface-raised/80">
                <div className="flex gap-2">
                  {selectedProject.status === 'in_progress' && (
                    <>
                      <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                        Update Progress
                      </button>
                      <button className="flex-1 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                        View Tasks
                      </button>
                    </>
                  )}
                  {selectedProject.status === 'planned' && (
                    <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                      <Play className="w-4 h-4" />
                      Start Project
                    </button>
                  )}
                  {selectedProject.status === 'completed' && (
                    <button className="flex-1 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                      View Summary Report
                    </button>
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div className="p-8 text-center text-gray-500">
              <Calendar className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>Select a project to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Create Project Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Migration Project</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Project Name
                </label>
                <input
                  type="text"
                  className="w-full px-3 py-2 glass-input text-sm"
                  placeholder="e.g., Analytics Platform Migration"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Application
                </label>
                <select className="w-full px-3 py-2 glass-input text-sm">
                  <option value="">Select an application</option>
                  <option value="1">Core ERP System</option>
                  <option value="2">Analytics Platform</option>
                  <option value="3">HR Management System</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Playbook
                </label>
                <select className="w-full px-3 py-2 glass-input text-sm">
                  <option value="">Select a playbook</option>
                  <option value="1">Cloud Rehosting Playbook</option>
                  <option value="2">Application Retirement Playbook</option>
                  <option value="3">Platform Modernization Playbook</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Start Date
                  </label>
                  <input
                    type="date"
                    className="w-full px-3 py-2 glass-input text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Target Date
                  </label>
                  <input
                    type="date"
                    className="w-full px-3 py-2 glass-input text-sm"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Description
                </label>
                <textarea
                  className="w-full px-3 py-2 glass-input text-sm"
                  rows={3}
                  placeholder="Describe the project goals and scope"
                />
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Project
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

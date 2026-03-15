import { useState } from 'react';
import {
  Plus,
  Search,
  AlertOctagon,
  Clock,
  CheckCircle,
  User,
  MessageSquare,
  Link2,
  X,
  ChevronRight,
} from 'lucide-react';

// Mock data
const incidents = [
  {
    id: 'INC-001',
    title: 'Production Agent Unresponsive',
    description: 'Agent prod-agent-01 has stopped responding to health checks and is not processing tasks.',
    severity: 'critical',
    status: 'investigating',
    assignee: { name: 'John Doe', avatar: null },
    relatedAlerts: 3,
    created: '2024-01-15T10:00:00Z',
    updated: '2024-01-15T10:45:00Z',
    timeline: [
      { type: 'created', user: 'System', time: '10:00 AM', content: 'Incident created from alert: High CPU Usage' },
      { type: 'comment', user: 'John Doe', time: '10:15 AM', content: 'Investigating the issue. Initial analysis shows memory leak.' },
      { type: 'status', user: 'John Doe', time: '10:30 AM', content: 'Status changed to investigating' },
      { type: 'comment', user: 'Jane Smith', time: '10:45 AM', content: 'Rolling back the latest deployment as a precaution.' },
    ],
  },
  {
    id: 'INC-002',
    title: 'Elevated Error Rates in Data Pipeline',
    description: 'The Salesforce sync integration is experiencing higher than normal failure rates.',
    severity: 'high',
    status: 'acknowledged',
    assignee: { name: 'Jane Smith', avatar: null },
    relatedAlerts: 2,
    created: '2024-01-15T08:30:00Z',
    updated: '2024-01-15T09:00:00Z',
    timeline: [
      { type: 'created', user: 'System', time: '8:30 AM', content: 'Incident created from alert: Integration Failure Rate' },
      { type: 'status', user: 'Jane Smith', time: '9:00 AM', content: 'Status changed to acknowledged' },
    ],
  },
  {
    id: 'INC-003',
    title: 'Database Connection Pool Exhaustion',
    description: 'The data service is running out of database connections during peak hours.',
    severity: 'medium',
    status: 'resolved',
    assignee: { name: 'Bob Wilson', avatar: null },
    relatedAlerts: 1,
    created: '2024-01-14T15:00:00Z',
    updated: '2024-01-14T17:30:00Z',
    resolved: '2024-01-14T17:30:00Z',
    timeline: [
      { type: 'created', user: 'System', time: 'Yesterday 3:00 PM', content: 'Incident created from alert: Database Connection Pool' },
      { type: 'comment', user: 'Bob Wilson', time: 'Yesterday 3:30 PM', content: 'Increased connection pool size.' },
      { type: 'resolved', user: 'Bob Wilson', time: 'Yesterday 5:30 PM', content: 'Incident resolved. Pool size increased and monitoring added.' },
    ],
  },
];

export function IncidentsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [selectedIncident, setSelectedIncident] = useState<typeof incidents[0] | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-900/40 text-red-400 border-red-800';
      case 'high':
        return 'bg-orange-900/40 text-orange-400 border-orange-800';
      case 'medium':
        return 'bg-yellow-900/40 text-yellow-400 border-yellow-800';
      case 'low':
        return 'bg-blue-900/40 text-blue-400 border-blue-800';
      default:
        return 'bg-gray-700/50 text-gray-300 border-surface-border';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'open':
        return 'bg-red-900/40 text-red-400';
      case 'acknowledged':
        return 'bg-yellow-900/40 text-yellow-400';
      case 'investigating':
        return 'bg-blue-900/40 text-blue-400';
      case 'resolved':
        return 'bg-green-900/40 text-green-400';
      case 'closed':
        return 'bg-gray-700/50 text-gray-300';
      default:
        return 'bg-gray-700/50 text-gray-300';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'open':
      case 'acknowledged':
        return <AlertOctagon className="w-4 h-4" />;
      case 'investigating':
        return <Clock className="w-4 h-4" />;
      case 'resolved':
      case 'closed':
        return <CheckCircle className="w-4 h-4" />;
      default:
        return null;
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const getTimelineIcon = (type: string) => {
    switch (type) {
      case 'comment':
        return <MessageSquare className="w-4 h-4 text-blue-500" />;
      case 'status':
        return <Clock className="w-4 h-4 text-yellow-500" />;
      case 'resolved':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      default:
        return <AlertOctagon className="w-4 h-4 text-red-500" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Incidents</h1>
          <p className="text-gray-500">Track and manage operational incidents</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Incident
        </button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg p-4 border border-surface-border">
          <p className="text-sm text-gray-500">Open</p>
          <p className="text-2xl font-bold text-red-600">
            {incidents.filter((i) => i.status === 'open' || i.status === 'acknowledged').length}
          </p>
        </div>
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg p-4 border border-surface-border">
          <p className="text-sm text-gray-500">Investigating</p>
          <p className="text-2xl font-bold text-blue-600">
            {incidents.filter((i) => i.status === 'investigating').length}
          </p>
        </div>
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg p-4 border border-surface-border">
          <p className="text-sm text-gray-500">Resolved Today</p>
          <p className="text-2xl font-bold text-green-600">
            {incidents.filter((i) => i.status === 'resolved').length}
          </p>
        </div>
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg p-4 border border-surface-border">
          <p className="text-sm text-gray-500">Avg Resolution Time</p>
          <p className="text-2xl font-bold text-white">2.5h</p>
        </div>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search incidents..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 glass-input text-sm"
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="px-3 py-2 glass-input text-sm"
        >
          <option value="all">All Statuses</option>
          <option value="open">Open</option>
          <option value="acknowledged">Acknowledged</option>
          <option value="investigating">Investigating</option>
          <option value="resolved">Resolved</option>
          <option value="closed">Closed</option>
        </select>
      </div>

      {/* Incidents List & Detail View */}
      <div className="grid grid-cols-3 gap-6">
        {/* List */}
        <div className="col-span-2 space-y-4">
          {incidents.map((incident) => (
            <div
              key={incident.id}
              onClick={() => setSelectedIncident(incident)}
              className={`bg-surface-raised/80 backdrop-blur-glass rounded-xl p-5 shadow-glass border cursor-pointer transition-all ${
                selectedIncident?.id === incident.id
                  ? 'border-primary-500 ring-2 ring-primary-100'
                  : 'border-surface-border hover:border-surface-border'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span className="text-xs font-mono text-gray-500">{incident.id}</span>
                    <span
                      className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                        incident.severity
                      )}`}
                    >
                      {incident.severity}
                    </span>
                    <span
                      className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${getStatusColor(
                        incident.status
                      )}`}
                    >
                      {getStatusIcon(incident.status)}
                      {incident.status}
                    </span>
                  </div>
                  <h3 className="text-base font-semibold text-white mb-1">{incident.title}</h3>
                  <p className="text-sm text-gray-500 line-clamp-2">{incident.description}</p>
                </div>
                <ChevronRight className="w-5 h-5 text-gray-400" />
              </div>
              <div className="flex items-center justify-between mt-4 pt-3 border-t border-surface-border">
                <div className="flex items-center gap-4 text-xs text-gray-500">
                  <span className="flex items-center gap-1">
                    <User className="w-3 h-3" />
                    {incident.assignee.name}
                  </span>
                  <span className="flex items-center gap-1">
                    <Link2 className="w-3 h-3" />
                    {incident.relatedAlerts} alerts
                  </span>
                </div>
                <span className="text-xs text-gray-400">
                  Updated {formatDate(incident.updated)}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Detail Panel */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border h-fit sticky top-6">
          {selectedIncident ? (
            <div>
              <div className="p-5 border-b border-surface-border">
                <div className="flex items-center justify-between mb-3">
                  <span className="text-xs font-mono text-gray-500">{selectedIncident.id}</span>
                  <div className="flex items-center gap-2">
                    <span
                      className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                        selectedIncident.severity
                      )}`}
                    >
                      {selectedIncident.severity}
                    </span>
                    <span
                      className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${getStatusColor(
                        selectedIncident.status
                      )}`}
                    >
                      {selectedIncident.status}
                    </span>
                  </div>
                </div>
                <h3 className="text-lg font-semibold text-white mb-2">
                  {selectedIncident.title}
                </h3>
                <p className="text-sm text-gray-400">{selectedIncident.description}</p>
              </div>

              <div className="p-5 border-b border-surface-border">
                <h4 className="text-sm font-medium text-white mb-3">Details</h4>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500">Assignee</span>
                    <span className="text-white">{selectedIncident.assignee.name}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Created</span>
                    <span className="text-white">{formatDate(selectedIncident.created)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500">Related Alerts</span>
                    <span className="text-primary-600 cursor-pointer hover:underline">
                      {selectedIncident.relatedAlerts} alerts
                    </span>
                  </div>
                </div>
              </div>

              <div className="p-5 border-b border-surface-border">
                <h4 className="text-sm font-medium text-white mb-3">Timeline</h4>
                <div className="space-y-4">
                  {selectedIncident.timeline.map((event, i) => (
                    <div key={i} className="flex gap-3">
                      <div className="mt-0.5">{getTimelineIcon(event.type)}</div>
                      <div className="flex-1">
                        <p className="text-sm text-white">{event.content}</p>
                        <p className="text-xs text-gray-500 mt-0.5">
                          {event.user} · {event.time}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="p-5">
                <div className="flex gap-2">
                  <input
                    type="text"
                    placeholder="Add a comment..."
                    className="flex-1 px-3 py-2 glass-input text-sm"
                  />
                  <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                    Send
                  </button>
                </div>
                <div className="flex gap-2 mt-3">
                  {selectedIncident.status !== 'resolved' && (
                    <button className="flex-1 px-3 py-2 border border-green-800 text-green-400 rounded-lg text-sm font-medium hover:bg-white/5">
                      Resolve
                    </button>
                  )}
                  <button className="flex-1 px-3 py-2 border border-surface-border text-gray-300 rounded-lg text-sm font-medium hover:bg-white/5">
                    Change Status
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-8 text-center text-gray-500">
              <AlertOctagon className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>Select an incident to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Create Incident Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Incident</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Title</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 glass-input text-sm"
                  placeholder="Brief description of the incident"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Description</label>
                <textarea
                  className="w-full px-3 py-2 glass-input text-sm"
                  rows={3}
                  placeholder="Detailed description of the incident"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Severity</label>
                  <select className="w-full px-3 py-2 glass-input text-sm">
                    <option value="critical">Critical</option>
                    <option value="high">High</option>
                    <option value="medium">Medium</option>
                    <option value="low">Low</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Assignee</label>
                  <select className="w-full px-3 py-2 glass-input text-sm">
                    <option>John Doe</option>
                    <option>Jane Smith</option>
                    <option>Bob Wilson</option>
                  </select>
                </div>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Incident
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

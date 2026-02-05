import { useState } from 'react';
import {
  Search,
  Download,
  ChevronRight,
  User,
  Bot,
  Server,
  Calendar,
  Clock,
  Eye,
} from 'lucide-react';

// Mock data
const auditEntries = [
  {
    id: '1',
    actor: { id: 'user-1', name: 'John Doe', type: 'user' },
    action: 'integration.created',
    resourceType: 'integration',
    resourceId: 'int-123',
    resourceName: 'Salesforce Sync',
    timestamp: '2024-01-15T14:32:00Z',
    ipAddress: '192.168.1.100',
    metadata: { source: 'web-ui' },
    changes: {
      before: null,
      after: { name: 'Salesforce Sync', enabled: true },
    },
  },
  {
    id: '2',
    actor: { id: 'user-2', name: 'Jane Smith', type: 'user' },
    action: 'policy.updated',
    resourceType: 'policy',
    resourceId: 'pol-456',
    resourceName: 'API Key Rotation Policy',
    timestamp: '2024-01-15T13:15:00Z',
    ipAddress: '192.168.1.101',
    metadata: { source: 'web-ui' },
    changes: {
      before: { enforcement_mode: 'warn' },
      after: { enforcement_mode: 'enforce' },
    },
  },
  {
    id: '3',
    actor: { id: 'system', name: 'System', type: 'system' },
    action: 'alert.fired',
    resourceType: 'alert',
    resourceId: 'alert-789',
    resourceName: 'High CPU Usage',
    timestamp: '2024-01-15T12:45:00Z',
    ipAddress: null,
    metadata: { triggered_value: 92.5, threshold: 80 },
    changes: null,
  },
  {
    id: '4',
    actor: { id: 'agent-1', name: 'prod-agent-01', type: 'agent' },
    action: 'task.completed',
    resourceType: 'task',
    resourceId: 'task-321',
    resourceName: 'Database Query Task',
    timestamp: '2024-01-15T12:30:00Z',
    ipAddress: '10.0.0.50',
    metadata: { duration_ms: 1250, rows_returned: 500 },
    changes: null,
  },
  {
    id: '5',
    actor: { id: 'user-1', name: 'John Doe', type: 'user' },
    action: 'connection.deleted',
    resourceType: 'connection',
    resourceId: 'conn-654',
    resourceName: 'Legacy Oracle DB',
    timestamp: '2024-01-15T11:20:00Z',
    ipAddress: '192.168.1.100',
    metadata: { reason: 'Deprecated' },
    changes: {
      before: { name: 'Legacy Oracle DB', type: 'oracle', enabled: false },
      after: null,
    },
  },
  {
    id: '6',
    actor: { id: 'user-3', name: 'Bob Wilson', type: 'user' },
    action: 'approval.approved',
    resourceType: 'approval_request',
    resourceId: 'apr-987',
    resourceName: 'New Integration Request',
    timestamp: '2024-01-15T10:45:00Z',
    ipAddress: '192.168.1.102',
    metadata: { stage: 2, comment: 'Looks good, approved.' },
    changes: {
      before: { status: 'pending', current_stage: 1 },
      after: { status: 'approved', current_stage: 2 },
    },
  },
  {
    id: '7',
    actor: { id: 'system', name: 'System', type: 'system' },
    action: 'incident.auto_created',
    resourceType: 'incident',
    resourceId: 'inc-111',
    resourceName: 'Critical Alert Incident',
    timestamp: '2024-01-15T10:30:00Z',
    ipAddress: null,
    metadata: { triggered_by_alert: 'alert-789' },
    changes: null,
  },
  {
    id: '8',
    actor: { id: 'user-2', name: 'Jane Smith', type: 'user' },
    action: 'standard.created',
    resourceType: 'standard',
    resourceId: 'std-222',
    resourceName: 'API Versioning Standard',
    timestamp: '2024-01-15T09:15:00Z',
    ipAddress: '192.168.1.101',
    metadata: { category: 'api', version: 1 },
    changes: {
      before: null,
      after: { name: 'API Versioning Standard', category: 'api' },
    },
  },
];

const actionCategories = [
  { value: 'all', label: 'All Actions' },
  { value: 'created', label: 'Created' },
  { value: 'updated', label: 'Updated' },
  { value: 'deleted', label: 'Deleted' },
  { value: 'approved', label: 'Approved' },
  { value: 'rejected', label: 'Rejected' },
];

const resourceTypes = [
  { value: 'all', label: 'All Resources' },
  { value: 'integration', label: 'Integrations' },
  { value: 'connection', label: 'Connections' },
  { value: 'policy', label: 'Policies' },
  { value: 'standard', label: 'Standards' },
  { value: 'alert', label: 'Alerts' },
  { value: 'incident', label: 'Incidents' },
  { value: 'approval_request', label: 'Approvals' },
  { value: 'task', label: 'Tasks' },
];

const actorTypes = [
  { value: 'all', label: 'All Actors' },
  { value: 'user', label: 'Users' },
  { value: 'system', label: 'System' },
  { value: 'agent', label: 'Agents' },
];

export function AuditLogPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [actionFilter, setActionFilter] = useState('all');
  const [resourceFilter, setResourceFilter] = useState('all');
  const [actorFilter, setActorFilter] = useState('all');
  const [selectedEntry, setSelectedEntry] = useState<typeof auditEntries[0] | null>(null);
  const [dateRange, setDateRange] = useState('7d');

  const getActorIcon = (type: string) => {
    switch (type) {
      case 'user':
        return <User className="w-4 h-4" />;
      case 'system':
        return <Server className="w-4 h-4" />;
      case 'agent':
        return <Bot className="w-4 h-4" />;
      default:
        return <User className="w-4 h-4" />;
    }
  };

  const getActionColor = (action: string) => {
    if (action.includes('created')) return 'text-green-600 bg-green-50';
    if (action.includes('updated')) return 'text-blue-600 bg-blue-50';
    if (action.includes('deleted')) return 'text-red-600 bg-red-50';
    if (action.includes('approved')) return 'text-green-600 bg-green-50';
    if (action.includes('rejected')) return 'text-red-600 bg-red-50';
    if (action.includes('fired')) return 'text-orange-600 bg-orange-50';
    if (action.includes('completed')) return 'text-purple-600 bg-purple-50';
    return 'text-gray-600 bg-gray-50';
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return {
      date: date.toLocaleDateString(),
      time: date.toLocaleTimeString(),
    };
  };

  const formatAction = (action: string) => {
    return action.replace('.', ' → ').replace(/_/g, ' ');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Audit Log</h1>
          <p className="text-gray-500">Complete history of all platform actions</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50">
          <Download className="w-4 h-4" />
          Export
        </button>
      </div>

      {/* Filters */}
      <div className="bg-white rounded-xl p-4 shadow-sm border border-gray-100">
        <div className="flex items-center gap-4 flex-wrap">
          <div className="relative flex-1 min-w-[200px] max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search by resource, actor, or action..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
          <select
            value={dateRange}
            onChange={(e) => setDateRange(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="1h">Last 1 hour</option>
            <option value="24h">Last 24 hours</option>
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
          </select>
          <select
            value={actionFilter}
            onChange={(e) => setActionFilter(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            {actionCategories.map((cat) => (
              <option key={cat.value} value={cat.value}>
                {cat.label}
              </option>
            ))}
          </select>
          <select
            value={resourceFilter}
            onChange={(e) => setResourceFilter(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            {resourceTypes.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
          <select
            value={actorFilter}
            onChange={(e) => setActorFilter(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            {actorTypes.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-3 gap-6">
        {/* Audit Log List */}
        <div className="col-span-2 bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-100">
            <div className="flex items-center justify-between">
              <h2 className="font-semibold text-gray-900">Activity Log</h2>
              <span className="text-sm text-gray-500">{auditEntries.length} entries</span>
            </div>
          </div>
          <div className="divide-y divide-gray-100 max-h-[600px] overflow-y-auto">
            {auditEntries.map((entry) => {
              const { date, time } = formatTimestamp(entry.timestamp);
              return (
                <div
                  key={entry.id}
                  onClick={() => setSelectedEntry(entry)}
                  className={`px-6 py-4 hover:bg-gray-50 cursor-pointer transition-colors ${
                    selectedEntry?.id === entry.id ? 'bg-primary-50' : ''
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-3">
                      <div className="p-2 bg-gray-100 rounded-lg">
                        {getActorIcon(entry.actor.type)}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-gray-900">{entry.actor.name}</span>
                          <span
                            className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${getActionColor(
                              entry.action
                            )}`}
                          >
                            {formatAction(entry.action)}
                          </span>
                        </div>
                        <p className="text-sm text-gray-600 mt-0.5">
                          {entry.resourceType}:{' '}
                          <span className="font-medium">{entry.resourceName}</span>
                        </p>
                        <div className="flex items-center gap-3 mt-2 text-xs text-gray-500">
                          <span className="flex items-center gap-1">
                            <Calendar className="w-3 h-3" />
                            {date}
                          </span>
                          <span className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            {time}
                          </span>
                          {entry.ipAddress && (
                            <span className="font-mono">{entry.ipAddress}</span>
                          )}
                        </div>
                      </div>
                    </div>
                    <ChevronRight className="w-5 h-5 text-gray-400" />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Detail Panel */}
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
          {selectedEntry ? (
            <>
              <div className="px-6 py-4 border-b border-gray-100">
                <h2 className="font-semibold text-gray-900">Entry Details</h2>
              </div>
              <div className="p-6 space-y-6">
                {/* Actor Info */}
                <div>
                  <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                    Actor
                  </h3>
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-gray-100 rounded-lg">
                      {getActorIcon(selectedEntry.actor.type)}
                    </div>
                    <div>
                      <p className="font-medium text-gray-900">{selectedEntry.actor.name}</p>
                      <p className="text-sm text-gray-500 capitalize">{selectedEntry.actor.type}</p>
                    </div>
                  </div>
                </div>

                {/* Action */}
                <div>
                  <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                    Action
                  </h3>
                  <span
                    className={`text-sm font-medium px-3 py-1 rounded-full ${getActionColor(
                      selectedEntry.action
                    )}`}
                  >
                    {formatAction(selectedEntry.action)}
                  </span>
                </div>

                {/* Resource */}
                <div>
                  <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                    Resource
                  </h3>
                  <p className="text-sm text-gray-900">{selectedEntry.resourceName}</p>
                  <p className="text-xs text-gray-500 font-mono mt-1">
                    {selectedEntry.resourceType}/{selectedEntry.resourceId}
                  </p>
                </div>

                {/* Timestamp */}
                <div>
                  <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                    Timestamp
                  </h3>
                  <p className="text-sm text-gray-900">
                    {new Date(selectedEntry.timestamp).toLocaleString()}
                  </p>
                </div>

                {/* IP Address */}
                {selectedEntry.ipAddress && (
                  <div>
                    <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                      IP Address
                    </h3>
                    <p className="text-sm font-mono text-gray-900">{selectedEntry.ipAddress}</p>
                  </div>
                )}

                {/* Metadata */}
                {selectedEntry.metadata && Object.keys(selectedEntry.metadata).length > 0 && (
                  <div>
                    <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                      Metadata
                    </h3>
                    <pre className="text-xs bg-gray-50 p-3 rounded-lg overflow-x-auto">
                      {JSON.stringify(selectedEntry.metadata, null, 2)}
                    </pre>
                  </div>
                )}

                {/* Changes */}
                {selectedEntry.changes && (
                  <div>
                    <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                      Changes
                    </h3>
                    <div className="space-y-3">
                      {selectedEntry.changes.before && (
                        <div>
                          <p className="text-xs font-medium text-red-600 mb-1">Before</p>
                          <pre className="text-xs bg-red-50 p-3 rounded-lg overflow-x-auto text-red-800">
                            {JSON.stringify(selectedEntry.changes.before, null, 2)}
                          </pre>
                        </div>
                      )}
                      {selectedEntry.changes.after && (
                        <div>
                          <p className="text-xs font-medium text-green-600 mb-1">After</p>
                          <pre className="text-xs bg-green-50 p-3 rounded-lg overflow-x-auto text-green-800">
                            {JSON.stringify(selectedEntry.changes.after, null, 2)}
                          </pre>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="flex items-center justify-center h-full py-20">
              <div className="text-center">
                <Eye className="w-12 h-12 text-gray-300 mx-auto mb-3" />
                <p className="text-gray-500">Select an entry to view details</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

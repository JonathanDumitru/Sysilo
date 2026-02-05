import { useState } from 'react';
import {
  Bell,
  Plus,
  Search,
  AlertTriangle,
  CheckCircle,
  Clock,
  MoreVertical,
  X,
} from 'lucide-react';

// Mock data
const alertRules = [
  {
    id: '1',
    name: 'High CPU Usage',
    description: 'Alert when CPU usage exceeds threshold',
    metric: 'cpu_usage',
    condition: 'gt',
    threshold: 80,
    severity: 'critical',
    enabled: true,
    channels: ['slack-ops', 'pagerduty'],
  },
  {
    id: '2',
    name: 'Integration Failure Rate',
    description: 'Alert when integration failure rate is high',
    metric: 'failure_rate',
    condition: 'gt',
    threshold: 5,
    severity: 'high',
    enabled: true,
    channels: ['slack-ops'],
  },
  {
    id: '3',
    name: 'Agent Disconnected',
    description: 'Alert when an agent loses connection',
    metric: 'agent_connected',
    condition: 'eq',
    threshold: 0,
    severity: 'high',
    enabled: true,
    channels: ['slack-ops', 'email'],
  },
  {
    id: '4',
    name: 'Low Disk Space',
    description: 'Alert when disk space is below threshold',
    metric: 'disk_free_percent',
    condition: 'lt',
    threshold: 10,
    severity: 'medium',
    enabled: false,
    channels: ['email'],
  },
];

const alertInstances = [
  {
    id: '1',
    ruleName: 'High CPU Usage',
    severity: 'critical',
    resource: 'prod-agent-01',
    triggeredValue: 92.5,
    status: 'firing',
    triggeredAt: '2024-01-15T10:30:00Z',
  },
  {
    id: '2',
    ruleName: 'Integration Failure Rate',
    severity: 'high',
    resource: 'Salesforce Sync',
    triggeredValue: 8.2,
    status: 'firing',
    triggeredAt: '2024-01-15T10:15:00Z',
  },
  {
    id: '3',
    ruleName: 'Agent Disconnected',
    severity: 'high',
    resource: 'dev-agent-02',
    triggeredValue: 0,
    status: 'acknowledged',
    triggeredAt: '2024-01-15T09:45:00Z',
  },
  {
    id: '4',
    ruleName: 'High CPU Usage',
    severity: 'critical',
    resource: 'prod-agent-02',
    triggeredValue: 85.3,
    status: 'resolved',
    triggeredAt: '2024-01-15T08:00:00Z',
    resolvedAt: '2024-01-15T08:30:00Z',
  },
];

type TabType = 'instances' | 'rules';

export function AlertsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('instances');
  const [searchQuery, setSearchQuery] = useState('');
  const [severityFilter, setSeverityFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [showCreateModal, setShowCreateModal] = useState(false);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-100 text-red-700';
      case 'high':
        return 'bg-orange-100 text-orange-700';
      case 'medium':
        return 'bg-yellow-100 text-yellow-700';
      case 'low':
        return 'bg-blue-100 text-blue-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'firing':
        return <AlertTriangle className="w-4 h-4 text-red-500" />;
      case 'acknowledged':
        return <Clock className="w-4 h-4 text-yellow-500" />;
      case 'resolved':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      default:
        return null;
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const getConditionLabel = (condition: string) => {
    switch (condition) {
      case 'gt':
        return '>';
      case 'lt':
        return '<';
      case 'eq':
        return '=';
      case 'gte':
        return '≥';
      case 'lte':
        return '≤';
      default:
        return condition;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Alerts</h1>
          <p className="text-gray-500">Manage alert rules and view triggered alerts</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Alert Rule
        </button>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="flex gap-6">
          <button
            onClick={() => setActiveTab('instances')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'instances'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            Alert Instances
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-red-100 text-red-700">
              {alertInstances.filter((a) => a.status === 'firing').length}
            </span>
          </button>
          <button
            onClick={() => setActiveTab('rules')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'rules'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            Alert Rules
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-700">
              {alertRules.length}
            </span>
          </button>
        </nav>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search alerts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <select
          value={severityFilter}
          onChange={(e) => setSeverityFilter(e.target.value)}
          className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        >
          <option value="all">All Severities</option>
          <option value="critical">Critical</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
        {activeTab === 'instances' && (
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="all">All Statuses</option>
            <option value="firing">Firing</option>
            <option value="acknowledged">Acknowledged</option>
            <option value="resolved">Resolved</option>
          </select>
        )}
      </div>

      {/* Alert Instances Tab */}
      {activeTab === 'instances' && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                <th className="px-6 py-3">Status</th>
                <th className="px-6 py-3">Alert</th>
                <th className="px-6 py-3">Severity</th>
                <th className="px-6 py-3">Resource</th>
                <th className="px-6 py-3">Triggered Value</th>
                <th className="px-6 py-3">Triggered At</th>
                <th className="px-6 py-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {alertInstances.map((alert) => (
                <tr key={alert.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      {getStatusIcon(alert.status)}
                      <span className="text-sm capitalize">{alert.status}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm font-medium text-gray-900">{alert.ruleName}</span>
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                        alert.severity
                      )}`}
                    >
                      {alert.severity}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-600">{alert.resource}</td>
                  <td className="px-6 py-4 text-sm font-mono text-gray-600">
                    {alert.triggeredValue}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {formatDate(alert.triggeredAt)}
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      {alert.status === 'firing' && (
                        <button className="text-xs font-medium text-primary-600 hover:text-primary-700">
                          Acknowledge
                        </button>
                      )}
                      {alert.status !== 'resolved' && (
                        <button className="text-xs font-medium text-green-600 hover:text-green-700">
                          Resolve
                        </button>
                      )}
                      <button className="text-xs font-medium text-gray-500 hover:text-gray-700">
                        Create Incident
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Alert Rules Tab */}
      {activeTab === 'rules' && (
        <div className="grid gap-4">
          {alertRules.map((rule) => (
            <div
              key={rule.id}
              className="bg-white rounded-xl p-6 shadow-sm border border-gray-100"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div
                    className={`p-2 rounded-lg ${
                      rule.enabled ? 'bg-primary-50' : 'bg-gray-100'
                    }`}
                  >
                    <Bell
                      className={`w-5 h-5 ${
                        rule.enabled ? 'text-primary-600' : 'text-gray-400'
                      }`}
                    />
                  </div>
                  <div>
                    <div className="flex items-center gap-3">
                      <h3 className="text-lg font-semibold text-gray-900">{rule.name}</h3>
                      <span
                        className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                          rule.severity
                        )}`}
                      >
                        {rule.severity}
                      </span>
                      {!rule.enabled && (
                        <span className="text-xs font-medium px-2 py-1 rounded-full bg-gray-100 text-gray-500">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 mt-1">{rule.description}</p>
                    <div className="flex items-center gap-4 mt-3">
                      <span className="text-xs text-gray-500">
                        Condition:{' '}
                        <code className="px-1.5 py-0.5 bg-gray-100 rounded text-gray-700">
                          {rule.metric} {getConditionLabel(rule.condition)} {rule.threshold}
                        </code>
                      </span>
                      <span className="text-xs text-gray-500">
                        Channels:{' '}
                        {rule.channels.map((ch) => (
                          <span
                            key={ch}
                            className="inline-block px-1.5 py-0.5 bg-gray-100 rounded text-gray-700 mr-1"
                          >
                            {ch}
                          </span>
                        ))}
                      </span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button className="p-2 text-gray-400 hover:text-gray-600">
                    <MoreVertical className="w-5 h-5" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create Alert Rule Modal (placeholder) */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Alert Rule</h2>
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
                  placeholder="Enter alert name"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Metric</label>
                <select className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
                  <option>cpu_usage</option>
                  <option>memory_usage</option>
                  <option>failure_rate</option>
                  <option>latency_p99</option>
                </select>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Condition</label>
                  <select className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
                    <option value="gt">Greater than</option>
                    <option value="lt">Less than</option>
                    <option value="eq">Equal to</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Threshold</label>
                  <input
                    type="number"
                    className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                    placeholder="80"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Severity</label>
                <select className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
                  <option value="critical">Critical</option>
                  <option value="high">High</option>
                  <option value="medium">Medium</option>
                  <option value="low">Low</option>
                </select>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Rule
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

import { useState } from 'react';
import {
  Bell,
  CheckCircle,
  Clock,
  AlertOctagon,
  TrendingUp,
} from 'lucide-react';

// Mock data for the dashboard
const systemHealth = {
  overall: 'healthy',
  services: [
    { name: 'API Gateway', status: 'healthy', latency: '45ms' },
    { name: 'Integration Service', status: 'healthy', latency: '120ms' },
    { name: 'Agent Gateway', status: 'healthy', latency: '32ms' },
    { name: 'Data Service', status: 'degraded', latency: '890ms' },
  ],
};

const alertSummary = {
  critical: 1,
  high: 2,
  medium: 5,
  low: 8,
};

const recentAlerts = [
  {
    id: '1',
    name: 'High CPU Usage',
    severity: 'critical',
    resource: 'prod-agent-01',
    time: '5 min ago',
    status: 'firing',
  },
  {
    id: '2',
    name: 'Integration Failure Rate',
    severity: 'high',
    resource: 'Salesforce Sync',
    time: '15 min ago',
    status: 'firing',
  },
  {
    id: '3',
    name: 'Database Connection Pool',
    severity: 'medium',
    resource: 'data-service',
    time: '1 hour ago',
    status: 'acknowledged',
  },
];

const activeIncidents = [
  {
    id: 'INC-001',
    title: 'Production Agent Unresponsive',
    severity: 'critical',
    status: 'investigating',
    assignee: 'John Doe',
    created: '30 min ago',
  },
  {
    id: 'INC-002',
    title: 'Elevated Error Rates in Data Pipeline',
    severity: 'high',
    status: 'acknowledged',
    assignee: 'Jane Smith',
    created: '2 hours ago',
  },
];

const metrics = [
  { name: 'Uptime', value: '99.95%', trend: '+0.02%', icon: TrendingUp },
  { name: 'Avg Response Time', value: '145ms', trend: '-12ms', icon: Clock },
  { name: 'Active Alerts', value: '16', trend: '+3', icon: Bell },
  { name: 'Open Incidents', value: '2', trend: '0', icon: AlertOctagon },
];

export function OperationsDashboardPage() {
  const [timeRange, setTimeRange] = useState('24h');

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-100 text-red-700 border-red-200';
      case 'high':
        return 'bg-orange-100 text-orange-700 border-orange-200';
      case 'medium':
        return 'bg-yellow-100 text-yellow-700 border-yellow-200';
      case 'low':
        return 'bg-blue-100 text-blue-700 border-blue-200';
      default:
        return 'bg-gray-100 text-gray-700 border-gray-200';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Operations Center</h1>
          <p className="text-gray-500">Real-time monitoring and incident management</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="1h">Last 1 hour</option>
            <option value="6h">Last 6 hours</option>
            <option value="24h">Last 24 hours</option>
            <option value="7d">Last 7 days</option>
          </select>
          <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
            Create Incident
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        {metrics.map((metric) => (
          <div
            key={metric.name}
            className="bg-white rounded-xl p-6 shadow-sm border border-gray-100"
          >
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-50 rounded-lg">
                <metric.icon className="w-5 h-5 text-primary-600" />
              </div>
              <span
                className={`text-xs font-medium ${
                  metric.trend.startsWith('+') ? 'text-green-600' : metric.trend.startsWith('-') ? 'text-red-600' : 'text-gray-500'
                }`}
              >
                {metric.trend}
              </span>
            </div>
            <div className="mt-4">
              <p className="text-3xl font-bold text-gray-900">{metric.value}</p>
              <p className="text-sm font-medium text-gray-500">{metric.name}</p>
            </div>
          </div>
        ))}
      </div>

      {/* System Health & Alert Summary */}
      <div className="grid grid-cols-3 gap-6">
        {/* System Health */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">System Health</h2>
            <span
              className={`flex items-center gap-1 text-sm font-medium ${
                systemHealth.overall === 'healthy' ? 'text-green-600' : 'text-yellow-600'
              }`}
            >
              <CheckCircle className="w-4 h-4" />
              {systemHealth.overall}
            </span>
          </div>
          <div className="space-y-3">
            {systemHealth.services.map((service) => (
              <div
                key={service.name}
                className="flex items-center justify-between py-2 border-b border-gray-50 last:border-0"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`w-2 h-2 rounded-full ${
                      service.status === 'healthy' ? 'bg-green-500' : 'bg-yellow-500'
                    }`}
                  />
                  <span className="text-sm text-gray-700">{service.name}</span>
                </div>
                <span className="text-xs text-gray-500">{service.latency}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Alert Summary */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Alert Summary</h2>
          <div className="grid grid-cols-2 gap-3">
            <div className="p-3 bg-red-50 rounded-lg border border-red-100">
              <p className="text-2xl font-bold text-red-700">{alertSummary.critical}</p>
              <p className="text-xs text-red-600">Critical</p>
            </div>
            <div className="p-3 bg-orange-50 rounded-lg border border-orange-100">
              <p className="text-2xl font-bold text-orange-700">{alertSummary.high}</p>
              <p className="text-xs text-orange-600">High</p>
            </div>
            <div className="p-3 bg-yellow-50 rounded-lg border border-yellow-100">
              <p className="text-2xl font-bold text-yellow-700">{alertSummary.medium}</p>
              <p className="text-xs text-yellow-600">Medium</p>
            </div>
            <div className="p-3 bg-blue-50 rounded-lg border border-blue-100">
              <p className="text-2xl font-bold text-blue-700">{alertSummary.low}</p>
              <p className="text-xs text-blue-600">Low</p>
            </div>
          </div>
        </div>

        {/* Active Incidents */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">Active Incidents</h2>
            <a href="/operations/incidents" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          <div className="space-y-3">
            {activeIncidents.map((incident) => (
              <div
                key={incident.id}
                className="p-3 bg-gray-50 rounded-lg border border-gray-100"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <span className="text-xs font-mono text-gray-500">{incident.id}</span>
                    <p className="text-sm font-medium text-gray-900">{incident.title}</p>
                  </div>
                  <span
                    className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                      incident.severity
                    )}`}
                  >
                    {incident.severity}
                  </span>
                </div>
                <div className="mt-2 flex items-center justify-between text-xs text-gray-500">
                  <span>{incident.assignee}</span>
                  <span>{incident.created}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Recent Alerts */}
      <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Recent Alerts</h2>
          <a href="/operations/alerts" className="text-sm text-primary-600 hover:text-primary-700">
            View all alerts
          </a>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                <th className="pb-3">Alert</th>
                <th className="pb-3">Severity</th>
                <th className="pb-3">Resource</th>
                <th className="pb-3">Status</th>
                <th className="pb-3">Time</th>
                <th className="pb-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {recentAlerts.map((alert) => (
                <tr key={alert.id} className="text-sm">
                  <td className="py-3 font-medium text-gray-900">{alert.name}</td>
                  <td className="py-3">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                        alert.severity
                      )}`}
                    >
                      {alert.severity}
                    </span>
                  </td>
                  <td className="py-3 text-gray-600">{alert.resource}</td>
                  <td className="py-3">
                    <span
                      className={`flex items-center gap-1 text-xs font-medium ${
                        alert.status === 'firing' ? 'text-red-600' : 'text-yellow-600'
                      }`}
                    >
                      <span
                        className={`w-1.5 h-1.5 rounded-full ${
                          alert.status === 'firing' ? 'bg-red-500' : 'bg-yellow-500'
                        }`}
                      />
                      {alert.status}
                    </span>
                  </td>
                  <td className="py-3 text-gray-500">{alert.time}</td>
                  <td className="py-3">
                    <button className="text-primary-600 hover:text-primary-700 text-xs font-medium">
                      Acknowledge
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

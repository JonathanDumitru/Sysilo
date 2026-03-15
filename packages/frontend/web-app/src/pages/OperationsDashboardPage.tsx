import { useMemo, useState } from 'react';
import {
  Bell,
  CheckCircle,
  Clock,
  AlertOctagon,
  TrendingUp,
  Loader2,
  AlertTriangle,
} from 'lucide-react';
import {
  useAlertInstances,
  useAcknowledgeAlert,
  useIncidents,
  useMetricAggregations,
  useCreateIncident,
} from '../hooks/useOperations';
import type { AlertInstance, Incident, MetricAggregationParams } from '../services/operations';

// --- Helpers ---

function formatRelativeTime(isoString: string): string {
  const now = Date.now();
  const then = new Date(isoString).getTime();
  const diffMs = now - then;

  if (diffMs < 0) return 'just now';

  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return `${seconds}s ago`;

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function getTimeRangeStart(range: string): string {
  const now = new Date();
  switch (range) {
    case '1h':
      now.setHours(now.getHours() - 1);
      break;
    case '6h':
      now.setHours(now.getHours() - 6);
      break;
    case '24h':
      now.setHours(now.getHours() - 24);
      break;
    case '7d':
      now.setDate(now.getDate() - 7);
      break;
    default:
      now.setHours(now.getHours() - 24);
  }
  return now.toISOString();
}

function getSeverityColor(severity: string) {
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
}

// --- Loading / Error Components ---

function LoadingSkeleton({ className = '' }: { className?: string }) {
  return (
    <div className={`animate-pulse bg-gray-700 rounded ${className}`} />
  );
}

function ErrorBanner({ message }: { message: string }) {
  return (
    <div className="flex items-center gap-2 p-4 bg-red-900/30 border border-red-800 rounded-lg text-sm text-red-400">
      <AlertTriangle className="w-4 h-4 flex-shrink-0" />
      <span>{message}</span>
    </div>
  );
}

// --- Main Page ---

export function OperationsDashboardPage() {
  const [timeRange, setTimeRange] = useState('24h');
  const [showCreateIncident, setShowCreateIncident] = useState(false);
  const [newIncident, setNewIncident] = useState({ title: '', description: '', severity: 'medium' as const });

  // --- Data Hooks ---
  const alertsQuery = useAlertInstances();
  const incidentsQuery = useIncidents('open');

  const metricsParams: MetricAggregationParams = useMemo(
    () => ({
      start_time: getTimeRangeStart(timeRange),
      end_time: new Date().toISOString(),
    }),
    [timeRange]
  );
  const metricsQuery = useMetricAggregations(metricsParams);

  const acknowledgeMutation = useAcknowledgeAlert();
  const createIncidentMutation = useCreateIncident();

  // --- Derived Data ---

  const alerts: AlertInstance[] = alertsQuery.data ?? [];
  const incidents: Incident[] = incidentsQuery.data ?? [];

  const alertSummary = useMemo(() => {
    const counts = { critical: 0, high: 0, medium: 0, low: 0 };
    for (const alert of alerts) {
      if (alert.severity in counts) {
        counts[alert.severity as keyof typeof counts]++;
      }
    }
    return counts;
  }, [alerts]);

  const firingAlerts = useMemo(
    () => alerts.filter((a) => a.status === 'firing' || a.status === 'acknowledged'),
    [alerts]
  );

  // Compute key metrics from aggregations and live data
  const keyMetrics = useMemo(() => {
    const aggregations = metricsQuery.data ?? [];
    const uptimeAgg = aggregations.find((a) => a.metric_name === 'uptime');
    const latencyAgg = aggregations.find((a) => a.metric_name === 'response_time');

    return [
      {
        name: 'Uptime',
        value: uptimeAgg ? `${uptimeAgg.avg_value.toFixed(2)}%` : '--',
        trend: '',
        icon: TrendingUp,
      },
      {
        name: 'Avg Response Time',
        value: latencyAgg ? `${Math.round(latencyAgg.avg_value)}ms` : '--',
        trend: '',
        icon: Clock,
      },
      {
        name: 'Active Alerts',
        value: String(firingAlerts.length),
        trend: '',
        icon: Bell,
      },
      {
        name: 'Open Incidents',
        value: String(incidents.length),
        trend: '',
        icon: AlertOctagon,
      },
    ];
  }, [metricsQuery.data, firingAlerts.length, incidents.length]);

  // --- Handlers ---

  function handleAcknowledge(alertId: string) {
    acknowledgeMutation.mutate(alertId);
  }

  function handleCreateIncident() {
    createIncidentMutation.mutate(
      {
        title: newIncident.title,
        description: newIncident.description,
        severity: newIncident.severity,
      },
      {
        onSuccess: () => {
          setShowCreateIncident(false);
          setNewIncident({ title: '', description: '', severity: 'medium' });
        },
      }
    );
  }

  // --- Global loading / error ---

  const isInitialLoading = alertsQuery.isLoading && incidentsQuery.isLoading;

  if (isInitialLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 text-primary-600 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Operations Center</h1>
          <p className="text-gray-500">Real-time monitoring and incident management</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="px-3 py-2 glass-input text-sm"
          >
            <option value="1h">Last 1 hour</option>
            <option value="6h">Last 6 hours</option>
            <option value="24h">Last 24 hours</option>
            <option value="7d">Last 7 days</option>
          </select>
          <button
            onClick={() => setShowCreateIncident(true)}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
          >
            Create Incident
          </button>
        </div>
      </div>

      {/* Create Incident Modal */}
      {showCreateIncident && (
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <h2 className="text-lg font-semibold text-white mb-4">Create Incident</h2>
          <div className="space-y-3">
            <input
              type="text"
              placeholder="Incident title"
              value={newIncident.title}
              onChange={(e) => setNewIncident({ ...newIncident, title: e.target.value })}
              className="w-full px-3 py-2 glass-input text-sm"
            />
            <textarea
              placeholder="Description"
              value={newIncident.description}
              onChange={(e) => setNewIncident({ ...newIncident, description: e.target.value })}
              className="w-full px-3 py-2 glass-input text-sm"
              rows={3}
            />
            <select
              value={newIncident.severity}
              onChange={(e) =>
                setNewIncident({ ...newIncident, severity: e.target.value as 'critical' | 'high' | 'medium' | 'low' | 'info' })
              }
              className="px-3 py-2 glass-input text-sm"
            >
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
              <option value="info">Info</option>
            </select>
            <div className="flex items-center gap-2">
              <button
                onClick={handleCreateIncident}
                disabled={createIncidentMutation.isPending || !newIncident.title}
                className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 disabled:opacity-50"
              >
                {createIncidentMutation.isPending ? 'Creating...' : 'Create'}
              </button>
              <button
                onClick={() => setShowCreateIncident(false)}
                className="px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-white/5"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Error banners */}
      {alertsQuery.isError && (
        <ErrorBanner message={`Failed to load alerts: ${alertsQuery.error?.message ?? 'Unknown error'}`} />
      )}
      {incidentsQuery.isError && (
        <ErrorBanner message={`Failed to load incidents: ${incidentsQuery.error?.message ?? 'Unknown error'}`} />
      )}
      {metricsQuery.isError && (
        <ErrorBanner message={`Failed to load metrics: ${metricsQuery.error?.message ?? 'Unknown error'}`} />
      )}

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        {keyMetrics.map((metric) => (
          <div
            key={metric.name}
            className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border"
          >
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-900/30 rounded-lg">
                <metric.icon className="w-5 h-5 text-primary-600" />
              </div>
              {metric.trend && (
                <span
                  className={`text-xs font-medium ${
                    metric.trend.startsWith('+') ? 'text-green-600' : metric.trend.startsWith('-') ? 'text-red-600' : 'text-gray-500'
                  }`}
                >
                  {metric.trend}
                </span>
              )}
            </div>
            <div className="mt-4">
              {metricsQuery.isLoading ? (
                <LoadingSkeleton className="h-8 w-20" />
              ) : (
                <p className="text-3xl font-bold text-white">{metric.value}</p>
              )}
              <p className="text-sm font-medium text-gray-500">{metric.name}</p>
            </div>
          </div>
        ))}
      </div>

      {/* System Health & Alert Summary */}
      <div className="grid grid-cols-3 gap-6">
        {/* System Health - static since this comes from a different source */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">System Health</h2>
            <span className="flex items-center gap-1 text-sm font-medium text-green-600">
              <CheckCircle className="w-4 h-4" />
              healthy
            </span>
          </div>
          <div className="space-y-3">
            {[
              { name: 'API Gateway', status: 'healthy', latency: '45ms' },
              { name: 'Integration Service', status: 'healthy', latency: '120ms' },
              { name: 'Agent Gateway', status: 'healthy', latency: '32ms' },
              { name: 'Data Service', status: 'healthy', latency: '65ms' },
            ].map((service) => (
              <div
                key={service.name}
                className="flex items-center justify-between py-2 border-b border-surface-border last:border-0"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`w-2 h-2 rounded-full ${
                      service.status === 'healthy' ? 'bg-green-500' : 'bg-yellow-500'
                    }`}
                  />
                  <span className="text-sm text-gray-300">{service.name}</span>
                </div>
                <span className="text-xs text-gray-500">{service.latency}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Alert Summary */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <h2 className="text-lg font-semibold text-white mb-4">Alert Summary</h2>
          {alertsQuery.isLoading ? (
            <div className="grid grid-cols-2 gap-3">
              {[1, 2, 3, 4].map((i) => (
                <LoadingSkeleton key={i} className="h-16" />
              ))}
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3">
              <div className="p-3 bg-red-900/30 rounded-lg border border-red-900/50">
                <p className="text-2xl font-bold text-red-400">{alertSummary.critical}</p>
                <p className="text-xs text-red-600">Critical</p>
              </div>
              <div className="p-3 bg-orange-900/30 rounded-lg border border-orange-900/50">
                <p className="text-2xl font-bold text-orange-400">{alertSummary.high}</p>
                <p className="text-xs text-orange-600">High</p>
              </div>
              <div className="p-3 bg-yellow-900/30 rounded-lg border border-yellow-900/50">
                <p className="text-2xl font-bold text-yellow-400">{alertSummary.medium}</p>
                <p className="text-xs text-yellow-600">Medium</p>
              </div>
              <div className="p-3 bg-blue-900/30 rounded-lg border border-blue-900/50">
                <p className="text-2xl font-bold text-blue-400">{alertSummary.low}</p>
                <p className="text-xs text-blue-600">Low</p>
              </div>
            </div>
          )}
        </div>

        {/* Active Incidents */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Active Incidents</h2>
            <a href="/operations/incidents" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          {incidentsQuery.isLoading ? (
            <div className="space-y-3">
              {[1, 2].map((i) => (
                <LoadingSkeleton key={i} className="h-20" />
              ))}
            </div>
          ) : incidents.length === 0 ? (
            <p className="text-sm text-gray-500 text-center py-4">No active incidents</p>
          ) : (
            <div className="space-y-3">
              {incidents.map((incident) => (
                <div
                  key={incident.id}
                  className="p-3 bg-surface-overlay/50 rounded-lg border border-surface-border"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <span className="text-xs font-mono text-gray-500">{incident.id.slice(0, 8)}</span>
                      <p className="text-sm font-medium text-white">{incident.title}</p>
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
                    <span>{incident.status}</span>
                    <span>{formatRelativeTime(incident.created_at)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Recent Alerts */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Recent Alerts</h2>
          <a href="/operations/alerts" className="text-sm text-primary-600 hover:text-primary-700">
            View all alerts
          </a>
        </div>
        {alertsQuery.isLoading ? (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <LoadingSkeleton key={i} className="h-12" />
            ))}
          </div>
        ) : alerts.length === 0 ? (
          <p className="text-sm text-gray-500 text-center py-4">No recent alerts</p>
        ) : (
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
              <tbody className="divide-y divide-surface-border">
                {alerts.map((alert) => (
                  <tr key={alert.id} className="text-sm">
                    <td className="py-3 font-medium text-white">{alert.rule_name}</td>
                    <td className="py-3">
                      <span
                        className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                          alert.severity
                        )}`}
                      >
                        {alert.severity}
                      </span>
                    </td>
                    <td className="py-3 text-gray-400">{alert.metric_name}</td>
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
                    <td className="py-3 text-gray-500">{formatRelativeTime(alert.fired_at)}</td>
                    <td className="py-3">
                      {alert.status === 'firing' && (
                        <button
                          onClick={() => handleAcknowledge(alert.id)}
                          disabled={acknowledgeMutation.isPending}
                          className="text-primary-600 hover:text-primary-700 text-xs font-medium disabled:opacity-50"
                        >
                          Acknowledge
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

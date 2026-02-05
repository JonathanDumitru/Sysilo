import { Activity, Server, Workflow, AlertTriangle } from 'lucide-react';

const stats = [
  { name: 'Active Integrations', value: '24', icon: Workflow, change: '+2 this week' },
  { name: 'Connected Agents', value: '8', icon: Server, change: 'All healthy' },
  { name: 'Runs Today', value: '156', icon: Activity, change: '98% success rate' },
  { name: 'Active Alerts', value: '3', icon: AlertTriangle, change: '2 warnings, 1 error' },
];

export function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
        <p className="text-gray-500">Overview of your integration platform</p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-6">
        {stats.map((stat) => (
          <div key={stat.name} className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-50 rounded-lg">
                <stat.icon className="w-5 h-5 text-primary-600" />
              </div>
            </div>
            <div className="mt-4">
              <p className="text-3xl font-bold text-gray-900">{stat.value}</p>
              <p className="text-sm font-medium text-gray-500">{stat.name}</p>
              <p className="text-xs text-gray-400 mt-1">{stat.change}</p>
            </div>
          </div>
        ))}
      </div>

      {/* Recent Activity */}
      <div className="grid grid-cols-2 gap-6">
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Recent Runs</h2>
          <div className="space-y-3">
            {[
              { name: 'Salesforce → Snowflake Sync', status: 'completed', time: '2 min ago' },
              { name: 'HubSpot Contact Import', status: 'running', time: '5 min ago' },
              { name: 'Daily Invoice Export', status: 'completed', time: '1 hour ago' },
              { name: 'Customer Data Validation', status: 'failed', time: '2 hours ago' },
            ].map((run, i) => (
              <div key={i} className="flex items-center justify-between py-2 border-b border-gray-50 last:border-0">
                <div>
                  <p className="text-sm font-medium text-gray-900">{run.name}</p>
                  <p className="text-xs text-gray-400">{run.time}</p>
                </div>
                <span
                  className={`text-xs font-medium px-2 py-1 rounded-full ${
                    run.status === 'completed'
                      ? 'bg-green-50 text-green-700'
                      : run.status === 'running'
                      ? 'bg-blue-50 text-blue-700'
                      : 'bg-red-50 text-red-700'
                  }`}
                >
                  {run.status}
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Agent Status</h2>
          <div className="space-y-3">
            {[
              { name: 'prod-agent-01', status: 'connected', location: 'AWS us-east-1' },
              { name: 'prod-agent-02', status: 'connected', location: 'AWS us-west-2' },
              { name: 'on-prem-agent', status: 'connected', location: 'Data Center' },
              { name: 'dev-agent', status: 'disconnected', location: 'Local' },
            ].map((agent, i) => (
              <div key={i} className="flex items-center justify-between py-2 border-b border-gray-50 last:border-0">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-2 h-2 rounded-full ${
                      agent.status === 'connected' ? 'bg-green-500' : 'bg-gray-300'
                    }`}
                  />
                  <div>
                    <p className="text-sm font-medium text-gray-900">{agent.name}</p>
                    <p className="text-xs text-gray-400">{agent.location}</p>
                  </div>
                </div>
                <span className="text-xs text-gray-500">{agent.status}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
